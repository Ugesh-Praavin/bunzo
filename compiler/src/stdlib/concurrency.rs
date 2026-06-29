//! Concurrency modules: thread, mutex, rwlock, channel, atomic.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

fn to_map(
    val: &RuntimeValue,
    op: &str,
    l: usize,
    c: usize,
) -> Result<Rc<RefCell<HashMap<String, RuntimeValue>>>, CompilerError> {
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

// ─── Thread ──────────────────────────────────────────────────────────────────

pub fn build_thread() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "spawn".to_string(),
        make_builtin("thread.spawn", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "thread.spawn".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            // Execute or simulate execution, returns a thread ID.
            Ok(RuntimeValue::Integer(1))
        }),
    );
    map.insert(
        "join".to_string(),
        make_builtin("thread.join", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "thread.join".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            Ok(RuntimeValue::Null)
        }),
    );
    module_map(map)
}

// ─── Mutex ───────────────────────────────────────────────────────────────────

pub fn build_mutex() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "new".to_string(),
        make_builtin("mutex.new", |args, _l, _c| {
            let initial = args.first().cloned().unwrap_or(RuntimeValue::Null);
            let mut fields = HashMap::new();
            fields.insert("locked".to_string(), RuntimeValue::Boolean(false));
            fields.insert("value".to_string(), initial);
            Ok(RuntimeValue::Map(Rc::new(RefCell::new(fields))))
        }),
    );
    map.insert(
        "lock".to_string(),
        make_builtin("mutex.lock", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "mutex.lock".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let m = to_map(&args[0], "mutex.lock", l, c)?;
            m.borrow_mut()
                .insert("locked".to_string(), RuntimeValue::Boolean(true));
            let val = m
                .borrow()
                .get("value")
                .cloned()
                .unwrap_or(RuntimeValue::Null);
            Ok(val)
        }),
    );
    map.insert(
        "unlock".to_string(),
        make_builtin("mutex.unlock", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "mutex.unlock".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let m = to_map(&args[0], "mutex.unlock", l, c)?;
            m.borrow_mut()
                .insert("locked".to_string(), RuntimeValue::Boolean(false));
            Ok(RuntimeValue::Null)
        }),
    );
    module_map(map)
}

// ─── RWLock ──────────────────────────────────────────────────────────────────

pub fn build_rwlock() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "new".to_string(),
        make_builtin("rwlock.new", |args, _l, _c| {
            let initial = args.first().cloned().unwrap_or(RuntimeValue::Null);
            let mut fields = HashMap::new();
            fields.insert("readers".to_string(), RuntimeValue::Integer(0));
            fields.insert("writer".to_string(), RuntimeValue::Boolean(false));
            fields.insert("value".to_string(), initial);
            Ok(RuntimeValue::Map(Rc::new(RefCell::new(fields))))
        }),
    );
    map.insert(
        "read".to_string(),
        make_builtin("rwlock.read", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "rwlock.read".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let lock = to_map(&args[0], "rwlock.read", l, c)?;
            let mut b = lock.borrow_mut();
            let new_readers = if let Some(RuntimeValue::Integer(readers)) = b.get("readers") {
                Some(*readers + 1)
            } else {
                None
            };
            if let Some(r) = new_readers {
                b.insert("readers".to_string(), RuntimeValue::Integer(r));
            }
            let val = b.get("value").cloned().unwrap_or(RuntimeValue::Null);
            Ok(val)
        }),
    );
    map.insert(
        "write".to_string(),
        make_builtin("rwlock.write", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "rwlock.write".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let lock = to_map(&args[0], "rwlock.write", l, c)?;
            let mut b = lock.borrow_mut();
            b.insert("writer".to_string(), RuntimeValue::Boolean(true));
            let val = b.get("value").cloned().unwrap_or(RuntimeValue::Null);
            Ok(val)
        }),
    );
    module_map(map)
}

// ─── Channel ─────────────────────────────────────────────────────────────────

pub fn build_channel() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "new".to_string(),
        make_builtin("channel.new", |_args, _l, _c| {
            let mut fields = HashMap::new();
            fields.insert(
                "queue".to_string(),
                RuntimeValue::Array(Rc::new(RefCell::new(Vec::new()))),
            );
            Ok(RuntimeValue::Map(Rc::new(RefCell::new(fields))))
        }),
    );
    map.insert(
        "send".to_string(),
        make_builtin("channel.send", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "channel.send".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let chan = to_map(&args[0], "channel.send", l, c)?;
            let b_chan = chan.borrow();
            if let Some(RuntimeValue::Array(arr)) = b_chan.get("queue") {
                arr.borrow_mut().push(args[1].clone());
            }
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "recv".to_string(),
        make_builtin("channel.recv", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "channel.recv".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let chan = to_map(&args[0], "channel.recv", l, c)?;
            let b_chan = chan.borrow();
            if let Some(RuntimeValue::Array(arr)) = b_chan.get("queue") {
                let mut b_arr = arr.borrow_mut();
                if !b_arr.is_empty() {
                    return Ok(b_arr.remove(0));
                }
            }
            Ok(RuntimeValue::Null)
        }),
    );
    module_map(map)
}

// ─── Atomic ──────────────────────────────────────────────────────────────────

pub fn build_atomic() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "new".to_string(),
        make_builtin("atomic.new", |args, _l, _c| {
            let initial = args.first().cloned().unwrap_or(RuntimeValue::Integer(0));
            let mut fields = HashMap::new();
            fields.insert("value".to_string(), initial);
            Ok(RuntimeValue::Map(Rc::new(RefCell::new(fields))))
        }),
    );
    map.insert(
        "load".to_string(),
        make_builtin("atomic.load", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "atomic.load".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let a = to_map(&args[0], "atomic.load", l, c)?;
            let val = a
                .borrow()
                .get("value")
                .cloned()
                .unwrap_or(RuntimeValue::Null);
            Ok(val)
        }),
    );
    map.insert(
        "store".to_string(),
        make_builtin("atomic.store", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "atomic.store".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let a = to_map(&args[0], "atomic.store", l, c)?;
            a.borrow_mut().insert("value".to_string(), args[1].clone());
            Ok(RuntimeValue::Null)
        }),
    );
    map.insert(
        "add".to_string(),
        make_builtin("atomic.add", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "atomic.add".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let a = to_map(&args[0], "atomic.add", l, c)?;
            let mut b = a.borrow_mut();
            if let (Some(RuntimeValue::Integer(curr)), RuntimeValue::Integer(val)) =
                (b.get("value"), &args[1])
            {
                let new_val = *curr + *val;
                b.insert("value".to_string(), RuntimeValue::Integer(new_val));
                return Ok(RuntimeValue::Integer(new_val));
            }
            Err(CompilerError::TypeMismatch {
                operation: "atomic.add".into(),
                expected: "Integer".into(),
                found: args[1].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    module_map(map)
}
