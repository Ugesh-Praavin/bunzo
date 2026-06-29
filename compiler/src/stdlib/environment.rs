//! Environment module: environment.get, environment.set, environment.has, environment.remove, environment.all.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "get".to_string(),
        make_builtin("environment.get", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "environment.get".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(key) = &args[0] {
                return Ok(RuntimeValue::String(std::env::var(key).unwrap_or_default()));
            }
            Err(CompilerError::TypeMismatch {
                operation: "environment.get".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "set".to_string(),
        make_builtin("environment.set", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "environment.set".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(key), RuntimeValue::String(val)) = (&args[0], &args[1]) {
                unsafe {
                    std::env::set_var(key, val);
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "environment.set".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "has".to_string(),
        make_builtin("environment.has", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "environment.has".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(key) = &args[0] {
                return Ok(RuntimeValue::Boolean(std::env::var(key).is_ok()));
            }
            Err(CompilerError::TypeMismatch {
                operation: "environment.has".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "remove".to_string(),
        make_builtin("environment.remove", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "environment.remove".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(key) = &args[0] {
                unsafe {
                    std::env::remove_var(key);
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "environment.remove".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "all".to_string(),
        make_builtin("environment.all", |_args, _l, _c| {
            let mut out = HashMap::new();
            for (k, v) in std::env::vars() {
                out.insert(k, RuntimeValue::String(v));
            }
            Ok(RuntimeValue::Map(Rc::new(RefCell::new(out))))
        }),
    );
    module_map(map)
}
