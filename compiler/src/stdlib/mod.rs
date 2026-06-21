//! Built-in Bunzo standard library modules (`import http`, `import db`, etc.).

pub mod builtins;
pub mod db;
pub mod http;
pub mod json;
pub mod math;
pub mod os;

pub use builtins::register_builtins;

use crate::runtime::value::RuntimeValue;

/// Returns a built-in module for `import <name>`, when `name` is a stdlib module.
pub fn build_module(name: &str) -> Option<RuntimeValue> {
    match name {
        "json" => Some(json::build()),
        "http" => Some(http::build()),
        "math" => Some(math::build()),
        "os" => Some(os::build()),
        "db" => Some(db::build()),
        _ => None,
    }
}

pub(crate) fn make_builtin(
    name: &str,
    func: fn(Vec<RuntimeValue>, usize, usize) -> Result<RuntimeValue, crate::diagnostics::CompilerError>,
) -> RuntimeValue {
    RuntimeValue::Builtin {
        name: name.to_string(),
        func,
    }
}

pub(crate) fn module_map(map: std::collections::HashMap<String, RuntimeValue>) -> RuntimeValue {
    RuntimeValue::Map(std::rc::Rc::new(std::cell::RefCell::new(map)))
}
