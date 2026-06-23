//! AST → IR lowering pass.
//!
//! This module transforms a type-checked [`ast::Program`] into an
//! [`IrModule`].  It is the only module permitted to import from both
//! `crate::ast` and `crate::ir` — keeping the two representations cleanly
//! separated.
//!
//! # Pipeline position
//!
//! ```text
//! ast::Program  (from Type Checker)
//!      │
//!      ▼  lower()
//! IrModule
//!      │
//!      ├──▶ Interpreter (future IR-based interpreter)
//!      └──▶ Native / LLVM / WASM backend
//! ```
//!
//! # Lowering strategy
//!
//! Each statement is lowered in-order.  Expressions are lowered
//! recursively and return an [`Operand`] that encodes their result
//! (either an inline constant or a virtual register).
//!
//! Control-flow statements (if/while/for) generate multiple basic blocks
//! and wire them together with jump/branch terminators.
//!
//! Top-level (module-scope) statements are collected into a synthesised
//! function called `__main__`.  User-defined `func` declarations become
//! their own `IrFunction`s.

use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use crate::diagnostics::CompilerError;

use super::builder::IrBuilder;
use super::function::IrParameter;
use super::instructions::{BinOpKind, Constant, Operand, UnaryOpKind};
use super::module::IrModule;
use super::types::IrType;

// ─── Public entry point ────────────────────────────────────────────────────

/// Lower a validated Bunzo [`Program`] into an [`IrModule`].
///
/// This is the sole entry point from the rest of the compiler.
/// It is called after semantic analysis and type checking succeed.
///
/// # Errors
///
/// Returns a [`CompilerError`] if a construct that should have been
/// rejected by an earlier pass is encountered unexpectedly.  Under
/// normal conditions this function does not fail — all reachable errors
/// represent internal consistency violations.
pub fn lower(program: &Program) -> Result<IrModule, CompilerError> {
    let mut ctx = LoweringContext::new("source.bz");
    ctx.lower_program(program)?;
    Ok(ctx.builder.finish())
}

// ─── Lowering Context ─────────────────────────────────────────────────────

/// Internal state for a single lowering pass.
///
/// Wraps an [`IrBuilder`] and tracks contextual information that the
/// builder itself does not need to know about, such as the loop-exit
/// label stack (needed to lower `break` and `continue`).
struct LoweringContext {
    builder: IrBuilder,

    /// Stack of (loop_header_label, loop_exit_label) pairs.
    ///
    /// The top of the stack is the innermost loop.  `break` jumps to the
    /// exit label; `continue` jumps to the header label.
    loop_stack: Vec<(String, String)>,
}

impl LoweringContext {
    fn new(source_name: &str) -> Self {
        Self {
            builder: IrBuilder::new(source_name),
            loop_stack: Vec::new(),
        }
    }

    // ── Program ─────────────────────────────────────────────────────────

    fn lower_program(&mut self, program: &Program) -> Result<(), CompilerError> {
        // Collect top-level statements that are NOT function declarations.
        // Function declarations are lowered as separate IrFunctions.
        let top_level_stmts: Vec<&Statement> = program
            .statements
            .iter()
            .filter(|s| !matches!(s, Statement::FunctionDeclaration { .. }))
            .collect();

        // Lower all top-level non-function statements into __main__.
        self.builder
            .begin_function("__main__", Vec::new(), IrType::Void);
        self.builder.begin_block("entry");

        for stmt in &top_level_stmts {
            self.lower_statement(stmt)?;
        }

        // Ensure __main__ ends with a return.
        self.ensure_terminated();
        self.builder.finish_function();

        // Lower function declarations as separate IrFunctions.
        for stmt in &program.statements {
            if let Statement::FunctionDeclaration {
                name,
                params,
                return_type,
                body,
                ..
            } = stmt
            {
                self.lower_function_declaration(name, params, return_type, body)?;
            }
        }

        Ok(())
    }

    // ── Function Declaration ─────────────────────────────────────────────

    fn lower_function_declaration(
        &mut self,
        name: &str,
        params: &[crate::ast::Parameter],
        return_type: &Option<String>,
        body: &[Statement],
    ) -> Result<(), CompilerError> {
        let ir_params: Vec<IrParameter> = params
            .iter()
            .map(|p| IrParameter {
                name: p.name.clone(),
                ty: type_name_to_ir_type(&p.type_name),
            })
            .collect();

        let ir_return_type = return_type
            .as_deref()
            .map(type_name_to_ir_type)
            .unwrap_or(IrType::Void);

        self.builder.begin_function(name, ir_params, ir_return_type);
        self.builder.begin_block("entry");

        for stmt in body {
            self.lower_statement(stmt)?;
        }

        self.ensure_terminated();
        self.builder.finish_function();
        Ok(())
    }

    // ── Statement Lowering ───────────────────────────────────────────────

    fn lower_statement(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        match stmt {
            Statement::LetDeclaration {
                name, initializer, ..
            } => {
                let operand = self.lower_expression(initializer)?;
                self.builder.emit_store(name, operand);
            }

            Statement::ConstDeclaration {
                name, initializer, ..
            } => {
                // Constants are treated identically to let at the IR level —
                // immutability is enforced by the semantic / type checker.
                let operand = self.lower_expression(initializer)?;
                self.builder.emit_store(name, operand);
            }

            Statement::Assignment { name, value, .. } => {
                let operand = self.lower_expression(value)?;
                self.builder.emit_store(name, operand);
            }

            Statement::PrintStatement { argument, .. } => {
                let operand = self.lower_expression(argument)?;
                self.builder.emit_print(operand);
            }

            Statement::ReturnStatement { value, .. } => match value {
                Some(expr) => {
                    let operand = self.lower_expression(expr)?;
                    self.builder.emit_return(operand);
                }
                None => {
                    self.builder.emit_return_void();
                }
            },

            Statement::ExpressionStatement { expression } => {
                // Lower for side effects (e.g. a bare function call).
                self.lower_expression_for_side_effects(expression)?;
            }

            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.lower_if(condition, then_branch, else_branch.as_deref())?;
            }

            Statement::WhileStatement {
                condition, body, ..
            } => {
                self.lower_while(condition, body)?;
            }

            Statement::ForStatement {
                variable,
                start,
                end,
                body,
                ..
            } => {
                self.lower_for(variable, start, end, body)?;
            }

            Statement::BreakStatement { .. } => {
                let exit_label = self
                    .loop_stack
                    .last()
                    .map(|(_, exit)| exit.clone())
                    .unwrap_or_else(|| "loop.exit".to_string());
                self.builder.emit_jump(exit_label);
            }

            Statement::ContinueStatement { .. } => {
                let header_label = self
                    .loop_stack
                    .last()
                    .map(|(header, _)| header.clone())
                    .unwrap_or_else(|| "loop.header".to_string());
                self.builder.emit_jump(header_label);
            }

            Statement::FunctionDeclaration { .. } => {
                // Nested function declarations are not lowered here;
                // they are handled at program level.
                // Ignore silently (already processed in lower_program).
            }

            // The following statement kinds are lowered minimally at this stage.
            // They will be fully expanded as later phases are implemented.
            Statement::StructDeclaration { .. }
            | Statement::ClassDeclaration { .. }
            | Statement::InterfaceDeclaration { .. }
            | Statement::EnumDeclaration { .. }
            | Statement::ImportDeclaration { .. }
            | Statement::ExportDeclaration { .. }
            | Statement::FieldAssignment { .. }
            | Statement::TryCatch { .. }
            | Statement::Throw { .. }
            | Statement::MatchStatement { .. }
            | Statement::SpawnStatement { .. } => {
                // These are kept for future stages (IR is foundation only).
                // No panic — just silently produce no instructions so that
                // programs using them still produce a valid (partial) module.
            }
        }
        Ok(())
    }

    // ── If / Else Lowering ───────────────────────────────────────────────

    fn lower_if(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&[Statement]>,
    ) -> Result<(), CompilerError> {
        let then_label = self.builder.fresh_label("then");
        let else_label = self.builder.fresh_label("else");
        let merge_label = self.builder.fresh_label("merge");

        // Emit the conditional branch in the current block.
        let cond_op = self.lower_expression(condition)?;
        self.builder
            .emit_branch(cond_op, then_label.clone(), else_label.clone());

        // ── then block ────────────────────────────────────────
        self.builder.begin_block(then_label);
        for stmt in then_branch {
            self.lower_statement(stmt)?;
        }
        if !self.current_block_is_terminated() {
            self.builder.emit_jump(merge_label.clone());
        }

        // ── else block ────────────────────────────────────────
        self.builder.begin_block(else_label);
        if let Some(else_stmts) = else_branch {
            for stmt in else_stmts {
                self.lower_statement(stmt)?;
            }
        }
        if !self.current_block_is_terminated() {
            self.builder.emit_jump(merge_label.clone());
        }

        // ── merge block ───────────────────────────────────────
        self.builder.begin_block(merge_label);
        Ok(())
    }

    // ── While Loop Lowering ──────────────────────────────────────────────

    fn lower_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<(), CompilerError> {
        let header_label = self.builder.fresh_label("loop.header");
        let body_label = self.builder.fresh_label("loop.body");
        let exit_label = self.builder.fresh_label("loop.exit");

        // Jump into the loop header from the current block.
        self.builder.emit_jump(header_label.clone());

        // ── header block (condition check) ────────────────────
        self.builder.begin_block(header_label.clone());
        let cond_op = self.lower_expression(condition)?;
        self.builder
            .emit_branch(cond_op, body_label.clone(), exit_label.clone());

        // ── body block ────────────────────────────────────────
        self.builder.begin_block(body_label);

        // Push loop context so `break`/`continue` resolve correctly.
        self.loop_stack
            .push((header_label.clone(), exit_label.clone()));

        for stmt in body {
            self.lower_statement(stmt)?;
        }

        self.loop_stack.pop();

        if !self.current_block_is_terminated() {
            self.builder.emit_jump(header_label);
        }

        // ── exit block ────────────────────────────────────────
        self.builder.begin_block(exit_label);
        Ok(())
    }

    // ── For Loop Lowering ────────────────────────────────────────────────

    /// Lower `for variable in start..end { body }`.
    ///
    /// Equivalent pseudo-IR:
    /// ```text
    ///   store variable <start>
    ///   jump loop.header.N
    /// loop.header.N:
    ///   %cmp = lt (load variable) <end>
    ///   branch %cmp loop.body.N loop.exit.N
    /// loop.body.N:
    ///   <body>
    ///   %next = add (load variable) 1
    ///   store variable %next
    ///   jump loop.header.N
    /// loop.exit.N:
    /// ```
    fn lower_for(
        &mut self,
        variable: &str,
        start: &Expression,
        end: &Expression,
        body: &[Statement],
    ) -> Result<(), CompilerError> {
        let header_label = self.builder.fresh_label("loop.header");
        let body_label = self.builder.fresh_label("loop.body");
        let exit_label = self.builder.fresh_label("loop.exit");

        // Initialise the loop variable.
        let start_op = self.lower_expression(start)?;
        self.builder.emit_store(variable, start_op);
        self.builder.emit_jump(header_label.clone());

        // ── header block ──────────────────────────────────────
        self.builder.begin_block(header_label.clone());
        let loop_var = self.builder.emit_load(variable);
        let end_op = self.lower_expression(end)?;
        let cmp = self
            .builder
            .emit_binop(BinOpKind::Less, Operand::Register(loop_var), end_op);
        self.builder.emit_branch(
            Operand::Register(cmp),
            body_label.clone(),
            exit_label.clone(),
        );

        // ── body block ────────────────────────────────────────
        self.builder.begin_block(body_label);

        self.loop_stack
            .push((header_label.clone(), exit_label.clone()));

        for stmt in body {
            self.lower_statement(stmt)?;
        }

        self.loop_stack.pop();

        if !self.current_block_is_terminated() {
            // Increment the loop variable.
            let current = self.builder.emit_load(variable);
            let one = self.builder.emit_const(IrType::Int, Constant::Int(1));
            let next = self.builder.emit_binop(
                BinOpKind::Add,
                Operand::Register(current),
                Operand::Register(one),
            );
            self.builder.emit_store(variable, Operand::Register(next));
            self.builder.emit_jump(header_label);
        }

        // ── exit block ────────────────────────────────────────
        self.builder.begin_block(exit_label);
        Ok(())
    }

    // ── Expression Lowering ──────────────────────────────────────────────

    /// Lower an expression, returning an [`Operand`] that encodes its result.
    ///
    /// Constant literals are returned as inline [`Operand::Constant`] values
    /// without emitting any instruction.  Everything else emits one or more
    /// instructions and returns an [`Operand::Register`].
    fn lower_expression(&mut self, expr: &Expression) -> Result<Operand, CompilerError> {
        match expr {
            Expression::IntegerLiteral { value, .. } => {
                Ok(Operand::Constant(Constant::Int(*value)))
            }

            Expression::FloatLiteral { value, .. } => {
                Ok(Operand::Constant(Constant::Float(*value)))
            }

            Expression::StringLiteral { value, .. } => {
                Ok(Operand::Constant(Constant::String(value.clone())))
            }

            Expression::BooleanLiteral { value, .. } => {
                Ok(Operand::Constant(Constant::Bool(*value)))
            }

            Expression::NullLiteral { .. } => Ok(Operand::Constant(Constant::Null)),

            Expression::Identifier { name, .. } => {
                let reg = self.builder.emit_load(name);
                Ok(Operand::Register(reg))
            }

            Expression::Grouping { expression, .. } => self.lower_expression(expression),

            Expression::BinaryOp {
                operator,
                left,
                right,
                ..
            } => {
                let left_op = self.lower_expression(left)?;
                let right_op = self.lower_expression(right)?;
                let kind = binary_operator_to_ir(operator);
                let reg = self.builder.emit_binop(kind, left_op, right_op);
                Ok(Operand::Register(reg))
            }

            Expression::UnaryOp {
                operator, operand, ..
            } => {
                let inner = self.lower_expression(operand)?;
                let kind = unary_operator_to_ir(operator);
                let reg = self.builder.emit_unary_op(kind, inner);
                Ok(Operand::Register(reg))
            }

            Expression::Call {
                callee, arguments, ..
            } => {
                let callee_op = self.lower_call_callee(callee)?;
                let mut arg_ops = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    arg_ops.push(self.lower_expression(arg)?);
                }
                let reg = self.builder.emit_call(callee_op, arg_ops);
                Ok(Operand::Register(reg))
            }

            Expression::FieldAccess { object, field, .. } => {
                let obj_op = self.lower_expression(object)?;
                let reg = self.builder.emit_get_field(obj_op, field.as_str());
                Ok(Operand::Register(reg))
            }

            Expression::ArrayLiteral { .. }
            | Expression::IndexExpression { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariantExpr { .. }
            | Expression::PropagateError { .. }
            | Expression::MoveExpr { .. }
            | Expression::AwaitExpr { .. }
            | Expression::SuperExpr { .. } => {
                // These will be fully lowered in future phases.
                // For now emit a null constant so programs still compile.
                Ok(Operand::Constant(Constant::Null))
            }
        }
    }

    /// Lower an expression purely for its side effects (discard result).
    ///
    /// Used for bare function-call expression statements where the return
    /// value is not captured.
    fn lower_expression_for_side_effects(
        &mut self,
        expr: &Expression,
    ) -> Result<(), CompilerError> {
        match expr {
            Expression::Call {
                callee, arguments, ..
            } => {
                let callee_op = self.lower_call_callee(callee)?;
                let mut arg_ops = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    arg_ops.push(self.lower_expression(arg)?);
                }
                self.builder.emit_call_void(callee_op, arg_ops);
            }
            other => {
                // Non-call expression statements: lower anyway to catch any
                // register-allocating sub-expressions (e.g. unary ops with
                // side effects in future extensions).
                let _ = self.lower_expression(other)?;
            }
        }
        Ok(())
    }

    /// Lower the callee position of a call expression.
    ///
    /// Named function references become [`Operand::FunctionRef`];
    /// everything else (method calls, closures) is lowered as a normal
    /// expression and becomes a register operand.
    fn lower_call_callee(&mut self, callee: &Expression) -> Result<Operand, CompilerError> {
        match callee {
            Expression::Identifier { name, .. } => Ok(Operand::FunctionRef(name.clone())),
            other => self.lower_expression(other),
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Returns `true` if the currently active basic block already has
    /// a terminator as its last instruction.
    fn current_block_is_terminated(&mut self) -> bool {
        // We reach into the module to check the last instruction without
        // triggering borrow-checker issues from using builder methods.
        let func_idx = match self.builder.module.functions.len().checked_sub(1) {
            Some(i) => i,
            None => return false,
        };
        let func = &self.builder.module.functions[func_idx];
        func.blocks
            .last()
            .map(|b| b.is_terminated())
            .unwrap_or(false)
    }

    /// Ensures the current basic block ends with a terminator.
    ///
    /// If the block is already terminated (e.g. by an explicit `return`)
    /// this is a no-op.  Otherwise a bare `return` is appended so the IR
    /// remains well-formed.
    fn ensure_terminated(&mut self) {
        if !self.current_block_is_terminated() {
            self.builder.emit_return_void();
        }
    }
}

// ─── Operator Mapping ──────────────────────────────────────────────────────

fn binary_operator_to_ir(op: &BinaryOperator) -> BinOpKind {
    match op {
        BinaryOperator::Add => BinOpKind::Add,
        BinaryOperator::Subtract => BinOpKind::Subtract,
        BinaryOperator::Multiply => BinOpKind::Multiply,
        BinaryOperator::Divide => BinOpKind::Divide,
        BinaryOperator::Modulo => BinOpKind::Modulo,
        BinaryOperator::Equal => BinOpKind::Equal,
        BinaryOperator::NotEqual => BinOpKind::NotEqual,
        BinaryOperator::Less => BinOpKind::Less,
        BinaryOperator::Greater => BinOpKind::Greater,
        BinaryOperator::LessEqual => BinOpKind::LessEqual,
        BinaryOperator::GreaterEqual => BinOpKind::GreaterEqual,
        BinaryOperator::And => BinOpKind::And,
        BinaryOperator::Or => BinOpKind::Or,
    }
}

fn unary_operator_to_ir(op: &UnaryOperator) -> UnaryOpKind {
    match op {
        UnaryOperator::Negate => UnaryOpKind::Negate,
        UnaryOperator::LogicalNot => UnaryOpKind::Not,
    }
}

// ─── Type Mapping ──────────────────────────────────────────────────────────

/// Map a Bunzo source type annotation string to an [`IrType`].
fn type_name_to_ir_type(name: &str) -> IrType {
    match name {
        "int" => IrType::Int,
        "float" => IrType::Float,
        "string" => IrType::String,
        "bool" => IrType::Bool,
        "null" => IrType::Null,
        "void" => IrType::Void,
        other => IrType::Class(other.to_string()),
    }
}
