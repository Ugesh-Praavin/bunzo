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
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{Expression, Program, Statement, Visibility};
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

struct MethodInfo {
    arity: usize,
    is_abstract: bool,
}

struct ClassInfo {
    is_abstract: bool,
    extends: Option<String>,
    implements: Vec<String>,
    fields: HashMap<String, Visibility>,
    methods: HashMap<String, MethodInfo>,
}

struct InterfaceInfo {
    methods: HashMap<String, usize>,
}

struct SemanticAnalyzer {
    current_scope: Rc<RefCell<Scope>>,
    in_function: bool,
    loop_depth: usize,
    /// Class whose method body is currently being analyzed (`this` / `super` context).
    current_class: Option<String>,
    struct_defs: HashMap<String, Vec<String>>,
    class_defs: HashMap<String, ClassInfo>,
    interface_defs: HashMap<String, InterfaceInfo>,
    analyzed_modules: HashMap<String, HashSet<String>>,
    active_imports: HashSet<String>,
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
        let mut analyzer = Self {
            current_scope: global_scope,
            in_function: false,
            loop_depth: 0,
            current_class: None,
            struct_defs: HashMap::new(),
            class_defs: HashMap::new(),
            interface_defs: HashMap::new(),
            analyzed_modules: HashMap::new(),
            active_imports: HashSet::new(),
        };
        // Pre-populate standard library module exports
        analyzer.analyzed_modules.insert(
            "math".to_string(),
            vec!["sqrt", "abs", "sin", "cos", "pow", "PI", "E"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "json".to_string(),
            vec!["encode", "decode"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "http".to_string(),
            vec!["get", "post", "put", "delete", "patch"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "db".to_string(),
            vec!["open", "execute", "query", "close"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "os".to_string(),
            vec!["args", "env", "exit"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "vector".to_string(),
            vec![
                "new",
                "with_capacity",
                "push",
                "pop",
                "get",
                "set",
                "insert",
                "remove",
                "front",
                "back",
                "len",
                "capacity",
                "is_empty",
                "clear",
                "contains",
                "index_of",
                "reverse",
                "sort",
                "resize",
                "swap",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "deque".to_string(),
            vec![
                "new",
                "push_front",
                "push_back",
                "pop_front",
                "pop_back",
                "front",
                "back",
                "get",
                "set",
                "len",
                "is_empty",
                "clear",
                "contains",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "stack".to_string(),
            vec![
                "new", "push", "pop", "top", "len", "is_empty", "clear", "contains", "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "queue".to_string(),
            vec![
                "new", "push", "pop", "front", "back", "len", "is_empty", "clear", "contains",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "priority_queue".to_string(),
            vec![
                "new", "push", "pop", "top", "len", "is_empty", "clear", "contains", "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "set".to_string(),
            vec![
                "new",
                "insert",
                "remove",
                "contains",
                "len",
                "is_empty",
                "clear",
                "first",
                "last",
                "min",
                "max",
                "lower_bound",
                "upper_bound",
                "union",
                "intersection",
                "difference",
                "symmetric_difference",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "hashset".to_string(),
            vec![
                "new",
                "insert",
                "remove",
                "contains",
                "len",
                "is_empty",
                "clear",
                "union",
                "intersection",
                "difference",
                "symmetric_difference",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "map".to_string(),
            vec![
                "new",
                "insert",
                "get",
                "remove",
                "contains",
                "len",
                "is_empty",
                "clear",
                "keys",
                "values",
                "first_key",
                "last_key",
                "first_value",
                "last_value",
                "lower_bound",
                "upper_bound",
                "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "hashmap".to_string(),
            vec![
                "new", "insert", "get", "remove", "contains", "len", "is_empty", "clear", "keys",
                "values", "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "bitset".to_string(),
            vec![
                "new", "set", "reset", "flip", "test", "count", "any", "none", "all", "len",
                "clear", "and", "or", "xor", "not", "iter",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "string".to_string(),
            vec!["len", "split", "join", "trim", "replace", "substring"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "filesystem".to_string(),
            vec!["read", "write", "exists", "mkdir", "remove", "listdir"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "path".to_string(),
            vec!["join", "basename", "dirname", "extension"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "time".to_string(),
            vec!["now", "sleep", "timestamp"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "random".to_string(),
            vec!["int", "float", "bool"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "crypto".to_string(),
            vec!["sha256", "uuid"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "encoding".to_string(),
            vec!["hex_encode", "hex_decode"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "process".to_string(),
            vec!["exec", "pid"].into_iter().map(String::from).collect(),
        );
        analyzer.analyzed_modules.insert(
            "io".to_string(),
            vec!["read_line", "read_char"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "environment".to_string(),
            vec!["get", "set", "has", "remove", "all"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "networking".to_string(),
            vec![
                "tcp_listen",
                "tcp_accept",
                "tcp_connect",
                "tcp_send",
                "tcp_recv",
                "udp_bind",
                "udp_send",
                "udp_recv",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "thread".to_string(),
            vec!["spawn", "join"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "mutex".to_string(),
            vec!["new", "lock", "unlock"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "rwlock".to_string(),
            vec!["new", "read", "write"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "channel".to_string(),
            vec!["new", "send", "recv"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "atomic".to_string(),
            vec!["new", "load", "store", "add"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        analyzer.analyzed_modules.insert(
            "regex".to_string(),
            vec![
                "match", "search", "find", "find_all", "replace", "split", "is_match",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "algorithm".to_string(),
            vec![
                "sort",
                "stable_sort",
                "reverse",
                "shuffle",
                "find",
                "find_if",
                "binary_search",
                "lower_bound",
                "upper_bound",
                "min",
                "max",
                "min_element",
                "max_element",
                "copy",
                "fill",
                "rotate",
                "swap",
                "unique",
                "count",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "numeric".to_string(),
            vec![
                "min",
                "max",
                "clamp",
                "abs",
                "gcd",
                "lcm",
                "factorial",
                "average",
                "sum",
                "product",
                "accumulate",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer.analyzed_modules.insert(
            "test".to_string(),
            vec![
                "assert",
                "assert_eq",
                "assert_ne",
                "assert_true",
                "assert_false",
                "fail",
                "skip",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
        analyzer
    }

    fn abstract_methods_inherited(&self, parent: &str) -> HashSet<String> {
        let mut pending = HashSet::new();
        let mut current = Some(parent.to_string());
        while let Some(name) = current {
            let Some(info) = self.class_defs.get(&name) else {
                break;
            };
            for (method_name, method_info) in &info.methods {
                if method_info.is_abstract {
                    pending.insert(method_name.clone());
                } else {
                    pending.remove(method_name);
                }
            }
            current = info.extends.clone();
        }
        pending
    }

    fn check_field_visibility(
        &self,
        class_name: &str,
        field: &str,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        let Some(info) = self.class_defs.get(class_name) else {
            return Ok(());
        };
        if matches!(info.fields.get(field), Some(Visibility::Private)) {
            if self.current_class.as_deref() != Some(class_name) {
                return Err(CompilerError::PrivateFieldAccess {
                    class_name: class_name.to_string(),
                    field: field.to_string(),
                    line,
                    column,
                });
            }
        }
        Ok(())
    }

    fn check_object_field_access(
        &self,
        object: &Expression,
        field: &str,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        if let Expression::Identifier { name, .. } = object {
            if name == "this" {
                if let Some(class_name) = &self.current_class {
                    self.check_field_visibility(class_name, field, line, column)?;
                }
            } else if self.analyzed_modules.contains_key(name) {
                let exports = self.analyzed_modules.get(name).unwrap();
                if !exports.contains(field) {
                    return Err(CompilerError::UnexportedMemberAccess {
                        module_name: name.clone(),
                        member: field.to_string(),
                        line,
                        column,
                    });
                }
            }
        }
        Ok(())
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
            Statement::LetDeclaration {
                name,
                initializer,
                line,
                column,
            } => {
                self.analyze_expression(initializer)?;
                self.current_scope.borrow_mut().define(
                    name.clone(),
                    false, // is_const = false
                    *line,
                    *column,
                )?;
            }
            Statement::ConstDeclaration {
                name,
                initializer,
                line,
                column,
            } => {
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
            Statement::FunctionDeclaration {
                name,
                params,
                body,
                line,
                column,
                ..
            } => {
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
                let function_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
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
            Statement::ReturnStatement {
                value,
                line,
                column,
            } => {
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
            Statement::Assignment {
                name,
                value,
                line,
                column,
            } => {
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
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.analyze_expression(condition)?;

                let then_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, then_scope);
                let result = self.analyze_block(then_branch);
                self.current_scope = previous_scope;
                result?;

                if let Some(else_stmts) = else_branch {
                    let else_scope =
                        Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                    let previous_scope = std::mem::replace(&mut self.current_scope, else_scope);
                    let result = self.analyze_block(else_stmts);
                    self.current_scope = previous_scope;
                    result?;
                }
            }
            Statement::WhileStatement {
                condition, body, ..
            } => {
                self.analyze_expression(condition)?;

                let loop_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, loop_scope);
                self.loop_depth += 1;
                let result = self.analyze_block(body);
                self.loop_depth -= 1;
                self.current_scope = previous_scope;
                result?;
            }
            Statement::ForStatement {
                variable,
                start,
                end,
                body,
                line,
                column,
            } => {
                self.analyze_expression(start)?;
                self.analyze_expression(end)?;

                let loop_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                loop_scope
                    .borrow_mut()
                    .define(variable.clone(), false, *line, *column)?;
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
            Statement::StructDeclaration {
                name,
                fields,
                line,
                column,
            } => {
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
            Statement::ClassDeclaration {
                name,
                extends,
                implements,
                is_abstract,
                fields,
                methods,
                line,
                column,
            } => {
                if self.struct_defs.contains_key(name) || self.class_defs.contains_key(name) {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }

                if let Some(parent) = extends {
                    if !self.class_defs.contains_key(parent) {
                        return Err(CompilerError::UnknownParentClass {
                            name: parent.clone(),
                            line: *line,
                            column: *column,
                        });
                    }
                }

                let mut seen_fields = HashSet::new();
                let mut field_map = HashMap::new();
                for field in fields {
                    if !seen_fields.insert(&field.name) {
                        return Err(CompilerError::DuplicateDeclaration {
                            name: field.name.clone(),
                            line: field.line,
                            column: field.column,
                        });
                    }
                    field_map.insert(field.name.clone(), field.visibility);
                }

                let mut method_map = HashMap::new();
                for method_stmt in methods {
                    if let Statement::FunctionDeclaration {
                        name: method_name,
                        params,
                        is_abstract: method_abstract,
                        ..
                    } = method_stmt
                    {
                        method_map.insert(
                            method_name.clone(),
                            MethodInfo {
                                arity: params.len(),
                                is_abstract: *method_abstract,
                            },
                        );
                    }
                }

                let mut pending_abstract = extends
                    .as_ref()
                    .map(|p| self.abstract_methods_inherited(p))
                    .unwrap_or_default();
                for (method_name, method_info) in &method_map {
                    if method_info.is_abstract {
                        pending_abstract.insert(method_name.clone());
                    } else {
                        pending_abstract.remove(method_name);
                    }
                }
                if !*is_abstract && !pending_abstract.is_empty() {
                    let missing = pending_abstract.into_iter().next().unwrap();
                    return Err(CompilerError::AbstractMethodNotImplemented {
                        class_name: name.clone(),
                        method_name: missing,
                        line: *line,
                        column: *column,
                    });
                }

                let mut all_interfaces = implements.clone();
                if let Some(parent) = extends {
                    if let Some(parent_info) = self.class_defs.get(parent) {
                        for iface in &parent_info.implements {
                            if !all_interfaces.contains(iface) {
                                all_interfaces.push(iface.clone());
                            }
                        }
                    }
                }

                for iface in &all_interfaces {
                    let Some(iface_info) = self.interface_defs.get(iface) else {
                        return Err(CompilerError::ModuleNotFound {
                            name: iface.clone(),
                            line: *line,
                            column: *column,
                        });
                    };
                    for (method_name, arity) in &iface_info.methods {
                        match method_map.get(method_name) {
                            Some(m) if m.arity == *arity && (!m.is_abstract || *is_abstract) => {}
                            _ => {
                                return Err(CompilerError::InterfaceNotImplemented {
                                    class_name: name.clone(),
                                    interface_name: iface.clone(),
                                    method_name: method_name.clone(),
                                    line: *line,
                                    column: *column,
                                });
                            }
                        }
                    }
                }

                let mut init_arity = 0;
                if let Some(m) = method_map.get("init").filter(|m| !m.is_abstract) {
                    init_arity = m.arity;
                } else if let Some(parent) = extends {
                    let mut curr = Some(parent.clone());
                    while let Some(ref p_name) = curr {
                        if let Some(p_info) = self.class_defs.get(p_name) {
                            if let Some(m) = p_info.methods.get("init").filter(|m| !m.is_abstract) {
                                init_arity = m.arity;
                                break;
                            }
                            curr = p_info.extends.clone();
                        } else {
                            break;
                        }
                    }
                }

                self.current_scope.borrow_mut().define_function(
                    name.clone(),
                    init_arity,
                    *line,
                    *column,
                )?;

                self.class_defs.insert(
                    name.clone(),
                    ClassInfo {
                        is_abstract: *is_abstract,
                        extends: extends.clone(),
                        implements: implements.clone(),
                        fields: field_map,
                        methods: method_map,
                    },
                );

                for method_stmt in methods {
                    if let Statement::FunctionDeclaration {
                        params,
                        body,
                        is_abstract: method_abstract,
                        line: m_line,
                        column: m_col,
                        ..
                    } = method_stmt
                    {
                        if *method_abstract {
                            continue;
                        }
                        let method_scope =
                            Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                        method_scope.borrow_mut().define(
                            "this".to_string(),
                            true,
                            *m_line,
                            *m_col,
                        )?;
                        for param in params {
                            method_scope.borrow_mut().define(
                                param.name.clone(),
                                false,
                                param.line,
                                param.column,
                            )?;
                        }

                        let previous_scope =
                            std::mem::replace(&mut self.current_scope, method_scope);
                        let was_in_function = self.in_function;
                        let previous_class = self.current_class.replace(name.clone());
                        self.in_function = true;

                        let result = self.analyze_block(body);

                        self.in_function = was_in_function;
                        self.current_class = previous_class;
                        self.current_scope = previous_scope;
                        result?;
                    }
                }
            }
            Statement::FieldAssignment {
                object,
                field,
                value,
                line,
                column,
            } => {
                self.check_object_field_access(object, field, *line, *column)?;
                self.analyze_expression(object)?;
                self.analyze_expression(value)?;
            }
            Statement::TryCatch {
                try_block,
                catch_var,
                catch_block,
                line,
                column,
            } => {
                let try_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                let previous_scope = std::mem::replace(&mut self.current_scope, try_scope);
                let try_result = self.analyze_block(try_block);
                self.current_scope = previous_scope;
                try_result?;

                let catch_scope =
                    Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                catch_scope
                    .borrow_mut()
                    .define(catch_var.clone(), false, *line, *column)?;
                let previous_scope = std::mem::replace(&mut self.current_scope, catch_scope);
                let catch_result = self.analyze_block(catch_block);
                self.current_scope = previous_scope;
                catch_result?;
            }
            Statement::Throw { value, .. } => {
                self.analyze_expression(value)?;
            }
            Statement::ImportDeclaration {
                name,
                path,
                line,
                column,
            } => {
                if !self.analyzed_modules.contains_key(name) {
                    let (_, source) =
                        crate::source::resolve_module(name, path.as_deref(), *line, *column)?;

                    if self.active_imports.contains(name) {
                        return Err(CompilerError::RuntimeException {
                            message: format!("Circular dependency detected for module '{}'", name),
                            line: *line,
                            column: *column,
                        });
                    }

                    let tokens = crate::lexer::tokenize(&source)?;
                    let program = crate::parser::parse(tokens)?;

                    let mut sub_analyzer = SemanticAnalyzer::new();
                    sub_analyzer.analyzed_modules = self.analyzed_modules.clone();
                    sub_analyzer.active_imports = self.active_imports.clone();
                    sub_analyzer.active_imports.insert(name.clone());

                    sub_analyzer.analyze_program(&program)?;

                    for (mod_name, mod_exports) in sub_analyzer.analyzed_modules {
                        self.analyzed_modules.insert(mod_name, mod_exports);
                    }

                    let mut exports = HashSet::new();
                    for stmt in &program.statements {
                        if let Statement::ExportDeclaration { name: exp_name, .. } = stmt {
                            exports.insert(exp_name.clone());
                        }
                    }
                    self.analyzed_modules.insert(name.clone(), exports);
                }

                self.current_scope
                    .borrow_mut()
                    .define(name.clone(), false, *line, *column)?;
            }
            Statement::ExportDeclaration {
                name,
                declaration,
                line,
                column,
            } => {
                if let Some(decl) = declaration {
                    self.analyze_statement(decl)?;
                } else {
                    if self.current_scope.borrow().lookup(name).is_none() {
                        return Err(CompilerError::UndefinedVariable {
                            name: name.clone(),
                            line: *line,
                            column: *column,
                        });
                    }
                }
            }
            Statement::EnumDeclaration {
                name, line, column, ..
            } => {
                if self.struct_defs.contains_key(name) || self.class_defs.contains_key(name) {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                // Register enum as a "class" with no fields so constructors resolve.
                self.class_defs.insert(
                    name.clone(),
                    ClassInfo {
                        is_abstract: false,
                        extends: None,
                        implements: vec![],
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                    },
                );
            }
            Statement::MatchStatement { subject, arms, .. } => {
                self.analyze_expression(subject)?;
                for arm in arms {
                    let arm_scope =
                        Rc::new(RefCell::new(Scope::with_parent(self.current_scope.clone())));
                    // Wildcard / identifier patterns bind a variable in the arm scope.
                    if let crate::ast::MatchPattern::Identifier(binding) = &arm.pattern {
                        if binding != "_" {
                            arm_scope
                                .borrow_mut()
                                .define(binding.clone(), false, 0, 0)?;
                        }
                    }
                    let prev = std::mem::replace(&mut self.current_scope, arm_scope);
                    let result = self.analyze_block(&arm.body);
                    self.current_scope = prev;
                    result?;
                }
            }
            Statement::InterfaceDeclaration {
                name,
                methods,
                line,
                column,
            } => {
                if self.interface_defs.contains_key(name)
                    || self.class_defs.contains_key(name)
                    || self.struct_defs.contains_key(name)
                {
                    return Err(CompilerError::DuplicateDeclaration {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    });
                }
                let mut method_map = HashMap::new();
                for sig in methods {
                    method_map.insert(sig.name.clone(), sig.params.len());
                }
                self.interface_defs.insert(
                    name.clone(),
                    InterfaceInfo {
                        methods: method_map,
                    },
                );
            }
            Statement::SpawnStatement { expression, .. } => {
                self.analyze_expression(expression)?;
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

            Expression::Grouping { expression, .. } => self.analyze_expression(expression),

            Expression::UnaryOp { operand, .. } => self.analyze_expression(operand),

            Expression::BinaryOp { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)
            }

            Expression::Call {
                callee,
                arguments,
                line,
                column,
            } => {
                for argument in arguments {
                    self.analyze_expression(argument)?;
                }

                // Direct calls by name (the common case, e.g. `add(1, 2)`)
                // get a compile-time arity check. Indirect callees (e.g. a
                // call on the result of another expression) fall back to
                // a plain analysis pass, since we don't yet have a type
                // system to know whether they hold a function value.
                if let Expression::Identifier {
                    name,
                    line: id_line,
                    column: id_col,
                } = callee.as_ref()
                {
                    if let Some(class_info) = self.class_defs.get(name) {
                        if class_info.is_abstract {
                            return Err(CompilerError::AbstractClassInstantiation {
                                name: name.clone(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }

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

            Expression::StructLiteral {
                name,
                fields,
                line,
                column,
            } => {
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

            Expression::FieldAccess {
                object,
                field,
                line,
                column,
            } => {
                self.check_object_field_access(object, field, *line, *column)?;
                self.analyze_expression(object)
            }
            // ── Phase 4+ expressions ──────────────────────────────────
            Expression::ArrayLiteral { elements, .. } => {
                for el in elements {
                    self.analyze_expression(el)?;
                }
                Ok(())
            }
            Expression::IndexExpression { object, index, .. } => {
                self.analyze_expression(object)?;
                self.analyze_expression(index)
            }
            Expression::EnumVariantExpr { payload, .. } => {
                if let Some(p) = payload {
                    self.analyze_expression(p)?;
                }
                Ok(())
            }
            Expression::PropagateError { expression, .. } => self.analyze_expression(expression),
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
            Expression::AwaitExpr { expression, .. } => self.analyze_expression(expression),
            Expression::SuperExpr { line, column } => {
                if self.current_class.is_none() {
                    return Err(CompilerError::InvalidSuper {
                        line: *line,
                        column: *column,
                    });
                }
                Ok(())
            }
        }
    }
}
