//! Static semantic analysis for Bunzo programs.
//!
//! This module checks whether the parsed Abstract Syntax Tree follows
//! the semantic rules of the language, such as validating variable references
//! and detecting duplicate variable declarations in the same scope.
//!
//! Phase 1 (functions) extends this with: function symbol tracking (so
//! calls can be arity-checked at compile time), a nested scope per
//! function body containing its parameters, and validation that `return`
//! only appears inside a function.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Expression, Program, Statement};
use crate::diagnostics::CompilerError;

/// A symbol defined in a lexical scope at compile-time.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The name of the symbol.
    pub name: String,
    /// Whether the symbol is immutable (`const` vs `let`).
    pub is_const: bool,
    /// Whether this symbol refers to a function declaration.
    ///
    /// Function symbols are always implicitly immutable (`is_const` is
    /// `true` for them), and carry a known parameter count so that direct
    /// calls can be arity-checked without a full type system.
    pub is_function: bool,
    /// The number of parameters the function declares.
    ///
    /// Only meaningful when `is_function` is `true`; otherwise `0`.
    pub param_count: usize,
    /// The 1-based line number of its declaration.
    pub line: usize,
    /// The 1-based column number of its declaration.
    pub column: usize,
}

/// A scope at compile-time holding defined symbols.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    /// The outer parent scope, if any.
    parent: Option<Rc<RefCell<Scope>>>,
    /// Symbols declared immediately in this scope.
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    /// Creates a new, parentless global scope.
    pub fn new() -> Self {
        Self {
            parent: None,
            symbols: HashMap::new(),
        }
    }

    /// Creates a new scope nested inside the given parent scope.
    pub fn with_parent(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            parent: Some(parent),
            symbols: HashMap::new(),
        }
    }

    /// Defines a variable symbol (`let`/`const`/parameter) in the
    /// immediate scope.
    ///
    /// # Errors
    ///
    /// Returns a [`CompilerError::DuplicateDeclaration`] if the symbol is already defined
    /// in this immediate scope.
    pub fn define(
        &mut self,
        name: String,
        is_const: bool,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        self.define_symbol(Symbol {
            name,
            is_const,
            is_function: false,
            param_count: 0,
            line,
            column,
        })
    }

    /// Defines a function symbol in the immediate scope.
    ///
    /// Function symbols are always immutable and carry their parameter
    /// count, enabling arity checks at call sites.
    ///
    /// # Errors
    ///
    /// Returns a [`CompilerError::DuplicateDeclaration`] if the name is already defined
    /// in this immediate scope.
    pub fn define_function(
        &mut self,
        name: String,
        param_count: usize,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        self.define_symbol(Symbol {
            name,
            is_const: true,
            is_function: true,
            param_count,
            line,
            column,
        })
    }

    /// Shared insertion logic for [`Scope::define`] and [`Scope::define_function`].
    fn define_symbol(&mut self, symbol: Symbol) -> Result<(), CompilerError> {
        if self.symbols.contains_key(&symbol.name) {
            return Err(CompilerError::DuplicateDeclaration {
                name: symbol.name,
                line: symbol.line,
                column: symbol.column,
            });
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Looks up a symbol, traversing parent scopes if necessary.
    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().lookup(name);
        }
        None
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

/// Statically analyzes a Bunzo program.
///
/// # Errors
///
/// Returns the first [`CompilerError`] encountered (e.g. undefined variable or duplicate declaration).
pub fn analyze(program: &Program) -> Result<(), CompilerError> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(program)
}

struct ClassInfo {
    fields: Vec<String>,
    methods: HashMap<String, usize>,
}

struct SemanticAnalyzer {
    current_scope: Rc<RefCell<Scope>>,
    /// `true` while analyzing the body of a function declaration.
    ///
    /// Used to reject `return` statements that appear at the top level
    /// or otherwise outside of any function.
    in_function: bool,
    /// Depth of nested loops currently being analyzed.
    ///
    /// Used to reject `break`/`continue` statements that appear outside
    /// of any loop.
    loop_depth: usize,
    /// Declared struct types, mapping struct name to its declared field
    /// names in declaration order.
    ///
    /// Structs share a single flat (global) namespace, separate from the
    /// variable scope chain — there is no nested struct declaration in
    /// Bunzo yet.
    struct_defs: HashMap<String, Vec<String>>,
    /// Declared class types, mapping class name to its details.
    class_defs: HashMap<String, ClassInfo>,
}

impl SemanticAnalyzer {
    fn new() -> Self {
        let global_scope = Rc::new(RefCell::new(Scope::new()));
        {
            let mut mut_scope = global_scope.borrow_mut();
            let _ = mut_scope.define_function("len".to_string(), 1, 1, 1);
            let _ = mut_scope.define_function("type".to_string(), 1, 1, 1);
            let _ = mut_scope.define_function("str".to_string(), 1, 1, 1);
            let _ = mut_scope.define_function("to_int".to_string(), 1, 1, 1);
            let _ = mut_scope.define_function("to_float".to_string(), 1, 1, 1);
            // `input` accepts 0 or 1 args; register with 0 so arity check is skipped.
            // We handle it as a special case in the runtime.
            let _ = mut_scope.define_function("input".to_string(), 0, 1, 1);
        }
        Self {
            current_scope: global_scope,
            in_function: false,
            loop_depth: 0,
            struct_defs: HashMap::new(),
            class_defs: HashMap::new(),
        }
    }

    fn analyze_program(&mut self, program: &Program) -> Result<(), CompilerError> {
        self.analyze_block(&program.statements)
    }

    /// Analyzes a sequence of statements in the current scope, in order.
    fn analyze_block(&mut self, statements: &[Statement]) -> Result<(), CompilerError> {
        for stmt in statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        match stmt {
            Statement::LetDeclaration { name, initializer, line, column } => {
                self.analyze_expression(initializer)?;
                self.current_scope.borrow_mut().define(
                    name.clone(),
                    false, // is_const = false
                    *line,
                    *column,
                )?;
            }
            Statement::ConstDeclaration { name, initializer, line, column } => {
                self.analyze_expression(initializer)?;
                self.current_scope.borrow_mut().define(
                    name.clone(),
                    true, // is_const = true
                    *line,
                    *column,
                )?;
            }
            Statement::PrintStatement { argument, .. } => {
                self.analyze_expression(argument)?;
            }
            Statement::ExpressionStatement { expression } => {
                self.analyze_expression(expression)?;
            }
            Statement::FunctionDeclaration { name, params, body, line, column, .. } => {
                // Define the function in the *enclosing* scope first, so
                // that recursive calls inside its own body resolve.
                self.current_scope.borrow_mut().define_function(
                    name.clone(),
                    params.len(),
                    *line,
                    *column,
                )?;

                // Analyze the body in a fresh scope nested under the
                // current one, seeded with the function's parameters.
                let function_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                for param in params {
                    function_scope.borrow_mut().define(
                        param.name.clone(),
                        false, // parameters are mutable, like `let`
                        param.line,
                        param.column,
                    )?;
                }

                let previous_scope = std::mem::replace(&mut self.current_scope, function_scope);
                let was_in_function = self.in_function;
                self.in_function = true;

                let result = self.analyze_block(body);

                self.in_function = was_in_function;
                self.current_scope = previous_scope;

                result?;
            }
            Statement::ReturnStatement { value, line, column } => {
                if !self.in_function {
                    return Err(CompilerError::ReturnOutsideFunction {
                        line: *line,
                        column: *column,
                    });
                }
                if let Some(expr) = value {
                    self.analyze_expression(expr)?;
                }
            }
            Statement::Assignment { name, value, line, column } => {
                self.analyze_expression(value)?;
                let symbol = self.current_scope.borrow().lookup(name).ok_or_else(|| {
                    CompilerError::UndefinedVariable {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    }
                })?;
                if symbol.is_const {
                    return Err(CompilerError::ConstReassignment {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
            }
            Statement::IfStatement { condition, then_branch, else_branch, .. } => {
                self.analyze_expression(condition)?;

                let then_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, then_scope);
                let result = self.analyze_block(then_branch);
                self.current_scope = previous_scope;
                result?;

                if let Some(else_stmts) = else_branch {
                    let else_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                    let previous_scope = std::mem::replace(&mut self.current_scope, else_scope);
                    let result = self.analyze_block(else_stmts);
                    self.current_scope = previous_scope;
                    result?;
                }
            }
            Statement::WhileStatement { condition, body, .. } => {
                self.analyze_expression(condition)?;

                let loop_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, loop_scope);
                self.loop_depth += 1;
                let result = self.analyze_block(body);
                self.loop_depth -= 1;
                self.current_scope = previous_scope;
                result?;
            }
            Statement::ForStatement { variable, start, end, body, line, column } => {
                self.analyze_expression(start)?;
                self.analyze_expression(end)?;

                let loop_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                loop_scope.borrow_mut().define(variable.clone(), false, *line, *column)?;
                let previous_scope = std::mem::replace(&mut self.current_scope, loop_scope);
                self.loop_depth += 1;
                let result = self.analyze_block(body);
                self.loop_depth -= 1;
                self.current_scope = previous_scope;
                result?;
            }
            Statement::BreakStatement { line, column } => {
                if self.loop_depth == 0 {
                    return Err(CompilerError::BreakOutsideLoop {
                        line: *line,
                        column: *column,
                    });
                }
            }
            Statement::ContinueStatement { line, column } => {
                if self.loop_depth == 0 {
                    return Err(CompilerError::ContinueOutsideLoop {
                        line: *line,
                        column: *column,
                    });
                }
            }
            Statement::StructDeclaration { name, fields, line, column } => {
                if self.struct_defs.contains_key(name) || self.class_defs.contains_key(name) {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }

                let mut seen = std::collections::HashSet::new();
                for field in fields {
                    if !seen.insert(&field.name) {
                        return Err(CompilerError::DuplicateDeclaration {
                            name: field.name.clone(),
                            line: field.line,
                            column: field.column,
                        });
                    }
                }

                let field_names = fields.iter().map(|f| f.name.clone()).collect();
                self.struct_defs.insert(name.clone(), field_names);
            }
            Statement::ClassDeclaration { name, fields, methods, line, column, .. } => {
                if self.struct_defs.contains_key(name) || self.class_defs.contains_key(name) {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }

                let mut seen = std::collections::HashSet::new();
                for field in fields {
                    if !seen.insert(&field.name) {
                        return Err(CompilerError::DuplicateDeclaration {
                            name: field.name.clone(),
                            line: field.line,
                            column: field.column,
                        });
                    }
                }

                // Define class constructor in the enclosing scope
                self.current_scope.borrow_mut().define_function(
                    name.clone(),
                    0,
                    *line,
                    *column,
                )?;

                let mut method_arities = HashMap::new();
                for method_stmt in methods {
                    if let Statement::FunctionDeclaration { name: method_name, params, .. } = method_stmt {
                        method_arities.insert(method_name.clone(), params.len());
                    }
                }

                self.class_defs.insert(name.clone(), ClassInfo {
                    fields: fields.iter().map(|f| f.name.clone()).collect(),
                    methods: method_arities,
                });

                for method_stmt in methods {
                    if let Statement::FunctionDeclaration { params, body, line: m_line, column: m_col, .. } = method_stmt {
                        let method_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                        method_scope.borrow_mut().define("this".to_string(), true, *m_line, *m_col)?;
                        for param in params {
                            method_scope.borrow_mut().define(
                                param.name.clone(),
                                false,
                                param.line,
                                param.column,
                            )?;
                        }

                        let previous_scope = std::mem::replace(&mut self.current_scope, method_scope);
                        let was_in_function = self.in_function;
                        self.in_function = true;

                        let result = self.analyze_block(body);

                        self.in_function = was_in_function;
                        self.current_scope = previous_scope;
                        result?;
                    }
                }
            }
            Statement::FieldAssignment { object, value, .. } => {
                self.analyze_expression(object)?;
                self.analyze_expression(value)?;
            }
            Statement::TryCatch { try_block, catch_var, catch_block, line, column } => {
                let try_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, try_scope);
                let try_result = self.analyze_block(try_block);
                self.current_scope = previous_scope;
                try_result?;

                let catch_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                catch_scope.borrow_mut().define(catch_var.clone(), false, *line, *column)?;
                let previous_scope = std::mem::replace(&mut self.current_scope, catch_scope);
                let catch_result = self.analyze_block(catch_block);
                self.current_scope = previous_scope;
                catch_result?;
            }
            Statement::Throw { value, .. } => {
                self.analyze_expression(value)?;
            }
            // ── Phase 4+ statements ───────────────────────────────────
            Statement::ImportDeclaration { .. } => {
                // Module resolution happens at runtime; semantic pass accepts.
                Ok(())
            }
            Statement::ExportDeclaration { .. } => Ok(()),
            Statement::EnumDeclaration { name, line, column, .. } => {
                if self.struct_defs.contains_key(name) || self.class_defs.contains_key(name) {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                // Register enum as a "class" with no fields so constructors resolve.
                self.class_defs.insert(name.clone(), ClassInfo {
                    fields: vec![],
                    methods: HashMap::new(),
                });
                Ok(())
            }
            Statement::MatchStatement { subject, arms, .. } => {
                self.analyze_expression(subject)?;
                for arm in arms {
                    let arm_scope = Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                    // Wildcard / identifier patterns bind a variable in the arm scope.
                    if let crate::ast::MatchPattern::Identifier(binding) = &arm.pattern {
                        if binding != "_" {
                            arm_scope.borrow_mut().define(binding.clone(), false, 0, 0)?;
                        }
                    }
                    let prev = std::mem::replace(&mut self.current_scope, arm_scope);
                    let result = self.analyze_block(&arm.body);
                    self.current_scope = prev;
                    result?;
                }
                Ok(())
            }
            Statement::InterfaceDeclaration { .. } => Ok(()), // structural typing — no deep check
            Statement::SpawnStatement { expression, .. } => {
                self.analyze_expression(expression)
            }
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<(), CompilerError> {
        match expr {
            Expression::IntegerLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BooleanLiteral { .. }
            | Expression::NullLiteral { .. } => Ok(()),

            Expression::Identifier { name, line, column } => {
                if self.current_scope.borrow().lookup(name).is_none() {
                    return Err(CompilerError::UndefinedVariable {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                Ok(())
            }

            Expression::Grouping { expression, .. } => {
                self.analyze_expression(expression)
            }

            Expression::UnaryOp { operand, .. } => {
                self.analyze_expression(operand)
            }

            Expression::BinaryOp { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)
            }

            Expression::Call { callee, arguments, line, column } => {
                for argument in arguments {
                    self.analyze_expression(argument)?;
                }

                // Direct calls by name (the common case, e.g. `add(1, 2)`)
                // get a compile-time arity check. Indirect callees (e.g. a
                // call on the result of another expression) fall back to
                // a plain analysis pass, since we don't yet have a type
                // system to know whether they hold a function value.
                if let Expression::Identifier { name, line: id_line, column: id_col } = callee.as_ref() {
                    let symbol = self.current_scope.borrow().lookup(name).ok_or_else(|| {
                        CompilerError::UndefinedVariable {
                            name: name.clone(),
                            line: *id_line,
                            column: *id_col,
                        }
                    })?;

                    if !symbol.is_function {
                        return Err(CompilerError::NotCallable {
                            found: format!("variable \"{name}\""),
                            line: *line,
                            column: *column,
                        });
                    }

                    if symbol.param_count != arguments.len() {
                        return Err(CompilerError::ArityMismatch {
                            name: name.clone(),
                            expected: symbol.param_count,
                            found: arguments.len(),
                            line: *line,
                            column: *column,
                        });
                    }

                    Ok(())
                } else {
                    self.analyze_expression(callee)
                }
            }

            Expression::StructLiteral { name, fields, line, column } => {
                let declared_fields = self.struct_defs.get(name).cloned().ok_or_else(|| {
                    CompilerError::UnknownStruct {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    }
                })?;

                for (_, value) in fields {
                    self.analyze_expression(value)?;
                }

                let mut seen = std::collections::HashSet::new();
                for (field_name, _) in fields {
                    if !seen.insert(field_name.as_str()) {
                        return Err(CompilerError::DuplicateDeclaration {
                            name: field_name.clone(),
                            line: *line,
                            column: *column,
                        });
                    }
                }

                let provided: std::collections::HashSet<&str> =
                    fields.iter().map(|(n, _)| n.as_str()).collect();
                let declared: std::collections::HashSet<&str> =
                    declared_fields.iter().map(|n| n.as_str()).collect();

                let missing: Vec<String> = declared
                    .difference(&provided)
                    .map(|s| s.to_string())
                    .collect();
                let unexpected: Vec<String> = provided
                    .difference(&declared)
                    .map(|s| s.to_string())
                    .collect();

                if !missing.is_empty() || !unexpected.is_empty() {
                    return Err(CompilerError::StructFieldMismatch {
                        struct_name: name.clone(),
                        missing,
                        unexpected,
                        line: *line,
                        column: *column,
                    });
                }

                Ok(())
            }

            Expression::FieldAccess { object, .. } => {
                // Without a full type system (Phase 4), we can't always
                // know statically which struct type `object` evaluates
                // to, so field existence is validated at runtime. We
                // still recurse to catch undefined variables etc. in the
                // object expression itself.
                self.analyze_expression(object)
            }
            // ── Phase 4+ expressions ──────────────────────────────────
            Expression::ArrayLiteral { elements, .. } => {
                for el in elements { self.analyze_expression(el)?; }
                Ok(())
            }
            Expression::IndexExpression { object, index, .. } => {
                self.analyze_expression(object)?;
                self.analyze_expression(index)
            }
            Expression::EnumVariantExpr { payload, .. } => {
                if let Some(p) = payload { self.analyze_expression(p)?; }
                Ok(())
            }
            Expression::PropagateError { expression, .. } => {
                self.analyze_expression(expression)
            }
            Expression::MoveExpr { name, line, column } => {
                if self.current_scope.borrow().lookup(name).is_none() {
                    return Err(CompilerError::UndefinedVariable {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                Ok(())
            }
            Expression::AwaitExpr { expression, .. } => {
                self.analyze_expression(expression)
            }
        }
    }
}
