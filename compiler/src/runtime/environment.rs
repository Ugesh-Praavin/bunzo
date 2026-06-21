//! Scope management and variable bindings for the Bunzo runtime.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::RuntimeValue;
use crate::diagnostics::CompilerError;

/// Holds a single variable binding in an environment.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableBinding {
    /// The current value of the variable.
    pub value: RuntimeValue,
    /// Whether the variable is immutable (`const` vs `let`).
    pub is_const: bool,
}

/// An environment representing a lexical scope.
///
/// Environments are chained together via parent pointers to represent nested scopes.
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// The outer lexical scope, if any.
    parent: Option<Rc<RefCell<Environment>>>,
    /// The variable bindings defined in this immediate scope.
    bindings: HashMap<String, VariableBinding>,
}

impl Environment {
    /// Creates a new, parentless global environment.
    pub fn new() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    /// Creates a new environment nested inside the given parent environment.
    pub fn with_parent(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        }
    }

    /// Defines a new variable in the immediate scope.
    ///
    /// # Errors
    ///
    /// Returns a [`CompilerError::DuplicateDeclaration`] if the variable name is
    /// already defined in this immediate scope.
    pub fn define(
        &mut self,
        name: String,
        value: RuntimeValue,
        is_const: bool,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        if self.bindings.contains_key(&name) {
            return Err(CompilerError::DuplicateDeclaration { name, line, column });
        }
        self.bindings
            .insert(name, VariableBinding { value, is_const });
        Ok(())
    }

    /// Retrieves the value of a variable, traversing outer scopes if necessary.
    ///
    /// # Errors
    ///
    /// Returns a [`CompilerError::UndefinedVariable`] if the variable cannot be found.
    pub fn get(
        &self,
        name: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        if let Some(binding) = self.bindings.get(name) {
            return Ok(binding.value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name, line, column);
        }
        Err(CompilerError::UndefinedVariable {
            name: name.to_string(),
            line,
            column,
        })
    }

    /// Reassigns an existing variable, traversing outer scopes if necessary.
    ///
    /// # Errors
    ///
    /// - Returns a [`CompilerError::ConstReassignment`] if the variable is immutable.
    /// - Returns a [`CompilerError::UndefinedVariable`] if the variable cannot be found.
    pub fn assign(
        &mut self,
        name: String,
        value: RuntimeValue,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        if let Some(binding) = self.bindings.get_mut(&name) {
            if binding.is_const {
                return Err(CompilerError::ConstReassignment { name, line, column });
            }
            binding.value = value;
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value, line, column);
        }
        Err(CompilerError::UndefinedVariable { name, line, column })
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
