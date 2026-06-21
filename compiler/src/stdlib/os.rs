//! OS module: `import os`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "args".to_string(),
        make_builtin("os.args", |_args, _l, _c| {
            let args: Vec<RuntimeValue> = std::env::args().map(RuntimeValue::String).collect();
            Ok(RuntimeValue::Array(Rc::new(RefCell::new(args))))
        }),
    );
    map.insert(
        "env".to_string(),
        make_builtin("os.env", |args, line, column| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "os.env".into(),
                    expected: 1,
                    found: args.len(),
                    line,
                    column,
                });
            }
            if let RuntimeValue::String(key) = &args[0] {
                return Ok(RuntimeValue::String(
                    std::env::var(key).unwrap_or_default(),
                ));
            }
            Err(CompilerError::TypeMismatch {
                operation: "os.env()".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line,
                column,
            })
        }),
    );
    map.insert(
        "exit".to_string(),
        make_builtin("os.exit", |args, _l, _c| {
            let code = if let Some(RuntimeValue::Integer(n)) = args.first() {
                *n as i32
            } else {
                0
            };
            std::process::exit(code);
        }),
    );
    module_map(map)
}
