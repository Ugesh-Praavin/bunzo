//! Filesystem module: read, write, exists, mkdir, remove, listdir.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "read".to_string(),
        make_builtin("filesystem.read", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.read".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path) = &args[0] {
                match fs::read_to_string(path) {
                    Ok(content) => return Ok(RuntimeValue::String(content)),
                    Err(e) => {
                        return Err(CompilerError::RuntimeException {
                            message: format!("failed to read file '{path}': {e}"),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.read".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "write".to_string(),
        make_builtin("filesystem.write", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.write".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(path), RuntimeValue::String(data)) = (&args[0], &args[1]) {
                match fs::write(path, data) {
                    Ok(_) => return Ok(RuntimeValue::Null),
                    Err(e) => {
                        return Err(CompilerError::RuntimeException {
                            message: format!("failed to write file '{path}': {e}"),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.write".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "exists".to_string(),
        make_builtin("filesystem.exists", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.exists".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path) = &args[0] {
                let exists = fs::metadata(path).is_ok();
                return Ok(RuntimeValue::Boolean(exists));
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.exists".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "mkdir".to_string(),
        make_builtin("filesystem.mkdir", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.mkdir".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path) = &args[0] {
                match fs::create_dir_all(path) {
                    Ok(_) => return Ok(RuntimeValue::Null),
                    Err(e) => {
                        return Err(CompilerError::RuntimeException {
                            message: format!("failed to create directory '{path}': {e}"),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.mkdir".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "remove".to_string(),
        make_builtin("filesystem.remove", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.remove".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path) = &args[0] {
                let is_dir = fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false);
                let res = if is_dir {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
                match res {
                    Ok(_) => return Ok(RuntimeValue::Null),
                    Err(e) => {
                        return Err(CompilerError::RuntimeException {
                            message: format!("failed to remove path '{path}': {e}"),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.remove".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "listdir".to_string(),
        make_builtin("filesystem.listdir", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "filesystem.listdir".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(path) = &args[0] {
                match fs::read_dir(path) {
                    Ok(entries) => {
                        let mut paths = Vec::new();
                        for entry in entries {
                            if let Ok(e) = entry {
                                paths.push(RuntimeValue::String(
                                    e.path().to_string_lossy().to_string(),
                                ));
                            }
                        }
                        return Ok(RuntimeValue::Array(Rc::new(RefCell::new(paths))));
                    }
                    Err(e) => {
                        return Err(CompilerError::RuntimeException {
                            message: format!("failed to list directory '{path}': {e}"),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Err(CompilerError::TypeMismatch {
                operation: "filesystem.listdir".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    module_map(map)
}
