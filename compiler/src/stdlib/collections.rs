//! Complete Collections standard library implementations.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

fn to_map(val: &RuntimeValue, op: &str, l: usize, c: usize) -> Result<Rc<RefCell<HashMap<String, RuntimeValue>>>, CompilerError> {
    if let RuntimeValue::Map(m) = val {
        return Ok(m.clone());
    }
    Err(CompilerError::TypeMismatch {
        operation: op.to_string(),
        expected: "Map".to_string(),
        found: val.type_name().to_string(),
        line: l,
        column: c,
    })
}

// ─── Vector ──────────────────────────────────────────────────────────────────

pub fn build_vector() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("vector.new", |_args, _l, _c| {
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("with_capacity".to_string(), make_builtin("vector.with_capacity", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.with_capacity".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("push".to_string(), make_builtin("vector.push", |args, l, c| {
        if args.len() != 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.push".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.push", l, c)?;
        arr.borrow_mut().push(args[1].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop".to_string(), make_builtin("vector.pop", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.pop".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.pop", l, c)?;
        Ok(arr.borrow_mut().pop().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("get".to_string(), make_builtin("vector.get", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.get".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.get", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let b = arr.borrow();
            if idx < b.len() {
                return Ok(b[idx].clone());
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.get".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("set".to_string(), make_builtin("vector.set", |args, l, c| {
        if args.len() < 3 {
            return Err(CompilerError::ArityMismatch { name: "vector.set".into(), expected: 3, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.set", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                b[idx] = args[2].clone();
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.set".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("insert".to_string(), make_builtin("vector.insert", |args, l, c| {
        if args.len() < 3 {
            return Err(CompilerError::ArityMismatch { name: "vector.insert".into(), expected: 3, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.insert", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx <= b.len() {
                b.insert(idx, args[2].clone());
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.insert".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("remove".to_string(), make_builtin("vector.remove", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.remove".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.remove", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                return Ok(b.remove(idx));
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.remove".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("front".to_string(), make_builtin("vector.front", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.front".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.front", l, c)?;
        Ok(arr.borrow().first().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("back".to_string(), make_builtin("vector.back", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.back".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.back", l, c)?;
        Ok(arr.borrow().last().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("len".to_string(), make_builtin("vector.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("capacity".to_string(), make_builtin("vector.capacity", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.capacity".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.capacity", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("vector.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(arr.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("vector.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.clear", l, c)?;
        arr.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("vector.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.contains", l, c)?;
        let val = &args[1];
        let contains = arr.borrow().iter().any(|x| format!("{x}") == format!("{val}"));
        Ok(RuntimeValue::Boolean(contains))
    }));
    map.insert("index_of".to_string(), make_builtin("vector.index_of", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.index_of".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.index_of", l, c)?;
        let val = &args[1];
        let pos = arr.borrow().iter().position(|x| format!("{x}") == format!("{val}"));
        Ok(pos.map(|idx| RuntimeValue::Integer(idx as i64)).unwrap_or(RuntimeValue::Integer(-1)))
    }));
    map.insert("reverse".to_string(), make_builtin("vector.reverse", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.reverse".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.reverse", l, c)?;
        arr.borrow_mut().reverse();
        Ok(RuntimeValue::Null)
    }));
    map.insert("sort".to_string(), make_builtin("vector.sort", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.sort".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.sort", l, c)?;
        arr.borrow_mut().sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
        Ok(RuntimeValue::Null)
    }));
    map.insert("resize".to_string(), make_builtin("vector.resize", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "vector.resize".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.resize", l, c)?;
        if let RuntimeValue::Integer(size) = &args[1] {
            let size = *size as usize;
            let mut b = arr.borrow_mut();
            b.resize(size, RuntimeValue::Null);
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.resize".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("swap".to_string(), make_builtin("vector.swap", |args, l, c| {
        if args.len() < 3 {
            return Err(CompilerError::ArityMismatch { name: "vector.swap".into(), expected: 3, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "vector.swap", l, c)?;
        if let (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) = (&args[1], &args[2]) {
            let a = *a as usize;
            let b = *b as usize;
            let mut arr_b = arr.borrow_mut();
            if a < arr_b.len() && b < arr_b.len() {
                arr_b.swap(a, b);
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "vector.swap".into(), expected: "Integer and Integer".into(), found: format!("{}, {}", args[1].type_name(), args[2].type_name()), line: l, column: c })
    }));
    map.insert("iter".to_string(), make_builtin("vector.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "vector.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}

// ─── Deque ───────────────────────────────────────────────────────────────────

pub fn build_deque() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("deque.new", |_args, _l, _c| {
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("push_front".to_string(), make_builtin("deque.push_front", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "deque.push_front".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.push_front", l, c)?;
        arr.borrow_mut().insert(0, args[1].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("push_back".to_string(), make_builtin("deque.push_back", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "deque.push_back".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.push_back", l, c)?;
        arr.borrow_mut().push(args[1].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop_front".to_string(), make_builtin("deque.pop_front", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.pop_front".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.pop_front", l, c)?;
        let mut b = arr.borrow_mut();
        if !b.is_empty() {
            return Ok(b.remove(0));
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop_back".to_string(), make_builtin("deque.pop_back", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.pop_back".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.pop_back", l, c)?;
        Ok(arr.borrow_mut().pop().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("front".to_string(), make_builtin("deque.front", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.front".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.front", l, c)?;
        Ok(arr.borrow().first().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("back".to_string(), make_builtin("deque.back", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.back".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.back", l, c)?;
        Ok(arr.borrow().last().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("get".to_string(), make_builtin("deque.get", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "deque.get".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.get", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let b = arr.borrow();
            if idx < b.len() {
                return Ok(b[idx].clone());
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "deque.get".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("set".to_string(), make_builtin("deque.set", |args, l, c| {
        if args.len() < 3 {
            return Err(CompilerError::ArityMismatch { name: "deque.set".into(), expected: 3, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.set", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                b[idx] = args[2].clone();
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "deque.set".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("len".to_string(), make_builtin("deque.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("deque.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(arr.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("deque.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.clear", l, c)?;
        arr.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("deque.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "deque.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "deque.contains", l, c)?;
        let val = &args[1];
        let contains = arr.borrow().iter().any(|x| format!("{x}") == format!("{val}"));
        Ok(RuntimeValue::Boolean(contains))
    }));
    map.insert("iter".to_string(), make_builtin("deque.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "deque.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}

// ─── Stack ───────────────────────────────────────────────────────────────────

pub fn build_stack() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("stack.new", |_args, _l, _c| {
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("push".to_string(), make_builtin("stack.push", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "stack.push".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.push", l, c)?;
        arr.borrow_mut().push(args[1].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop".to_string(), make_builtin("stack.pop", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.pop".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.pop", l, c)?;
        Ok(arr.borrow_mut().pop().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("top".to_string(), make_builtin("stack.top", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.top".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.top", l, c)?;
        Ok(arr.borrow().last().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("len".to_string(), make_builtin("stack.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("stack.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(arr.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("stack.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.clear", l, c)?;
        arr.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("stack.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "stack.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "stack.contains", l, c)?;
        let val = &args[1];
        let contains = arr.borrow().iter().any(|x| format!("{x}") == format!("{val}"));
        Ok(RuntimeValue::Boolean(contains))
    }));
    map.insert("iter".to_string(), make_builtin("stack.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "stack.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}

// ─── Queue ───────────────────────────────────────────────────────────────────

pub fn build_queue() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("queue.new", |_args, _l, _c| {
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("push".to_string(), make_builtin("queue.push", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "queue.push".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.push", l, c)?;
        arr.borrow_mut().push(args[1].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop".to_string(), make_builtin("queue.pop", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.pop".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.pop", l, c)?;
        let mut b = arr.borrow_mut();
        if !b.is_empty() {
            return Ok(b.remove(0));
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("front".to_string(), make_builtin("queue.front", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.front".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.front", l, c)?;
        Ok(arr.borrow().first().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("back".to_string(), make_builtin("queue.back", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.back".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.back", l, c)?;
        Ok(arr.borrow().last().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("len".to_string(), make_builtin("queue.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("queue.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(arr.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("queue.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.clear", l, c)?;
        arr.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("queue.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "queue.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "queue.contains", l, c)?;
        let val = &args[1];
        let contains = arr.borrow().iter().any(|x| format!("{x}") == format!("{val}"));
        Ok(RuntimeValue::Boolean(contains))
    }));
    map.insert("iter".to_string(), make_builtin("queue.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "queue.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}

// ─── Priority Queue ──────────────────────────────────────────────────────────

pub fn build_priority_queue() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("priority_queue.new", |_args, _l, _c| {
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))))
    }));
    map.insert("push".to_string(), make_builtin("priority_queue.push", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.push".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.push", l, c)?;
        let mut b = arr.borrow_mut();
        b.push(args[1].clone());
        b.sort_by(|x, y| format!("{x}").cmp(&format!("{y}"))); // Simple max-heap sorting representation
        Ok(RuntimeValue::Null)
    }));
    map.insert("pop".to_string(), make_builtin("priority_queue.pop", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.pop".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.pop", l, c)?;
        Ok(arr.borrow_mut().pop().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("top".to_string(), make_builtin("priority_queue.top", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.top".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.top", l, c)?;
        Ok(arr.borrow().last().cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("len".to_string(), make_builtin("priority_queue.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("priority_queue.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(arr.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("priority_queue.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.clear", l, c)?;
        arr.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("priority_queue.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "priority_queue.contains", l, c)?;
        let val = &args[1];
        let contains = arr.borrow().iter().any(|x| format!("{x}") == format!("{val}"));
        Ok(RuntimeValue::Boolean(contains))
    }));
    map.insert("iter".to_string(), make_builtin("priority_queue.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "priority_queue.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}

// ─── Set / HashSet ───────────────────────────────────────────────────────────

pub fn build_set() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("set.new", |_args, _l, _c| {
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(HashMap::new()))))
    }));
    map.insert("insert".to_string(), make_builtin("set.insert", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.insert".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "set.insert", l, c)?;
        let key = format!("{}", args[1]);
        m.borrow_mut().insert(key, RuntimeValue::Boolean(true));
        Ok(RuntimeValue::Null)
    }));
    map.insert("remove".to_string(), make_builtin("set.remove", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.remove".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "set.remove", l, c)?;
        let key = format!("{}", args[1]);
        m.borrow_mut().remove(&key);
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("set.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "set.contains", l, c)?;
        let key = format!("{}", args[1]);
        Ok(RuntimeValue::Boolean(m.borrow().contains_key(&key)))
    }));
    map.insert("len".to_string(), make_builtin("set.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.len", l, c)?;
        Ok(RuntimeValue::Integer(m.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("set.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(m.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("set.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.clear", l, c)?;
        m.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("first".to_string(), make_builtin("set.first", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.first".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.first", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.first().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("last".to_string(), make_builtin("set.last", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.last".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.last", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.last().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("min".to_string(), make_builtin("set.min", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.min".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.min", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.first().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("max".to_string(), make_builtin("set.max", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.max".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.max", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.last().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("lower_bound".to_string(), make_builtin("set.lower_bound", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.lower_bound".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "set.lower_bound", l, c)?;
        let bound = format!("{}", args[1]);
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        for k in keys {
            if k >= bound {
                return Ok(RuntimeValue::String(k));
            }
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("upper_bound".to_string(), make_builtin("set.upper_bound", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.upper_bound".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "set.upper_bound", l, c)?;
        let bound = format!("{}", args[1]);
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        for k in keys {
            if k > bound {
                return Ok(RuntimeValue::String(k));
            }
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("union".to_string(), make_builtin("set.union", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.union".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m1 = to_map(&args[0], "set.union", l, c)?;
        let m2 = to_map(&args[1], "set.union", l, c)?;
        let mut out = HashMap::new();
        for k in m1.borrow().keys() {
            out.insert(k.clone(), RuntimeValue::Boolean(true));
        }
        for k in m2.borrow().keys() {
            out.insert(k.clone(), RuntimeValue::Boolean(true));
        }
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(out))))
    }));
    map.insert("intersection".to_string(), make_builtin("set.intersection", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.intersection".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m1 = to_map(&args[0], "set.intersection", l, c)?;
        let m2 = to_map(&args[1], "set.intersection", l, c)?;
        let mut out = HashMap::new();
        for k in m1.borrow().keys() {
            if m2.borrow().contains_key(k) {
                out.insert(k.clone(), RuntimeValue::Boolean(true));
            }
        }
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(out))))
    }));
    map.insert("difference".to_string(), make_builtin("set.difference", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.difference".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m1 = to_map(&args[0], "set.difference", l, c)?;
        let m2 = to_map(&args[1], "set.difference", l, c)?;
        let mut out = HashMap::new();
        for k in m1.borrow().keys() {
            if !m2.borrow().contains_key(k) {
                out.insert(k.clone(), RuntimeValue::Boolean(true));
            }
        }
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(out))))
    }));
    map.insert("symmetric_difference".to_string(), make_builtin("set.symmetric_difference", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "set.symmetric_difference".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m1 = to_map(&args[0], "set.symmetric_difference", l, c)?;
        let m2 = to_map(&args[1], "set.symmetric_difference", l, c)?;
        let mut out = HashMap::new();
        for k in m1.borrow().keys() {
            if !m2.borrow().contains_key(k) {
                out.insert(k.clone(), RuntimeValue::Boolean(true));
            }
        }
        for k in m2.borrow().keys() {
            if !m1.borrow().contains_key(k) {
                out.insert(k.clone(), RuntimeValue::Boolean(true));
            }
        }
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(out))))
    }));
    map.insert("iter".to_string(), make_builtin("set.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "set.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "set.iter", l, c)?;
        let mut keys_vec: Vec<RuntimeValue> = m.borrow().keys().cloned().map(RuntimeValue::String).collect();
        keys_vec.sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(keys_vec))))
    }));
    module_map(map)
}

// ─── Map / HashMap ───────────────────────────────────────────────────────────

pub fn build_map() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("map.new", |_args, _l, _c| {
        Ok(RuntimeValue::Map(Rc::new(RefCell::new(HashMap::new()))))
    }));
    map.insert("insert".to_string(), make_builtin("map.insert", |args, l, c| {
        if args.len() < 3 {
            return Err(CompilerError::ArityMismatch { name: "map.insert".into(), expected: 3, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.insert", l, c)?;
        let key = format!("{}", args[1]);
        m.borrow_mut().insert(key, args[2].clone());
        Ok(RuntimeValue::Null)
    }));
    map.insert("get".to_string(), make_builtin("map.get", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "map.get".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.get", l, c)?;
        let key = format!("{}", args[1]);
        Ok(m.borrow().get(&key).cloned().unwrap_or(RuntimeValue::Null))
    }));
    map.insert("remove".to_string(), make_builtin("map.remove", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "map.remove".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.remove", l, c)?;
        let key = format!("{}", args[1]);
        m.borrow_mut().remove(&key);
        Ok(RuntimeValue::Null)
    }));
    map.insert("contains".to_string(), make_builtin("map.contains", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "map.contains".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.contains", l, c)?;
        let key = format!("{}", args[1]);
        Ok(RuntimeValue::Boolean(m.borrow().contains_key(&key)))
    }));
    map.insert("len".to_string(), make_builtin("map.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.len", l, c)?;
        Ok(RuntimeValue::Integer(m.borrow().len() as i64))
    }));
    map.insert("is_empty".to_string(), make_builtin("map.is_empty", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.is_empty".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.is_empty", l, c)?;
        Ok(RuntimeValue::Boolean(m.borrow().is_empty()))
    }));
    map.insert("clear".to_string(), make_builtin("map.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.clear", l, c)?;
        m.borrow_mut().clear();
        Ok(RuntimeValue::Null)
    }));
    map.insert("keys".to_string(), make_builtin("map.keys", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.keys".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.keys", l, c)?;
        let mut keys_vec: Vec<RuntimeValue> = m.borrow().keys().cloned().map(RuntimeValue::String).collect();
        keys_vec.sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(keys_vec))))
    }));
    map.insert("values".to_string(), make_builtin("map.values", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.values".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.values", l, c)?;
        let mut pairs: Vec<(String, RuntimeValue)> = m.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        pairs.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        let vals_vec: Vec<RuntimeValue> = pairs.into_iter().map(|(_, v)| v).collect();
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(vals_vec))))
    }));
    map.insert("first_key".to_string(), make_builtin("map.first_key", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.first_key".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.first_key", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.first().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("last_key".to_string(), make_builtin("map.last_key", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.last_key".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.last_key", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        Ok(keys.last().map(|k| RuntimeValue::String(k.clone())).unwrap_or(RuntimeValue::Null))
    }));
    map.insert("first_value".to_string(), make_builtin("map.first_value", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.first_value".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.first_value", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        if let Some(first_key) = keys.first() {
            return Ok(m.borrow().get(first_key).cloned().unwrap_or(RuntimeValue::Null));
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("last_value".to_string(), make_builtin("map.last_value", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.last_value".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.last_value", l, c)?;
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        if let Some(last_key) = keys.last() {
            return Ok(m.borrow().get(last_key).cloned().unwrap_or(RuntimeValue::Null));
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("lower_bound".to_string(), make_builtin("map.lower_bound", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "map.lower_bound".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.lower_bound", l, c)?;
        let bound = format!("{}", args[1]);
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        for k in keys {
            if k >= bound {
                return Ok(RuntimeValue::String(k));
            }
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("upper_bound".to_string(), make_builtin("map.upper_bound", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "map.upper_bound".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let m = to_map(&args[0], "map.upper_bound", l, c)?;
        let bound = format!("{}", args[1]);
        let mut keys: Vec<String> = m.borrow().keys().cloned().collect();
        keys.sort();
        for k in keys {
            if k > bound {
                return Ok(RuntimeValue::String(k));
            }
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("iter".to_string(), make_builtin("map.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "map.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        let m = to_map(&args[0], "map.iter", l, c)?;
        let mut keys_vec: Vec<RuntimeValue> = m.borrow().keys().cloned().map(RuntimeValue::String).collect();
        keys_vec.sort_by(|x, y| format!("{x}").cmp(&format!("{y}")));
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(keys_vec))))
    }));
    module_map(map)
}

// ─── BitSet ──────────────────────────────────────────────────────────────────

pub fn build_bitset() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("new".to_string(), make_builtin("bitset.new", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.new".into(), expected: 1, found: 0, line: l, column: c });
        }
        if let RuntimeValue::Integer(size) = &args[0] {
            let size = *size as usize;
            let bits = vec![RuntimeValue::Boolean(false); size];
            return Ok(RuntimeValue::Array(Rc::new(RefCell::new(bits))));
        }
        Err(CompilerError::TypeMismatch { operation: "bitset.new".into(), expected: "Integer".into(), found: args[0].type_name().to_string(), line: l, column: c })
    }));
    map.insert("set".to_string(), make_builtin("bitset.set", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.set".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.set", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                b[idx] = RuntimeValue::Boolean(true);
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "bitset.set".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("reset".to_string(), make_builtin("bitset.reset", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.reset".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.reset", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                b[idx] = RuntimeValue::Boolean(false);
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "bitset.reset".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("flip".to_string(), make_builtin("bitset.flip", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.flip".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.flip", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let mut b = arr.borrow_mut();
            if idx < b.len() {
                if let RuntimeValue::Boolean(curr) = b[idx] {
                    b[idx] = RuntimeValue::Boolean(!curr);
                }
            }
            return Ok(RuntimeValue::Null);
        }
        Err(CompilerError::TypeMismatch { operation: "bitset.flip".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("test".to_string(), make_builtin("bitset.test", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.test".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.test", l, c)?;
        if let RuntimeValue::Integer(idx) = &args[1] {
            let idx = *idx as usize;
            let b = arr.borrow();
            if idx < b.len() {
                return Ok(b[idx].clone());
            }
            return Ok(RuntimeValue::Boolean(false));
        }
        Err(CompilerError::TypeMismatch { operation: "bitset.test".into(), expected: "Integer".into(), found: args[1].type_name().to_string(), line: l, column: c })
    }));
    map.insert("count".to_string(), make_builtin("bitset.count", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.count".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.count", l, c)?;
        let count = arr.borrow().iter().filter(|x| matches!(x, RuntimeValue::Boolean(true))).count();
        Ok(RuntimeValue::Integer(count as i64))
    }));
    map.insert("any".to_string(), make_builtin("bitset.any", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.any".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.any", l, c)?;
        let any = arr.borrow().iter().any(|x| matches!(x, RuntimeValue::Boolean(true)));
        Ok(RuntimeValue::Boolean(any))
    }));
    map.insert("none".to_string(), make_builtin("bitset.none", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.none".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.none", l, c)?;
        let none = arr.borrow().iter().all(|x| matches!(x, RuntimeValue::Boolean(false)));
        Ok(RuntimeValue::Boolean(none))
    }));
    map.insert("all".to_string(), make_builtin("bitset.all", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.all".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.all", l, c)?;
        let all = arr.borrow().iter().all(|x| matches!(x, RuntimeValue::Boolean(true)));
        Ok(RuntimeValue::Boolean(all))
    }));
    map.insert("len".to_string(), make_builtin("bitset.len", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.len".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.len", l, c)?;
        Ok(RuntimeValue::Integer(arr.borrow().len() as i64))
    }));
    map.insert("clear".to_string(), make_builtin("bitset.clear", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.clear".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.clear", l, c)?;
        let mut b = arr.borrow_mut();
        for val in b.iter_mut() {
            *val = RuntimeValue::Boolean(false);
        }
        Ok(RuntimeValue::Null)
    }));
    map.insert("and".to_string(), make_builtin("bitset.and", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.and".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr1 = to_array(&args[0], "bitset.and", l, c)?;
        let arr2 = to_array(&args[1], "bitset.and", l, c)?;
        let len = std::cmp::min(arr1.borrow().len(), arr2.borrow().len());
        let mut out = Vec::new();
        for i in 0..len {
            let b1 = matches!(arr1.borrow()[i], RuntimeValue::Boolean(true));
            let b2 = matches!(arr2.borrow()[i], RuntimeValue::Boolean(true));
            out.push(RuntimeValue::Boolean(b1 && b2));
        }
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(out))))
    }));
    map.insert("or".to_string(), make_builtin("bitset.or", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.or".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr1 = to_array(&args[0], "bitset.or", l, c)?;
        let arr2 = to_array(&args[1], "bitset.or", l, c)?;
        let len = std::cmp::min(arr1.borrow().len(), arr2.borrow().len());
        let mut out = Vec::new();
        for i in 0..len {
            let b1 = matches!(arr1.borrow()[i], RuntimeValue::Boolean(true));
            let b2 = matches!(arr2.borrow()[i], RuntimeValue::Boolean(true));
            out.push(RuntimeValue::Boolean(b1 || b2));
        }
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(out))))
    }));
    map.insert("xor".to_string(), make_builtin("bitset.xor", |args, l, c| {
        if args.len() < 2 {
            return Err(CompilerError::ArityMismatch { name: "bitset.xor".into(), expected: 2, found: args.len(), line: l, column: c });
        }
        let arr1 = to_array(&args[0], "bitset.xor", l, c)?;
        let arr2 = to_array(&args[1], "bitset.xor", l, c)?;
        let len = std::cmp::min(arr1.borrow().len(), arr2.borrow().len());
        let mut out = Vec::new();
        for i in 0..len {
            let b1 = matches!(arr1.borrow()[i], RuntimeValue::Boolean(true));
            let b2 = matches!(arr2.borrow()[i], RuntimeValue::Boolean(true));
            out.push(RuntimeValue::Boolean(b1 ^ b2));
        }
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(out))))
    }));
    map.insert("not".to_string(), make_builtin("bitset.not", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.not".into(), expected: 1, found: 0, line: l, column: c });
        }
        let arr = to_array(&args[0], "bitset.not", l, c)?;
        let mut out = Vec::new();
        for val in arr.borrow().iter() {
            let b = matches!(val, RuntimeValue::Boolean(true));
            out.push(RuntimeValue::Boolean(!b));
        }
        Ok(RuntimeValue::Array(Rc::new(RefCell::new(out))))
    }));
    map.insert("iter".to_string(), make_builtin("bitset.iter", |args, l, c| {
        if args.is_empty() {
            return Err(CompilerError::ArityMismatch { name: "bitset.iter".into(), expected: 1, found: 0, line: l, column: c });
        }
        Ok(args[0].clone())
    }));
    module_map(map)
}
