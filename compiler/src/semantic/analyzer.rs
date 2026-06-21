//! Static semantic analysis for Bunzo programs.
//!
//! This module checks whether the parsed Abstract Syntax Tree follows
//! the semantic rules of the language, such as validating variable references
//! and detecting duplicate variable declarations in the same scope.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Block, Expression, Program, Statement};
use crate::diagnostics::CompilerError;

/// A symbol defined in a lexical scope at compile-time.
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The name of the symbol.
    pub name: String,
    /// Whether the symbol is immutable (`const` vs `let`).
    pub is_const: bool,
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

    /// Defines a symbol in the immediate scope.
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
        if self.symbols.contains_key(&name) {
            return Err(CompilerError::DuplicateDeclaration { name, line, column });
        }
        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                is_const,
                line,
                column,
            },
        );
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

struct SemanticAnalyzer {
    current_scope: Rc<RefCell<Scope>>,
}

impl SemanticAnalyzer {
    fn new() -> Self {
        Self {
            current_scope: Rc::new(RefCell::new(Scope::new())),
        }
    }

    fn analyze_program(&mut self, program: &Program) -> Result<(), CompilerError> {
        for stmt in &program.statements {
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
            Statement::AssignStatement {
                name,
                value,
                line,
                column,
            } => {
                self.analyze_expression(value)?;
                let symbol = self.current_scope.borrow().lookup(name);
                match symbol {
                    Some(sym) => {
                        if sym.is_const {
                            return Err(CompilerError::ConstReassignment {
                                name: name.clone(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }
                    None => {
                        return Err(CompilerError::UndefinedVariable {
                            name: name.clone(),
                            line: *line,
                            column: *column,
                        });
                    }
                }
            }
            Statement::PrintStatement { argument, .. } => {
                self.analyze_expression(argument)?;
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.analyze_expression(condition)?;
                self.analyze_block(then_branch)?;
                if let Some(else_blk) = else_branch {
                    self.analyze_block(else_blk)?;
                }
            }
            Statement::WhileStatement {
                condition, body, ..
            } => {
                self.analyze_expression(condition)?;
                self.analyze_block(body)?;
            }
            Statement::ForInStatement {
                variable,
                iterable,
                body,
                line,
                column,
            } => {
                self.analyze_expression(iterable)?;

                // ForInStatement's loop variable is scoped to a child scope.
                let child_scope = Rc::new(RefCell::new(Scope::with_parent(Rc::clone(
                    &self.current_scope,
                ))));
                child_scope.borrow_mut().define(
                    variable.clone(),
                    false, // is_const = false (loop variables are reassignable inside the body)
                    *line,
                    *column,
                )?;

                let saved = Rc::clone(&self.current_scope);
                self.current_scope = child_scope;

                for stmt in &body.statements {
                    self.analyze_statement(stmt)?;
                }

                self.current_scope = saved;
            }
            Statement::Break { .. } => {}
            Statement::Continue { .. } => {}
            Statement::ExpressionStatement { expression } => {
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

            Expression::Range { start, end, .. } => {
                self.analyze_expression(start)?;
                self.analyze_expression(end)
            }
        }
    }

    /// Analyzes a block in a fresh child scope.
    ///
    /// Opens a new scope, analyzes all statements, then restores the parent scope.
    fn analyze_block(&mut self, block: &Block) -> Result<(), CompilerError> {
        let child_scope = Rc::new(RefCell::new(Scope::with_parent(Rc::clone(
            &self.current_scope,
        ))));
        let saved = Rc::clone(&self.current_scope);
        self.current_scope = child_scope;

        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }

        // Restore parent scope.
        self.current_scope = saved;
        Ok(())
    }
}
