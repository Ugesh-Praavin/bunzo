//! String module: len, split, join, trim, replace, substring.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "len".to_string(),
        make_builtin("string.len", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "string.len".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(s) = &args[0] {
                return Ok(RuntimeValue::Integer(s.len() as i64));
            }
            Err(CompilerError::TypeMismatch { operation: "string.len".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "split".to_string(),
        make_builtin("string.split", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "string.split".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::String(s), RuntimeValue::String(sep)) = (&args[0], &args[1]) {
                let parts: Vec<RuntimeValue> = s.split(sep).map(|p| RuntimeValue::String(p.to_string())).collect();
                return Ok(RuntimeValue::Array(Rc::new(RefCell::new(parts))));
            }
            Err(CompilerError::TypeMismatch { operation: "string.split".into(), expected: "String and String".into(), found: format!("{}, {}", args[0].type_name(), args[1].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "join".to_string(),
        make_builtin("string.join", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "string.join".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::Array(arr), RuntimeValue::String(sep)) = (&args[0], &args[1]) {
                let parts: Vec<String> = arr.borrow().iter().map(|v| format!("{v}")).collect();
                return Ok(RuntimeValue::String(parts.join(sep)));
            }
            Err(CompilerError::TypeMismatch { operation: "string.join".into(), expected: "Array and String".into(), found: format!("{}, {}", args[0].type_name(), args[1].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "trim".to_string(),
        make_builtin("string.trim", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "string.trim".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(s) = &args[0] {
                return Ok(RuntimeValue::String(s.trim().to_string()));
            }
            Err(CompilerError::TypeMismatch { operation: "string.trim".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "replace".to_string(),
        make_builtin("string.replace", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch { name: "string.replace".into(), expected: 3, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::String(s), RuntimeValue::String(old), RuntimeValue::String(new)) = (&args[0], &args[1], &args[2]) {
                return Ok(RuntimeValue::String(s.replace(old, new)));
            }
            Err(CompilerError::TypeMismatch { operation: "string.replace".into(), expected: "String, String, and String".into(), found: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "substring".to_string(),
        make_builtin("string.substring", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch { name: "string.substring".into(), expected: 3, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::String(s), RuntimeValue::Integer(start), RuntimeValue::Integer(end)) = (&args[0], &args[1], &args[2]) {
                let start = *start as usize;
                let end = *end as usize;
                if start <= end && end <= s.len() {
                    return Ok(RuntimeValue::String(s[start..end].to_string()));
                }
                return Err(CompilerError::RuntimeException { message: format!("index out of bounds for substring: start={}, end={}, len={}", start, end, s.len()), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "string.substring".into(), expected: "String, Integer, and Integer".into(), found: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()), line: l, column: c })
        }),
    );
    module_map(map)
}
