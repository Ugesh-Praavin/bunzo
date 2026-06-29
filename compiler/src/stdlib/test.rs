//! Unit testing utilities: assert, assert_eq, assert_ne, assert_true, assert_false, fail, skip.

use std::collections::HashMap;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "assert".to_string(),
        make_builtin("test.assert", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "test.assert".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::Boolean(cond) = &args[0] {
                if !cond {
                    return Err(CompilerError::RuntimeException {
                        message: "assertion failed".to_string(),
                        line: l,
                        column: c,
                    });
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "test.assert".into(),
                expected: "Boolean".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "assert_eq".to_string(),
        make_builtin("test.assert_eq", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "test.assert_eq".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if format!("{}", args[0]) != format!("{}", args[1]) {
                return Err(CompilerError::RuntimeException {
                    message: format!("assertion failed: {} != {}", args[0], args[1]),
                    line: l,
                    column: c,
                });
            }
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "assert_ne".to_string(),
        make_builtin("test.assert_ne", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "test.assert_ne".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if format!("{}", args[0]) == format!("{}", args[1]) {
                return Err(CompilerError::RuntimeException {
                    message: format!("assertion failed: {} == {}", args[0], args[1]),
                    line: l,
                    column: c,
                });
            }
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "assert_true".to_string(),
        make_builtin("test.assert_true", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "test.assert_true".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::Boolean(cond) = &args[0] {
                if !cond {
                    return Err(CompilerError::RuntimeException {
                        message: "assertion failed: expected true".to_string(),
                        line: l,
                        column: c,
                    });
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "test.assert_true".into(),
                expected: "Boolean".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "assert_false".to_string(),
        make_builtin("test.assert_false", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "test.assert_false".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::Boolean(cond) = &args[0] {
                if *cond {
                    return Err(CompilerError::RuntimeException {
                        message: "assertion failed: expected false".to_string(),
                        line: l,
                        column: c,
                    });
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "test.assert_false".into(),
                expected: "Boolean".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "fail".to_string(),
        make_builtin("test.fail", |args, l, c| {
            let msg = if let Some(RuntimeValue::String(s)) = args.first() {
                s.clone()
            } else {
                "test failed".to_string()
            };
            Err(CompilerError::RuntimeException {
                message: msg,
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "skip".to_string(),
        make_builtin("test.skip", |args, _l, _c| {
            let msg = if let Some(RuntimeValue::String(s)) = args.first() {
                s.clone()
            } else {
                "skipped".to_string()
            };
            println!("[SKIPPED] {}", msg);
            Ok(RuntimeValue::Null)
        }),
    );
    module_map(map)
}
