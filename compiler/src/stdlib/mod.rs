//! Built-in Bunzo standard library modules (`import http`, `import db`, etc.).

pub mod algorithm;
pub mod builtins;
pub mod collections;
pub mod concurrency;
pub mod crypto;
pub mod db;
pub mod encoding;
pub mod environment;
pub mod filesystem;
pub mod http;
pub mod io_module;
pub mod json;
pub mod math;
pub mod networking;
pub mod numeric;
pub mod os;
pub mod path;
pub mod process;
pub mod random;
pub mod regex;
pub mod string;
pub mod test;
pub mod time_module;

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
        "vector" => Some(collections::build_vector()),
        "deque" => Some(collections::build_deque()),
        "stack" => Some(collections::build_stack()),
        "queue" => Some(collections::build_queue()),
        "priority_queue" => Some(collections::build_priority_queue()),
        "set" => Some(collections::build_set()),
        "hashset" => Some(collections::build_set()),
        "map" => Some(collections::build_map()),
        "hashmap" => Some(collections::build_map()),
        "bitset" => Some(collections::build_bitset()),
        "string" => Some(string::build()),
        "filesystem" => Some(filesystem::build()),
        "path" => Some(path::build()),
        "time" => Some(time_module::build()),
        "random" => Some(random::build()),
        "crypto" => Some(crypto::build()),
        "encoding" => Some(encoding::build()),
        "process" => Some(process::build()),
        "environment" => Some(environment::build()),
        "io" => Some(io_module::build()),
        "networking" => Some(networking::build()),
        "thread" => Some(concurrency::build_thread()),
        "mutex" => Some(concurrency::build_mutex()),
        "rwlock" => Some(concurrency::build_rwlock()),
        "channel" => Some(concurrency::build_channel()),
        "atomic" => Some(concurrency::build_atomic()),
        "regex" => Some(regex::build()),
        "algorithm" => Some(algorithm::build()),
        "numeric" => Some(numeric::build()),
        "test" => Some(test::build()),
        _ => None,
    }
}

pub(crate) fn make_builtin(
    name: &str,
    func: fn(
        Vec<RuntimeValue>,
        usize,
        usize,
    ) -> Result<RuntimeValue, crate::diagnostics::CompilerError>,
) -> RuntimeValue {
    RuntimeValue::Builtin {
        name: name.to_string(),
        func,
    }
}

pub(crate) fn module_map(map: std::collections::HashMap<String, RuntimeValue>) -> RuntimeValue {
    RuntimeValue::Map(std::rc::Rc::new(std::cell::RefCell::new(map)))
}
