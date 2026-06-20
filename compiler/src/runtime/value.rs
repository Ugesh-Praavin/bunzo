//! Representation of Bunzo values at runtime.

use std::fmt;

/// A runtime value in the Bunzo interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// An integer value.
    Integer(i64),
    /// A double-precision floating-point value.
    Float(f64),
    /// A UTF-8 string value.
    String(String),
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// The `null` value.
    Null,
}

impl RuntimeValue {
    /// Returns a user-friendly name of the value's type.
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Integer(_) => "Integer",
            RuntimeValue::Float(_) => "Float",
            RuntimeValue::String(_) => "String",
            RuntimeValue::Boolean(_) => "Boolean",
            RuntimeValue::Null => "Null",
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Integer(val) => write!(f, "{val}"),
            RuntimeValue::Float(val) => write!(f, "{val}"),
            RuntimeValue::String(val) => write!(f, "{val}"),
            RuntimeValue::Boolean(val) => write!(f, "{val}"),
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}
