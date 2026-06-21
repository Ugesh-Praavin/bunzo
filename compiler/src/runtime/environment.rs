//! Scope management and variable bindings for the Bunzo runtime.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::RuntimeValue;
use crate::diagnostics::CompilerError;

/// A single variable binding.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableBinding {
    pub value: RuntimeValue,
    pub is_const: bool,
    /// Whether this name was marked `export`.
    pub exported: bool,
}

/// A lexical scope environment.
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub bindings: HashMap<String, VariableBinding>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        }
    }

    /// Build a shallow snapshot (no parent) for thread spawning.
    pub fn snapshot(env: &Environment) -> Environment {
        Environment {
            parent: None,
            bindings: env.bindings.clone(),
        }
    }

    /// Define a variable in this immediate scope.
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
        self.bindings.insert(
            name,
            VariableBinding {
                value,
                is_const,
                exported: false,
            },
        );
        Ok(())
    }

    /// Define and mark as exported.
    pub fn define_exported(
        &mut self,
        name: String,
        value: RuntimeValue,
        is_const: bool,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        self.define(name.clone(), value, is_const, line, column)?;
        if let Some(b) = self.bindings.get_mut(&name) {
            b.exported = true;
        }
        Ok(())
    }

    /// Mark an existing binding as exported in this scope.
    pub fn mark_exported(&mut self, name: &str) {
        if let Some(b) = self.bindings.get_mut(name) {
            b.exported = true;
        }
    }

    /// Get a value, traversing parent scopes.
    pub fn get(
        &self,
        name: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        if let Some(b) = self.bindings.get(name) {
            return Ok(b.value.clone());
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

    /// Get a value only from this scope (no parent traversal) — used by module export.
    pub fn get_direct(&self, name: &str) -> Option<RuntimeValue> {
        self.bindings.get(name).map(|b| b.value.clone())
    }

    /// Return all exported names in this immediate scope.
    pub fn exported_names(&self) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|(_, b)| b.exported)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Reassign an existing variable, traversing parent scopes.
    pub fn assign(
        &mut self,
        name: String,
        value: RuntimeValue,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        if let Some(binding) = self.bindings.get_mut(&name) {
            if binding.is_const {
                // Allow assigning `Moved` sentinel even to const (for move semantics)
                if !matches!(value, RuntimeValue::Moved) {
                    return Err(CompilerError::ConstReassignment { name, line, column });
                }
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
