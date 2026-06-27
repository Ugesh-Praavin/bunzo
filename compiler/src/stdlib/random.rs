//! Random module: int, float, bool.

use std::collections::HashMap;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

std::thread_local! {
    static SEED: std::cell::RefCell<u64> = std::cell::RefCell::new(123456789);
}

fn next_random() -> u64 {
    SEED.with(|s| {
        let mut seed = s.borrow_mut();
        *seed = (*seed).wrapping_mul(1103515245).wrapping_add(12345) % 2147483648;
        *seed
    })
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "int".to_string(),
        make_builtin("random.int", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "random.int".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::Integer(min), RuntimeValue::Integer(max)) = (&args[0], &args[1]) {
                if min >= max {
                    return Ok(RuntimeValue::Integer(*min));
                }
                let range = (max - min) as u64;
                let val = (next_random() % range) as i64 + min;
                return Ok(RuntimeValue::Integer(val));
            }
            Err(CompilerError::TypeMismatch { operation: "random.int".into(), expected: "Integer and Integer".into(), found: format!("{}, {}", args[0].type_name(), args[1].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "float".to_string(),
        make_builtin("random.float", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "random.float".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::Float(min), RuntimeValue::Float(max)) = (&args[0], &args[1]) {
                let r = next_random() as f64 / 2147483648.0;
                let val = min + r * (max - min);
                return Ok(RuntimeValue::Float(val));
            }
            Err(CompilerError::TypeMismatch { operation: "random.float".into(), expected: "Float and Float".into(), found: format!("{}, {}", args[0].type_name(), args[1].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "bool".to_string(),
        make_builtin("random.bool", |_args, _l, _c| {
            let val = (next_random() % 2) == 0;
            Ok(RuntimeValue::Boolean(val))
        }),
    );
    module_map(map)
}
