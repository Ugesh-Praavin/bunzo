//! Generic algorithms for Bunzo collections: sort, stable_sort, reverse, shuffle, find, find_if, binary_search, etc.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

fn to_array(val: &RuntimeValue, op: &str, l: usize, c: usize) -> Result<Rc<RefCell<Vec<RuntimeValue>>>, CompilerError> {
    if let RuntimeValue::Array(arr) = val {
        return Ok(arr.clone());
    }
    Err(CompilerError::TypeMismatch {
        operation: op.to_string(),
        expected: "Array".to_string(),
        found: val.type_name().to_string(),
        line: l,
        column: c,
    })
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "sort".to_string(),
        make_builtin("algorithm.sort", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.sort".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.sort", l, c)?;
            arr.borrow_mut().sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "stable_sort".to_string(),
        make_builtin("algorithm.stable_sort", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.stable_sort".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.stable_sort", l, c)?;
            arr.borrow_mut().sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "reverse".to_string(),
        make_builtin("algorithm.reverse", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.reverse".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.reverse", l, c)?;
            arr.borrow_mut().reverse();
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "shuffle".to_string(),
        make_builtin("algorithm.shuffle", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.shuffle".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.shuffle", l, c)?;
            // Simple deterministic/mock shuffle
            let mut b = arr.borrow_mut();
            let len = b.len();
            for i in 0..len {
                let target = (i * 3 + 1) % len;
                b.swap(i, target);
            }
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "find".to_string(),
        make_builtin("algorithm.find", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.find".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.find", l, c)?;
            let target = &args[1];
            let pos = arr.borrow().iter().position(|x| format!("{x}") == format!("{target}"));
            Ok(pos.map(|idx| RuntimeValue::Integer(idx as i64)).unwrap_or(RuntimeValue::Integer(-1)))
        }),
    );
    map.insert(
        "find_if".to_string(),
        make_builtin("algorithm.find_if", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.find_if".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.find_if", l, c)?;
            // Return first element position (simple mock)
            let b = arr.borrow();
            if !b.is_empty() {
                return Ok(RuntimeValue::Integer(0));
            }
            Ok(RuntimeValue::Integer(-1))
        }),
    );
    map.insert(
        "binary_search".to_string(),
        make_builtin("algorithm.binary_search", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.binary_search".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.binary_search", l, c)?;
            let target = &args[1];
            let found = arr.borrow().binary_search_by(|x| format!("{x}").cmp(&format!("{target}"))).is_ok();
            Ok(RuntimeValue::Boolean(found))
        }),
    );
    map.insert(
        "lower_bound".to_string(),
        make_builtin("algorithm.lower_bound", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.lower_bound".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.lower_bound", l, c)?;
            let target = &args[1];
            let pos = arr.borrow().partition_point(|x| format!("{x}") < format!("{target}"));
            Ok(RuntimeValue::Integer(pos as i64))
        }),
    );
    map.insert(
        "upper_bound".to_string(),
        make_builtin("algorithm.upper_bound", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.upper_bound".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.upper_bound", l, c)?;
            let target = &args[1];
            let pos = arr.borrow().partition_point(|x| format!("{x}") <= format!("{target}"));
            Ok(RuntimeValue::Integer(pos as i64))
        }),
    );
    map.insert(
        "min".to_string(),
        make_builtin("algorithm.min", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.min".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.min", l, c)?;
            let min_val = arr.borrow().iter().min_by(|x, y| format!("{x}").cmp(&format!("{y}"))).cloned().unwrap_or(RuntimeValue::Null);
            Ok(min_val)
        }),
    );
    map.insert(
        "max".to_string(),
        make_builtin("algorithm.max", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.max".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.max", l, c)?;
            let max_val = arr.borrow().iter().max_by(|x, y| format!("{x}").cmp(&format!("{y}"))).cloned().unwrap_or(RuntimeValue::Null);
            Ok(max_val)
        }),
    );
    map.insert(
        "min_element".to_string(),
        make_builtin("algorithm.min_element", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.min_element".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.min_element", l, c)?;
            let pos = arr.borrow().iter().position(|x| format!("{x}") == format!("{}", arr.borrow().iter().min_by(|a, b| format!("{a}").cmp(&format!("{b}"))).unwrap()));
            Ok(pos.map(|idx| RuntimeValue::Integer(idx as i64)).unwrap_or(RuntimeValue::Integer(-1)))
        }),
    );
    map.insert(
        "max_element".to_string(),
        make_builtin("algorithm.max_element", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.max_element".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.max_element", l, c)?;
            let pos = arr.borrow().iter().position(|x| format!("{x}") == format!("{}", arr.borrow().iter().max_by(|a, b| format!("{a}").cmp(&format!("{b}"))).unwrap()));
            Ok(pos.map(|idx| RuntimeValue::Integer(idx as i64)).unwrap_or(RuntimeValue::Integer(-1)))
        }),
    );
    map.insert(
        "copy".to_string(),
        make_builtin("algorithm.copy", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.copy".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.copy", l, c)?;
            let new_arr = arr.borrow().clone();
            Ok(RuntimeValue::Array(Rc::new(RefCell::new(new_arr))))
        }),
    );
    map.insert(
        "fill".to_string(),
        make_builtin("algorithm.fill", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.fill".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.fill", l, c)?;
            let val = &args[1];
            for x in arr.borrow_mut().iter_mut() {
                *x = val.clone();
            }
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "rotate".to_string(),
        make_builtin("algorithm.rotate", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.rotate".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.rotate", l, c)?;
            if let RuntimeValue::Integer(pivot) = &args[1] {
                let mut b = arr.borrow_mut();
                let pivot = *pivot as usize;
                if pivot < b.len() {
                    b.rotate_left(pivot);
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch { operation: "algorithm.rotate".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "swap".to_string(),
        make_builtin("algorithm.swap", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.swap".into(), expected: 3, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.swap", l, c)?;
            if let (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) = (&args[1], &args[2]) {
                let a = *a as usize;
                let b = *b as usize;
                let mut b_arr = arr.borrow_mut();
                if a < b_arr.len() && b < b_arr.len() {
                    b_arr.swap(a, b);
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch { operation: "algorithm.swap".into(), expected: "Integer, Integer".into(), found: format!("{}, {}", args[1].type_name(), args[2].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "unique".to_string(),
        make_builtin("algorithm.unique", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch { name: "algorithm.unique".into(), expected: 1, found: 0, line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.unique", l, c)?;
            let mut b = arr.borrow_mut();
            b.dedup_by(|x, y| format!("{x}") == format!("{y}"));
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "count".to_string(),
        make_builtin("algorithm.count", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "algorithm.count".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            let arr = to_array(&args[0], "algorithm.count", l, c)?;
            let target = &args[1];
            let count = arr.borrow().iter().filter(|x| format!("{x}") == format!("{target}")).count();
            Ok(RuntimeValue::Integer(count as i64))
        }),
    );
    module_map(map)
}
