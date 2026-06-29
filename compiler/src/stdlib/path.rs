//! Path module: join, basename, dirname, extension.

use std::collections::HashMap;
use std::path::Path;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "join".to_string(),
        make_builtin("path.join", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "path.join".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(p1), RuntimeValue::String(p2)) = (&args[0], &args[1]) {
                let path = Path::new(p1).join(p2);
                return Ok(RuntimeValue::String(path.to_string_lossy().to_string()));
            }
            Err(CompilerError::TypeMismatch {
                operation: "path.join".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "basename".to_string(),
        make_builtin("path.basename", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "path.basename".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path_str) = &args[0] {
                let name = Path::new(path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                return Ok(RuntimeValue::String(name));
            }
            Err(CompilerError::TypeMismatch {
                operation: "path.basename".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "dirname".to_string(),
        make_builtin("path.dirname", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "path.dirname".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path_str) = &args[0] {
                let parent = Path::new(path_str)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                return Ok(RuntimeValue::String(parent));
            }
            Err(CompilerError::TypeMismatch {
                operation: "path.dirname".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "extension".to_string(),
        make_builtin("path.extension", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "path.extension".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path_str) = &args[0] {
                let ext = Path::new(path_str)
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                return Ok(RuntimeValue::String(ext));
            }
            Err(CompilerError::TypeMismatch {
                operation: "path.extension".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    module_map(map)
}
