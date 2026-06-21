//! AST evaluation and execution engine for the Bunzo runtime.
//!
//! Covers Phases 1–3 + control flow + try/catch + stdlib builtins.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator, Visibility};
use crate::diagnostics::CompilerError;
use crate::stdlib::{build_module, register_builtins};
use super::environment::Environment;
use super::value::{BzClass, BzFunction, RuntimeValue};

// ── Control-flow signals ──────────────────────────────────────────────────

/// Internal signal used to propagate non-error control-flow through the
/// call stack without unwinding through every match arm.
enum Signal {
    /// A `return` statement was hit; carry the return value.
    Return(RuntimeValue),
    /// A `break` statement was hit.
    Break,
    /// A `continue` statement was hit.
    Continue,
    /// A `throw` statement was hit; carry the thrown value.
    Throw(RuntimeValue, usize, usize),
}

/// Result of executing a statement: either nothing interesting happened,
/// or a control-flow signal needs to propagate.
type StmtResult = Result<Option<Signal>, CompilerError>;

// ── Public API ────────────────────────────────────────────────────────────

/// Executes a complete Bunzo program, writing output to stdout.
pub fn execute(program: Program) -> Result<(), CompilerError> {
    let mut interpreter = Interpreter::new(std::io::stdout());
    interpreter.interpret(program)
}

// ── Interpreter ───────────────────────────────────────────────────────────

/// The Bunzo AST interpreter.
pub struct Interpreter<W: std::io::Write> {
    environment: Rc<RefCell<Environment>>,
    stdout: W,
    /// Class whose method is currently executing (`this` / `super` context).
    method_class: Option<String>,
    /// Receiver object for the active method call.
    method_receiver: Option<Rc<RuntimeValue>>,
}

impl<W: std::io::Write> Interpreter<W> {
    /// Creates a new interpreter writing output to the given stream.
    pub fn new(stdout: W) -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));
        // Register stdlib builtins into the global environment.
        register_builtins(&env);
        Self {
            environment: env,
            stdout,
            method_class: None,
            method_receiver: None,
        }
    }

    /// Interprets a complete program.
    pub fn interpret(&mut self, program: Program) -> Result<(), CompilerError> {
        for stmt in &program.statements {
            if let Some(signal) = self.exec_stmt(stmt)? {
                match signal {
                    Signal::Throw(val, line, col) => {
                        return Err(CompilerError::Thrown { value: val, line, column: col });
                    }
                    Signal::Return(_) => {
                        return Err(CompilerError::ReturnOutsideFunction { line: 0, column: 0 });
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // ── Statement execution ───────────────────────────────────────────

    fn exec_stmt(&mut self, stmt: &Statement) -> StmtResult {
        match stmt {
            Statement::LetDeclaration { name, initializer, line, column } => {
                let value = self.eval_expr(initializer)?;
                self.environment.borrow_mut().define(
                    name.clone(), value, false, *line, *column,
                )?;
                Ok(None)
            }
            Statement::ConstDeclaration { name, initializer, line, column } => {
                let value = self.eval_expr(initializer)?;
                self.environment.borrow_mut().define(
                    name.clone(), value, true, *line, *column,
                )?;
                Ok(None)
            }
            Statement::PrintStatement { argument, .. } => {
                let value = self.eval_expr(argument)?;
                writeln!(self.stdout, "{value}").map_err(CompilerError::Io)?;
                Ok(None)
            }
            Statement::ExpressionStatement { expression } => {
                self.eval_expr(expression)?;
                Ok(None)
            }
            Statement::FunctionDeclaration { name, params, body, line, column, visibility, is_abstract, .. } => {
                if *is_abstract {
                    return Ok(None);
                }
                let func = RuntimeValue::Function(Rc::new(BzFunction {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                    owner_class: None,
                    visibility: *visibility,
                    is_abstract: false,
                }));
                self.environment.borrow_mut().define(
                    name.clone(), func, true, *line, *column,
                )?;
                Ok(None)
            }
            Statement::ReturnStatement { value, .. } => {
                let val = match value {
                    Some(expr) => self.eval_expr(expr)?,
                    None => RuntimeValue::Null,
                };
                Ok(Some(Signal::Return(val)))
            }
            Statement::Assignment { name, value, line, column } => {
                let val = self.eval_expr(value)?;
                self.environment.borrow_mut().assign(
                    name.clone(), val, *line, *column,
                )?;
                Ok(None)
            }
            Statement::IfStatement { condition, then_branch, else_branch, line, column } => {
                let cond = self.eval_expr(condition)?;
                let cond_bool = require_bool(cond, "if condition", *line, *column)?;
                let branch = if cond_bool {
                    Some(then_branch.as_slice())
                } else {
                    else_branch.as_deref()
                };
                if let Some(stmts) = branch {
                    let child = Rc::new(RefCell::new(
                        Environment::with_parent(self.environment.clone()),
                    ));
                    return self.exec_block_in_env(stmts, child);
                }
                Ok(None)
            }
            Statement::WhileStatement { condition, body, line, column } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    let cond_bool = require_bool(cond, "while condition", *line, *column)?;
                    if !cond_bool { break; }
                    let child = Rc::new(RefCell::new(
                        Environment::with_parent(self.environment.clone()),
                    ));
                    match self.exec_block_in_env(body, child)? {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => continue,
                        other => {
                            if other.is_some() { return Ok(other); }
                        }
                    }
                }
                Ok(None)
            }
            Statement::ForStatement { variable, start, end, body, line, column } => {
                let start_val = self.eval_expr(start)?;
                let end_val   = self.eval_expr(end)?;
                let (start_i, end_i) = match (start_val, end_val) {
                    (RuntimeValue::Integer(s), RuntimeValue::Integer(e)) => (s, e),
                    _ => return Err(CompilerError::TypeMismatch {
                        operation: "for range".to_string(),
                        expected:  "Integer".to_string(),
                        found:     "non-integer".to_string(),
                        line: *line, column: *column,
                    }),
                };
                let mut i = start_i;
                while i < end_i {
                    let child = Rc::new(RefCell::new(
                        Environment::with_parent(self.environment.clone()),
                    ));
                    child.borrow_mut().define(
                        variable.clone(),
                        RuntimeValue::Integer(i),
                        false, *line, *column,
                    )?;
                    match self.exec_block_in_env(body, child)? {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => { i += 1; continue; }
                        other => {
                            if other.is_some() { return Ok(other); }
                        }
                    }
                    i += 1;
                }
                Ok(None)
            }
            Statement::BreakStatement { .. } => Ok(Some(Signal::Break)),
            Statement::ContinueStatement { .. } => Ok(Some(Signal::Continue)),
            Statement::StructDeclaration { name, fields, line, column } => {
                // Struct declarations don't produce a runtime value; they
                // are validated by semantic analysis. Register a sentinel
                // so that field-list metadata can be checked at construction.
                // We store the field order in the env as a Function sentinel.
                let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                let func = RuntimeValue::Function(Rc::new(BzFunction {
                    name: format!("__struct__{name}"),
                    params: field_names,
                    body: vec![],
                    closure: self.environment.clone(),
                    owner_class: None,
                    visibility: Visibility::Public,
                    is_abstract: false,
                }));
                self.environment.borrow_mut().define(
                    format!("__struct__{name}"), func, true, *line, *column,
                ).unwrap_or(()); // ignore if re-defined in tests
                Ok(None)
            }
            Statement::ClassDeclaration {
                name,
                fields,
                methods,
                line,
                column,
                extends,
                is_abstract,
                ..
            } => {
                let mut method_map: HashMap<String, Rc<BzFunction>> = HashMap::new();
                let mut field_visibility: HashMap<String, Visibility> = HashMap::new();
                let mut all_fields: Vec<String> = Vec::new();
                let parent_name = extends.clone();

                if let Some(ref parent_name) = parent_name {
                    if let Ok(RuntimeValue::Class(parent_class)) =
                        self.environment.borrow().get(parent_name, *line, *column)
                    {
                        all_fields = parent_class.fields.clone();
                        field_visibility = parent_class.field_visibility.clone();
                        for (mname, mfunc) in &parent_class.methods {
                            method_map.insert(mname.clone(), mfunc.clone());
                        }
                    }
                }

                for f in fields {
                    if !all_fields.contains(&f.name) {
                        all_fields.push(f.name.clone());
                    }
                    field_visibility.insert(f.name.clone(), f.visibility);
                }

                for m in methods {
                    if let Statement::FunctionDeclaration {
                        name: mname,
                        params,
                        body,
                        visibility,
                        is_abstract: method_abstract,
                        ..
                    } = m
                    {
                        if *method_abstract {
                            continue;
                        }
                        method_map.insert(
                            mname.clone(),
                            Rc::new(BzFunction {
                                name: mname.clone(),
                                params: params.iter().map(|p| p.name.clone()).collect(),
                                body: body.clone(),
                                closure: self.environment.clone(),
                                owner_class: Some(name.clone()),
                                visibility: *visibility,
                                is_abstract: false,
                            }),
                        );
                    }
                }

                let class = RuntimeValue::Class(Rc::new(BzClass {
                    name: name.clone(),
                    parent: parent_name,
                    is_abstract: *is_abstract,
                    fields: all_fields,
                    field_visibility,
                    methods: method_map,
                }));
                self.environment.borrow_mut().define(
                    name.clone(), class, true, *line, *column,
                )?;
                Ok(None)
            }
            Statement::FieldAssignment { object, field, value, line, column } => {
                let obj_val = self.eval_expr(object)?;
                let new_val = self.eval_expr(value)?;
                match &obj_val {
                    RuntimeValue::Object { class_name, field_visibility, fields, .. } => {
                        if matches!(field_visibility.get(field), Some(Visibility::Private)) {
                            if self.method_class.as_deref() != Some(class_name.as_str()) {
                                return Err(CompilerError::PrivateFieldAccess {
                                    class_name: class_name.clone(),
                                    field: field.clone(),
                                    line: *line,
                                    column: *column,
                                });
                            }
                        }
                        fields.borrow_mut().insert(field.clone(), new_val);
                    }
                    RuntimeValue::Struct { fields: _, .. } => {
                        // structs are value types — need to locate and update the var
                        // We fall back to re-assigning through the identifier in `object`.
                        drop(obj_val);
                        return self.exec_field_assign_struct(object, field, new_val, *line, *column);
                    }
                    other => return Err(CompilerError::TypeMismatch {
                        operation: "field assignment".to_string(),
                        expected:  "Object or Struct".to_string(),
                        found:     other.type_name().to_string(),
                        line: *line, column: *column,
                    }),
                }
                Ok(None)
            }
            Statement::TryCatch { try_block, catch_var, catch_block, line, column } => {
                let child = Rc::new(RefCell::new(
                    Environment::with_parent(self.environment.clone()),
                ));
                let result = self.exec_block_in_env(try_block, child);
                match result {
                    Ok(Some(Signal::Throw(val, _, _))) | Err(CompilerError::Thrown { value: val, .. }) => {
                        let catch_env = Rc::new(RefCell::new(
                            Environment::with_parent(self.environment.clone()),
                        ));
                        catch_env.borrow_mut().define(
                            catch_var.clone(), val, false, *line, *column,
                        )?;
                        self.exec_block_in_env(catch_block, catch_env)
                    }
                    other => other,
                }
            }
            Statement::Throw { value, line, column } => {
                let val = self.eval_expr(value)?;
                Ok(Some(Signal::Throw(val, *line, *column)))
            }
            // ── Phase 4+ statements ───────────────────────────────────
            Statement::ImportDeclaration { name, path, line, column } => {
                self.exec_import(name, path.as_deref(), *line, *column)
            }
            Statement::ExportDeclaration { .. } => Ok(None), // export is metadata; no runtime effect
            Statement::EnumDeclaration { name, variants, line, column } => {
                self.exec_enum_declaration(name, variants, *line, *column)
            }
            Statement::MatchStatement { subject, arms, line, column } => {
                self.exec_match(subject, arms, *line, *column)
            }
            Statement::InterfaceDeclaration { .. } => Ok(None), // structural — no runtime value
            Statement::SpawnStatement { expression, line, column } => {
                self.exec_spawn(expression, *line, *column)
            }
        }
    }

    // ── Block helpers ─────────────────────────────────────────────────

    /// Execute a slice of statements in a given environment, then restore
    /// the caller's environment before returning.
    fn exec_block_in_env(
        &mut self,
        stmts: &[Statement],
        child: Rc<RefCell<Environment>>,
    ) -> StmtResult {
        let previous = std::mem::replace(&mut self.environment, child);
        let result = self.exec_block(stmts);
        self.environment = previous;
        result
    }

    fn exec_block(&mut self, stmts: &[Statement]) -> StmtResult {
        for stmt in stmts {
            if let Some(signal) = self.exec_stmt(stmt)? {
                return Ok(Some(signal));
            }
        }
        Ok(None)
    }

    /// Handle field assignment on a struct stored in a plain variable.
    /// Structs are value types, so we read, mutate, and write back.
    fn exec_field_assign_struct(
        &mut self,
        object: &Expression,
        field: &str,
        new_val: RuntimeValue,
        line: usize,
        column: usize,
    ) -> StmtResult {
        if let Expression::Identifier { name, .. } = object {
            let current = self.environment.borrow().get(name, line, column)?;
            if let RuntimeValue::Struct { name: sname, mut fields } = current {
                fields.insert(field.to_string(), new_val);
                let updated = RuntimeValue::Struct { name: sname, fields };
                self.environment.borrow_mut().assign(name.clone(), updated, line, column)?;
                return Ok(None);
            }
        }
        Err(CompilerError::TypeMismatch {
            operation: "field assignment".to_string(),
            expected:  "Struct".to_string(),
            found:     "other".to_string(),
            line, column,
        })
    }

    // ── Expression evaluation ─────────────────────────────────────────

    pub fn evaluate_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<RuntimeValue, CompilerError> {
        self.eval_expr(expr)
    }

    pub(crate) fn eval_expr(&mut self, expr: &Expression) -> Result<RuntimeValue, CompilerError> {
        match expr {
            Expression::IntegerLiteral { value, .. } => Ok(RuntimeValue::Integer(*value)),
            Expression::FloatLiteral   { value, .. } => Ok(RuntimeValue::Float(*value)),
            Expression::StringLiteral  { value, .. } => Ok(RuntimeValue::String(value.clone())),
            Expression::BooleanLiteral { value, .. } => Ok(RuntimeValue::Boolean(*value)),
            Expression::NullLiteral    { .. }        => Ok(RuntimeValue::Null),
            Expression::Identifier { name, line, column } => {
                self.environment.borrow().get(name, *line, *column)
            }
            Expression::Grouping { expression, .. } => self.eval_expr(expression),
            Expression::UnaryOp  { operator, operand, line, column } => {
                self.eval_unary(operator, operand, *line, *column)
            }
            Expression::BinaryOp { operator, left, right, line, column } => {
                self.eval_binary(operator, left, right, *line, *column)
            }
            Expression::Call { callee, arguments, line, column } => {
                self.eval_call(callee, arguments, *line, *column)
            }
            Expression::StructLiteral { name, fields, line, column } => {
                self.eval_struct_literal(name, fields, *line, *column)
            }
            Expression::FieldAccess { object, field, line, column } => {
                self.eval_field_access(object, field, *line, *column)
            }
            // ── Phase 4+ expressions ──────────────────────────────────
            Expression::ArrayLiteral { elements, .. } => {
                let mut vals = Vec::with_capacity(elements.len());
                for el in elements { vals.push(self.eval_expr(el)?); }
                Ok(RuntimeValue::Array(std::rc::Rc::new(std::cell::RefCell::new(vals))))
            }
            Expression::IndexExpression { object, index, line, column } => {
                self.eval_index(object, index, *line, *column)
            }
            Expression::EnumVariantExpr { enum_name, variant, payload, line, column } => {
                let payload_val = if let Some(p) = payload {
                    Some(Box::new(self.eval_expr(p)?))
                } else {
                    None
                };
                Ok(RuntimeValue::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    payload: payload_val.map(|b| std::rc::Rc::new(*b)),
                })
            }
            Expression::PropagateError { expression, line, column } => {
                // Evaluate inner expression; if it results in a Thrown error, re-propagate.
                let val = self.eval_expr(expression)?;
                match val {
                    RuntimeValue::Error(msg) => Err(CompilerError::Thrown {
                        value: RuntimeValue::String(msg),
                        line: *line,
                        column: *column,
                    }),
                    other => Ok(other),
                }
            }
            Expression::MoveExpr { name, line, column } => {
                // Get the value and mark the variable as moved (set to Null).
                let val = self.environment.borrow().get(name, *line, *column)?;
                self.environment.borrow_mut().assign(name.clone(), RuntimeValue::Moved, *line, *column)?;
                Ok(val)
            }
            Expression::AwaitExpr { expression, line, column } => {
                // For channels: recv(). For futures: resolve immediately (no async runtime yet).
                let val = self.eval_expr(expression)?;
                match val {
                    RuntimeValue::Channel(ch) => {
                        let received = ch.lock().unwrap().recv()
                            .map_err(|_| CompilerError::RuntimeException {
                                message: "channel closed".to_string(),
                                line: *line, column: *column,
                            })?;
                        Ok(received)
                    }
                    other => Ok(other), // non-channel await is a no-op
                }
            }
            Expression::SuperExpr { line, column } => {
                let class_name = self.method_class.clone().ok_or(CompilerError::InvalidSuper {
                    line: *line,
                    column: *column,
                })?;
                let receiver = self.method_receiver.clone().ok_or(CompilerError::InvalidSuper {
                    line: *line,
                    column: *column,
                })?;
                let parent_class = self.lookup_class(&class_name, *line, *column)?
                    .parent
                    .clone()
                    .ok_or(CompilerError::RuntimeException {
                        message: format!("class \"{class_name}\" has no parent for super"),
                        line: *line,
                        column: *column,
                    })?;
                Ok(RuntimeValue::SuperHandle {
                    receiver,
                    parent_class,
                })
            }
        }
    }

    // ── Unary evaluation ──────────────────────────────────────────────

    fn eval_unary(
        &mut self,
        op: &UnaryOperator,
        operand: &Expression,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        let val = self.eval_expr(operand)?;
        match op {
            UnaryOperator::Negate => match val {
                RuntimeValue::Integer(v) => Ok(RuntimeValue::Integer(v.wrapping_neg())),
                RuntimeValue::Float(v)   => Ok(RuntimeValue::Float(-v)),
                other => Err(CompilerError::TypeMismatch {
                    operation: "unary negation '-'".to_string(),
                    expected:  "Integer or Float".to_string(),
                    found:     other.type_name().to_string(),
                    line, column,
                }),
            },
            UnaryOperator::LogicalNot => match val {
                RuntimeValue::Boolean(v) => Ok(RuntimeValue::Boolean(!v)),
                other => Err(CompilerError::TypeMismatch {
                    operation: "logical negation '!'".to_string(),
                    expected:  "Boolean".to_string(),
                    found:     other.type_name().to_string(),
                    line, column,
                }),
            },
        }
    }

    // ── Binary evaluation ─────────────────────────────────────────────

    fn eval_binary(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        // Short-circuit logical operators first.
        if *op == BinaryOperator::And {
            let lv = self.eval_expr(left)?;
            let lb = require_bool(lv, "logical AND '&&'", line, column)?;
            if !lb { return Ok(RuntimeValue::Boolean(false)); }
            let rv = self.eval_expr(right)?;
            let rb = require_bool(rv, "logical AND '&&'", line, column)?;
            return Ok(RuntimeValue::Boolean(rb));
        }
        if *op == BinaryOperator::Or {
            let lv = self.eval_expr(left)?;
            let lb = require_bool(lv, "logical OR '||'", line, column)?;
            if lb { return Ok(RuntimeValue::Boolean(true)); }
            let rv = self.eval_expr(right)?;
            let rb = require_bool(rv, "logical OR '||'", line, column)?;
            return Ok(RuntimeValue::Boolean(rb));
        }

        let lv = self.eval_expr(left)?;
        let rv = self.eval_expr(right)?;

        match op {
            BinaryOperator::Add         => eval_add(lv, rv, line, column),
            BinaryOperator::Subtract    => eval_arithmetic(lv, rv, "-", line, column),
            BinaryOperator::Multiply    => eval_arithmetic(lv, rv, "*", line, column),
            BinaryOperator::Divide      => eval_division(lv, rv, "/", line, column),
            BinaryOperator::Modulo      => eval_division(lv, rv, "%", line, column),
            BinaryOperator::Equal       => Ok(RuntimeValue::Boolean(eval_equality(&lv, &rv))),
            BinaryOperator::NotEqual    => Ok(RuntimeValue::Boolean(!eval_equality(&lv, &rv))),
            BinaryOperator::Less        => eval_comparison(lv, rv, "<",  line, column),
            BinaryOperator::Greater     => eval_comparison(lv, rv, ">",  line, column),
            BinaryOperator::LessEqual   => eval_comparison(lv, rv, "<=", line, column),
            BinaryOperator::GreaterEqual=> eval_comparison(lv, rv, ">=", line, column),
            BinaryOperator::And | BinaryOperator::Or => unreachable!(),
        }
    }

    // ── Call evaluation ───────────────────────────────────────────────

    fn eval_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        // Evaluate callee, then decide what kind of call it is.
        let callee_val = self.eval_expr(callee)?;

        // Evaluate all arguments eagerly.
        let mut arg_vals: Vec<RuntimeValue> = Vec::with_capacity(arguments.len());
        for arg in arguments {
            arg_vals.push(self.eval_expr(arg)?);
        }

        self.call_value(callee_val, arg_vals, line, column)
    }

    fn call_value(
        &mut self,
        callee: RuntimeValue,
        args: Vec<RuntimeValue>,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        match callee {
            RuntimeValue::Function(func) => {
                self.call_function(func, args, line, column)
            }
            RuntimeValue::BoundMethod { receiver, method } => {
                self.call_bound_method(receiver, method, args, line, column)
            }
            RuntimeValue::Builtin { func, .. } => {
                func(args, line, column)
            }
            RuntimeValue::Class(class) => {
                if class.is_abstract {
                    return Err(CompilerError::AbstractClassInstantiation {
                        name: class.name.clone(),
                        line,
                        column,
                    });
                }
                let fields_map: HashMap<String, RuntimeValue> = class
                    .fields
                    .iter()
                    .map(|f| (f.clone(), RuntimeValue::Null))
                    .collect();
                let receiver = Rc::new(RuntimeValue::Object {
                    class_name: class.name.clone(),
                    parent_class: class.parent.clone(),
                    fields: Rc::new(RefCell::new(fields_map)),
                    methods: class.methods.clone(),
                    field_visibility: class.field_visibility.clone(),
                });

                if let Some(init) = class.methods.get("init") {
                    if init.params.len() != args.len() {
                        return Err(CompilerError::ArityMismatch {
                            name: format!("{}.init", class.name),
                            expected: init.params.len(),
                            found: args.len(),
                            line,
                            column,
                        });
                    }
                    self.call_bound_method(receiver.clone(), init.clone(), args, line, column)?;
                } else if !args.is_empty() {
                    return Err(CompilerError::ArityMismatch {
                        name: class.name.clone(),
                        expected: 0,
                        found: args.len(),
                        line,
                        column,
                    });
                }

                Ok(Rc::try_unwrap(receiver).unwrap_or_else(|rc| (*rc).clone()))
            }
            other => Err(CompilerError::NotCallable {
                found:  format!("{}", other.type_name()),
                line, column,
            }),
        }
    }

    fn call_function(
        &mut self,
        func: Rc<BzFunction>,
        args: Vec<RuntimeValue>,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        if func.params.len() != args.len() {
            return Err(CompilerError::ArityMismatch {
                name:     func.name.clone(),
                expected: func.params.len(),
                found:    args.len(),
                line, column,
            });
        }
        // Build the call environment from the closure.
        let call_env = Rc::new(RefCell::new(
            Environment::with_parent(func.closure.clone()),
        ));
        for (name, val) in func.params.iter().zip(args.into_iter()) {
            call_env.borrow_mut().define(name.clone(), val, false, line, column)?;
        }
        let previous = std::mem::replace(&mut self.environment, call_env);
        let result = self.exec_block(&func.body);
        self.environment = previous;
        match result? {
            Some(Signal::Return(v)) => Ok(v),
            Some(Signal::Throw(val, line, col)) => Err(CompilerError::Thrown { value: val, line, column: col }),
            _ => Ok(RuntimeValue::Null),
        }
    }

    fn call_bound_method(
        &mut self,
        receiver: Rc<RuntimeValue>,
        method: Rc<BzFunction>,
        args: Vec<RuntimeValue>,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        if method.params.len() != args.len() {
            return Err(CompilerError::ArityMismatch {
                name:     method.name.clone(),
                expected: method.params.len(),
                found:    args.len(),
                line, column,
            });
        }
        let call_env = Rc::new(RefCell::new(
            Environment::with_parent(method.closure.clone()),
        ));
        // Bind `this` to the receiver object.
        call_env.borrow_mut().define(
            "this".to_string(), (*receiver).clone(), true, line, column,
        )?;
        for (name, val) in method.params.iter().zip(args.into_iter()) {
            call_env.borrow_mut().define(name.clone(), val, false, line, column)?;
        }
        let previous = std::mem::replace(&mut self.environment, call_env);
        let prev_class = std::mem::replace(&mut self.method_class, method.owner_class.clone());
        let prev_receiver = std::mem::replace(&mut self.method_receiver, Some(receiver.clone()));
        let result   = self.exec_block(&method.body);
        self.method_receiver = prev_receiver;
        self.method_class = prev_class;
        self.environment = previous;
        // The Object's fields are stored behind an Rc<RefCell<...>> so any
        // FieldAssignment inside the method already mutated the shared state
        // in-place.  No flush-back needed.
        match result? {
            Some(Signal::Return(v)) => Ok(v),
            Some(Signal::Throw(val, ln, col)) => Err(CompilerError::Thrown { value: val, line: ln, column: col }),
            _ => Ok(RuntimeValue::Null),
        }
    }

    // ── Struct literal ────────────────────────────────────────────────

    fn eval_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, Expression)],
        _line: usize,
        _column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        let mut field_map: HashMap<String, RuntimeValue> = HashMap::new();
        for (fname, fexpr) in fields {
            let val = self.eval_expr(fexpr)?;
            field_map.insert(fname.clone(), val);
        }
        Ok(RuntimeValue::Struct {
            name: name.to_string(),
            fields: field_map,
        })
    }

    // ── Field access ──────────────────────────────────────────────────

    fn eval_field_access(
        &mut self,
        object: &Expression,
        field: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        let obj_val = self.eval_expr(object)?;
        match &obj_val {
            RuntimeValue::SuperHandle { receiver, parent_class } => {
                let parent = self.lookup_class(parent_class, line, column)?;
                if let Some(method) = parent.methods.get(field) {
                    return Ok(RuntimeValue::BoundMethod {
                        receiver: receiver.clone(),
                        method: method.clone(),
                    });
                }
                return Err(CompilerError::NoSuchField {
                    struct_name: parent_class.clone(),
                    field: field.to_string(),
                    line,
                    column,
                });
            }
            RuntimeValue::Struct { name, fields } => {
                fields.get(field).cloned().ok_or_else(|| CompilerError::NoSuchField {
                    struct_name: name.clone(),
                    field: field.to_string(),
                    line, column,
                })
            }
            RuntimeValue::Object { class_name, fields, methods, field_visibility, .. } => {
                if matches!(field_visibility.get(field), Some(Visibility::Private)) {
                    if self.method_class.as_deref() != Some(class_name.as_str()) {
                        return Err(CompilerError::PrivateFieldAccess {
                            class_name: class_name.clone(),
                            field: field.to_string(),
                            line,
                            column,
                        });
                    }
                }
                if let Some(val) = fields.borrow().get(field).cloned() {
                    return Ok(val);
                }
                if let Some(method) = methods.get(field).cloned() {
                    let bound = RuntimeValue::BoundMethod {
                        receiver: Rc::new(obj_val.clone()),
                        method,
                    };
                    return Ok(bound);
                }
                Err(CompilerError::NoSuchField {
                    struct_name: class_name.clone(),
                    field: field.to_string(),
                    line, column,
                })
            }
            RuntimeValue::Map(map) => {
                map.borrow().get(field).cloned().ok_or_else(|| CompilerError::NoSuchField {
                    struct_name: "Map".to_string(),
                    field: field.to_string(),
                    line, column,
                })
            }
            other => Err(CompilerError::TypeMismatch {
                operation: "field access '.'".to_string(),
                expected:  "Struct, Object, Map, or super".to_string(),
                found:     other.type_name().to_string(),
                line, column,
            }),
        }
    }

    fn lookup_class(
        &self,
        name: &str,
        line: usize,
        column: usize,
    ) -> Result<Rc<BzClass>, CompilerError> {
        match self.environment.borrow().get(name, line, column)? {
            RuntimeValue::Class(class) => Ok(class),
            other => Err(CompilerError::TypeMismatch {
                operation: "class lookup".to_string(),
                expected: "Class".to_string(),
                found: other.type_name().to_string(),
                line,
                column,
            }),
        }
    }
} // impl Interpreter

// ── Phase 4+ helper methods (added inside a second impl block) ────────────

impl<W: std::io::Write> Interpreter<W> {

    // ── Index expression ──────────────────────────────────────────────

    fn eval_index(
        &mut self,
        object: &Expression,
        index: &Expression,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        let obj = self.eval_expr(object)?;
        let idx = self.eval_expr(index)?;
        match obj {
            RuntimeValue::Array(arr) => {
                let i = match idx {
                    RuntimeValue::Integer(n) => n,
                    other => return Err(CompilerError::InvalidIndex {
                        found: other.type_name().to_string(), line, column,
                    }),
                };
                let borrow = arr.borrow();
                let len = borrow.len();
                if i < 0 || i as usize >= len {
                    return Err(CompilerError::IndexOutOfBounds { index: i, length: len, line, column });
                }
                Ok(borrow[i as usize].clone())
            }
            RuntimeValue::Map(map) => {
                let key = match idx {
                    RuntimeValue::String(s) => s,
                    RuntimeValue::Integer(n) => n.to_string(),
                    other => return Err(CompilerError::InvalidIndex {
                        found: other.type_name().to_string(), line, column,
                    }),
                };
                map.borrow().get(&key).cloned().ok_or_else(|| CompilerError::NoSuchField {
                    struct_name: "Map".to_string(),
                    field: key,
                    line, column,
                })
            }
            other => Err(CompilerError::TypeMismatch {
                operation: "index".to_string(),
                expected: "Array or Map".to_string(),
                found: other.type_name().to_string(),
                line, column,
            }),
        }
    }

    // ── Import ────────────────────────────────────────────────────────

    fn exec_import(
        &mut self,
        name: &str,
        path: Option<&str>,
        line: usize,
        column: usize,
    ) -> StmtResult {
        if let Some(module) = build_module(name) {
            self.environment
                .borrow_mut()
                .define(name.to_string(), module, true, line, column)?;
            return Ok(None);
        }

        // File-based import: cwd, then stdlib/
        let candidates: Vec<String> = if let Some(p) = path {
            vec![if p.ends_with(".bz") {
                p.to_string()
            } else {
                format!("{p}.bz")
            }]
        } else {
            vec![format!("{name}.bz"), format!("stdlib/{name}.bz")]
        };

        let source = candidates
            .iter()
            .find_map(|file_path| std::fs::read_to_string(file_path).ok())
            .ok_or(CompilerError::ModuleNotFound {
                name: name.to_string(),
                line,
                column,
            })?;

        let tokens = crate::lexer::tokenize(&source).map_err(|e| e)?;
        let program = crate::parser::parse(tokens).map_err(|e| e)?;
        crate::semantic::analyze(&program).map_err(|e| e)?;

        // Run the module in a child environment, then expose its exports.
        let module_env = Rc::new(RefCell::new(
            Environment::with_parent(self.environment.clone()),
        ));
        let previous = std::mem::replace(&mut self.environment, module_env.clone());
        for stmt in &program.statements {
            self.exec_stmt(stmt)?;
        }
        self.environment = previous;

        // Collect all exported names into a Map and bind as module name.
        let exports: HashMap<String, RuntimeValue> = module_env
            .borrow()
            .exported_names()
            .into_iter()
            .filter_map(|n| {
                module_env.borrow().get_direct(&n).map(|v| (n, v))
            })
            .collect();
        let module_val = RuntimeValue::Map(Rc::new(RefCell::new(exports)));
        self.environment.borrow_mut().define(name.to_string(), module_val, true, line, column)?;
        Ok(None)
    }

    // ── Enum declaration ──────────────────────────────────────────────

    fn exec_enum_declaration(
        &mut self,
        name: &str,
        variants: &[(String, Option<String>)],
        line: usize,
        column: usize,
    ) -> StmtResult {
        // Build a Map of variant_name -> EnumVariant value.
        let mut map: HashMap<String, RuntimeValue> = HashMap::new();
        for (vname, _payload_type) in variants {
            map.insert(vname.clone(), RuntimeValue::EnumVariant {
                enum_name: name.to_string(),
                variant: vname.clone(),
                payload: None,
            });
        }
        let enum_val = RuntimeValue::Map(Rc::new(RefCell::new(map)));
        self.environment.borrow_mut().define(name.to_string(), enum_val, true, line, column)?;
        Ok(None)
    }

    // ── Match statement ───────────────────────────────────────────────

    fn exec_match(
        &mut self,
        subject: &Expression,
        arms: &[crate::ast::MatchArm],
        _line: usize,
        _column: usize,
    ) -> StmtResult {
        let val = self.eval_expr(subject)?;
        for arm in arms {
            if let Some(binding) = match_pattern(&arm.pattern, &val) {
                let child = Rc::new(RefCell::new(
                    Environment::with_parent(self.environment.clone()),
                ));
                for (bname, bval) in binding {
                    child.borrow_mut().define(bname, bval, false, 0, 0)?;
                }
                return self.exec_block_in_env(&arm.body, child);
            }
        }
        Ok(None)
    }

    // ── Spawn ─────────────────────────────────────────────────────────

    fn exec_spawn(
        &mut self,
        expression: &Expression,
        line: usize,
        column: usize,
    ) -> StmtResult {
        // Run inline for now — Rc-based values are not Send across OS threads.
        let callee = self.eval_expr(expression)?;
        match callee {
            RuntimeValue::Function(func) => {
                let _ = self.call_function(func, vec![], line, column)?;
                Ok(None)
            }
            RuntimeValue::BoundMethod { receiver, method } => {
                let _ = self.call_bound_method(receiver, method, vec![], line, column)?;
                Ok(None)
            }
            other => Err(CompilerError::NotCallable {
                found: other.type_name().to_string(),
                line, column,
            }),
        }
    }
}

// ── Free arithmetic helpers ───────────────────────────────────────────────

fn eval_add(
    left: RuntimeValue,
    right: RuntimeValue,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l.wrapping_add(r))),
        (RuntimeValue::Float(l),   RuntimeValue::Float(r))   => Ok(RuntimeValue::Float(l + r)),
        (RuntimeValue::Integer(l), RuntimeValue::Float(r))   => Ok(RuntimeValue::Float(l as f64 + r)),
        (RuntimeValue::Float(l),   RuntimeValue::Integer(r)) => Ok(RuntimeValue::Float(l + r as f64)),
        (RuntimeValue::String(l),  RuntimeValue::String(r))  => Ok(RuntimeValue::String(format!("{l}{r}"))),
        (l, r) => Err(CompilerError::TypeMismatch {
            operation: "addition '+'".to_string(),
            expected:  "numbers or strings".to_string(),
            found:     format!("{} and {}", l.type_name(), r.type_name()),
            line, column,
        }),
    }
}

fn eval_arithmetic(
    left: RuntimeValue,
    right: RuntimeValue,
    op: &str,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(match op {
            "-" => l.wrapping_sub(r),
            "*" => l.wrapping_mul(r),
            _ => unreachable!(),
        })),
        (RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(match op {
            "-" => l - r,
            "*" => l * r,
            _ => unreachable!(),
        })),
        (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(match op {
            "-" => l as f64 - r,
            "*" => l as f64 * r,
            _ => unreachable!(),
        })),
        (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Float(match op {
            "-" => l - r as f64,
            "*" => l * r as f64,
            _ => unreachable!(),
        })),
        (l, r) => Err(CompilerError::TypeMismatch {
            operation: format!("arithmetic '{op}'"),
            expected:  "numbers (Integer or Float)".to_string(),
            found:     format!("{} and {}", l.type_name(), r.type_name()),
            line, column,
        }),
    }
}

fn eval_division(
    left: RuntimeValue,
    right: RuntimeValue,
    op: &str,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
            if r == 0 { return Err(CompilerError::DivisionByZero { line, column }); }
            Ok(RuntimeValue::Integer(match op { "/" => l.wrapping_div(r), _ => l.wrapping_rem(r) }))
        }
        (RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
            if r == 0.0 { return Err(CompilerError::DivisionByZero { line, column }); }
            Ok(RuntimeValue::Float(match op { "/" => l / r, _ => l % r }))
        }
        (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => {
            if r == 0.0 { return Err(CompilerError::DivisionByZero { line, column }); }
            let lf = l as f64;
            Ok(RuntimeValue::Float(match op { "/" => lf / r, _ => lf % r }))
        }
        (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => {
            if r == 0 { return Err(CompilerError::DivisionByZero { line, column }); }
            let rf = r as f64;
            Ok(RuntimeValue::Float(match op { "/" => l / rf, _ => l % rf }))
        }
        (l, r) => Err(CompilerError::TypeMismatch {
            operation: format!("'{op}'"),
            expected:  "numbers".to_string(),
            found:     format!("{} and {}", l.type_name(), r.type_name()),
            line, column,
        }),
    }
}

fn eval_equality(left: &RuntimeValue, right: &RuntimeValue) -> bool {
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => l == r,
        (RuntimeValue::Float(l),   RuntimeValue::Float(r))   => l == r,
        (RuntimeValue::Integer(l), RuntimeValue::Float(r))   => (*l as f64) == *r,
        (RuntimeValue::Float(l),   RuntimeValue::Integer(r)) => *l == (*r as f64),
        (RuntimeValue::String(l),  RuntimeValue::String(r))  => l == r,
        (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => l == r,
        (RuntimeValue::Null,       RuntimeValue::Null)        => true,
        _ => false,
    }
}

fn eval_comparison(
    left: RuntimeValue,
    right: RuntimeValue,
    op: &str,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    macro_rules! cmp {
        ($l:expr, $r:expr) => {
            Ok(RuntimeValue::Boolean(match op {
                "<"  => $l < $r,
                ">"  => $l > $r,
                "<=" => $l <= $r,
                ">=" => $l >= $r,
                _    => unreachable!(),
            }))
        };
    }
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => cmp!(l, r),
        (RuntimeValue::Float(l),   RuntimeValue::Float(r))   => cmp!(l, r),
        (RuntimeValue::Integer(l), RuntimeValue::Float(r))   => cmp!(l as f64, r),
        (RuntimeValue::Float(l),   RuntimeValue::Integer(r)) => cmp!(l, r as f64),
        (RuntimeValue::String(l),  RuntimeValue::String(r))  => cmp!(l, r),
        (l, r) => Err(CompilerError::TypeMismatch {
            operation: format!("comparison '{op}'"),
            expected:  "numbers or strings".to_string(),
            found:     format!("{} and {}", l.type_name(), r.type_name()),
            line, column,
        }),
    }
}

// ── Truthiness ────────────────────────────────────────────────────────────

fn is_truthy(val: &RuntimeValue) -> bool {
    match val {
        RuntimeValue::Boolean(b) => *b,
        RuntimeValue::Null       => false,
        RuntimeValue::Integer(n) => *n != 0,
        RuntimeValue::Float(f)   => *f != 0.0,
        _                        => true,
    }
}

fn require_bool(val: RuntimeValue, op: &str, line: usize, column: usize) -> Result<bool, CompilerError> {
    match val {
        RuntimeValue::Boolean(b) => Ok(b),
        other => Err(CompilerError::TypeMismatch {
            operation: op.to_string(),
            expected:  "Boolean".to_string(),
            found:     other.type_name().to_string(),
            line, column,
        }),
    }
}


// ── Pattern matching helper ───────────────────────────────────────────────

/// Try to match `val` against `pattern`. Returns `Some(bindings)` on match,
/// where `bindings` is a list of (name, value) pairs to bind in the arm scope.
fn match_pattern(
    pattern: &crate::ast::MatchPattern,
    val: &RuntimeValue,
) -> Option<Vec<(String, RuntimeValue)>> {
    use crate::ast::MatchPattern;
    match pattern {
        MatchPattern::Wildcard => Some(vec![]),
        MatchPattern::Integer(n) => {
            if let RuntimeValue::Integer(v) = val { if v == n { return Some(vec![]); } }
            None
        }
        MatchPattern::Float(n) => {
            if let RuntimeValue::Float(v) = val { if v == n { return Some(vec![]); } }
            None
        }
        MatchPattern::StringLit(s) => {
            if let RuntimeValue::String(v) = val { if v == s { return Some(vec![]); } }
            None
        }
        MatchPattern::Boolean(b) => {
            if let RuntimeValue::Boolean(v) = val { if v == b { return Some(vec![]); } }
            None
        }
        MatchPattern::Null => {
            if matches!(val, RuntimeValue::Null) { Some(vec![]) } else { None }
        }
        MatchPattern::Identifier(name) => {
            if name == "_" { return Some(vec![]); }
            Some(vec![(name.clone(), val.clone())])
        }
        MatchPattern::EnumVariant(variant_name, sub_pattern) => {
            if let RuntimeValue::EnumVariant { variant, payload, .. } = val {
                if variant == variant_name {
                    match (sub_pattern, payload) {
                        (None, _) => return Some(vec![]),
                        (Some(sub), Some(p)) => {
                            return match_pattern(sub, p);
                        }
                        _ => return None,
                    }
                }
            }
            None
        }
    }
}
