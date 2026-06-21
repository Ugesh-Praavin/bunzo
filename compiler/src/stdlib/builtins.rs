//! Global builtins (`len`, `push`, `channel`, etc.) registered at interpreter startup.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::diagnostics::CompilerError;
use crate::runtime::environment::Environment;
use crate::runtime::value::RuntimeValue;

/// Registers all stdlib builtins into the given environment.
pub fn register_builtins(env: &Rc<RefCell<Environment>>) {
    let builtins: &[(
        &str,
        fn(Vec<RuntimeValue>, usize, usize) -> Result<RuntimeValue, CompilerError>,
    )] = &[
        ("len", builtin_len),
        ("type", builtin_type),
        ("str", builtin_str),
        ("to_int", builtin_to_int),
        ("to_float", builtin_to_float),
        ("input", builtin_input),
        ("push", builtin_push),
        ("pop", builtin_pop),
        ("keys", builtin_keys),
        ("values", builtin_values),
        ("contains", builtin_contains),
        ("split", builtin_split),
        ("join", builtin_join),
        ("map_fn", builtin_map_fn),
        ("map_set", builtin_map_set),
        ("map_get", builtin_map_get),
        ("channel", builtin_channel),
        ("send", builtin_send),
        ("recv", builtin_recv),
    ];
    let mut e = env.borrow_mut();
    for (name, func) in builtins {
        let _ = e.define(
            name.to_string(),
            RuntimeValue::Builtin {
                name: name.to_string(),
                func: *func,
            },
            true,
            0,
            0,
        );
    }
}

fn eval_equality(left: &RuntimeValue, right: &RuntimeValue) -> bool {
    match (left, right) {
        (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => l == r,
        (RuntimeValue::Float(l), RuntimeValue::Float(r)) => l == r,
        (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => (*l as f64) == *r,
        (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => *l == (*r as f64),
        (RuntimeValue::String(l), RuntimeValue::String(r)) => l == r,
        (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => l == r,
        (RuntimeValue::Null, RuntimeValue::Null) => true,
        _ => false,
    }
}

fn builtin_len(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "len".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    match &args[0] {
        RuntimeValue::String(s) => Ok(RuntimeValue::Integer(s.len() as i64)),
        other => Err(CompilerError::TypeMismatch {
            operation: "len()".into(),
            expected: "String".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_type(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "type".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    Ok(RuntimeValue::String(args[0].type_name().to_string()))
}

fn builtin_str(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "str".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    Ok(RuntimeValue::String(format!("{}", args[0])))
}

fn builtin_to_int(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "to_int".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    match &args[0] {
        RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(*n)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Integer(*f as i64)),
        RuntimeValue::String(s) => {
            s.trim()
                .parse::<i64>()
                .map(RuntimeValue::Integer)
                .map_err(|_| CompilerError::TypeMismatch {
                    operation: "to_int()".into(),
                    expected: "parseable integer string".into(),
                    found: format!("\"{s}\""),
                    line,
                    column,
                })
        }
        other => Err(CompilerError::TypeMismatch {
            operation: "to_int()".into(),
            expected: "Integer, Float, or String".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_to_float(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "to_float".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    match &args[0] {
        RuntimeValue::Integer(n) => Ok(RuntimeValue::Float(*n as f64)),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(*f)),
        RuntimeValue::String(s) => s
            .trim()
            .parse::<f64>()
            .map(RuntimeValue::Float)
            .map_err(|_| CompilerError::TypeMismatch {
                operation: "to_float()".into(),
                expected: "parseable float string".into(),
                found: format!("\"{s}\""),
                line,
                column,
            }),
        other => Err(CompilerError::TypeMismatch {
            operation: "to_float()".into(),
            expected: "Integer, Float, or String".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_input(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() > 1 {
        return Err(CompilerError::ArityMismatch {
            name: "input".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    if let Some(prompt) = args.first() {
        print!("{prompt}");
    }
    let mut line_buf = String::new();
    std::io::stdin()
        .read_line(&mut line_buf)
        .map_err(CompilerError::Io)?;
    Ok(RuntimeValue::String(
        line_buf
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string(),
    ))
}

fn builtin_push(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "push".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Array(arr) = &args[0] {
        arr.borrow_mut().push(args[1].clone());
        return Ok(args[0].clone());
    }
    Err(CompilerError::TypeMismatch {
        operation: "push()".into(),
        expected: "Array".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_pop(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "pop".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Array(arr) = &args[0] {
        return Ok(arr.borrow_mut().pop().unwrap_or(RuntimeValue::Null));
    }
    Err(CompilerError::TypeMismatch {
        operation: "pop()".into(),
        expected: "Array".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_keys(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "keys".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Map(map) = &args[0] {
        let keys: Vec<RuntimeValue> = map
            .borrow()
            .keys()
            .map(|k| RuntimeValue::String(k.clone()))
            .collect();
        return Ok(RuntimeValue::Array(Rc::new(RefCell::new(keys))));
    }
    Err(CompilerError::TypeMismatch {
        operation: "keys()".into(),
        expected: "Map".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_values(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "values".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Map(map) = &args[0] {
        let vals: Vec<RuntimeValue> = map.borrow().values().cloned().collect();
        return Ok(RuntimeValue::Array(Rc::new(RefCell::new(vals))));
    }
    Err(CompilerError::TypeMismatch {
        operation: "values()".into(),
        expected: "Map".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_contains(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "contains".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    match (&args[0], &args[1]) {
        (RuntimeValue::Array(arr), _) => {
            let found = arr.borrow().iter().any(|v| eval_equality(v, &args[1]));
            Ok(RuntimeValue::Boolean(found))
        }
        (RuntimeValue::Map(map), RuntimeValue::String(key)) => {
            Ok(RuntimeValue::Boolean(map.borrow().contains_key(key)))
        }
        (RuntimeValue::String(s), RuntimeValue::String(sub)) => {
            Ok(RuntimeValue::Boolean(s.contains(sub.as_str())))
        }
        _ => Err(CompilerError::TypeMismatch {
            operation: "contains()".into(),
            expected: "Array, Map, or String".into(),
            found: args[0].type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_split(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "split".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    match (&args[0], &args[1]) {
        (RuntimeValue::String(s), RuntimeValue::String(sep)) => {
            let parts: Vec<RuntimeValue> = s
                .split(sep.as_str())
                .map(|p| RuntimeValue::String(p.to_string()))
                .collect();
            Ok(RuntimeValue::Array(Rc::new(RefCell::new(parts))))
        }
        _ => Err(CompilerError::TypeMismatch {
            operation: "split()".into(),
            expected: "String, String".into(),
            found: args[0].type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_join(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "join".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    match (&args[0], &args[1]) {
        (RuntimeValue::Array(arr), RuntimeValue::String(sep)) => {
            let parts: Vec<String> = arr.borrow().iter().map(|v| format!("{v}")).collect();
            Ok(RuntimeValue::String(parts.join(sep)))
        }
        _ => Err(CompilerError::TypeMismatch {
            operation: "join()".into(),
            expected: "Array, String".into(),
            found: args[0].type_name().to_string(),
            line,
            column,
        }),
    }
}

fn builtin_map_fn(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.is_empty() {
        return Err(CompilerError::ArityMismatch {
            name: "map_fn".into(),
            expected: 1,
            found: 0,
            line,
            column,
        });
    }
    Ok(RuntimeValue::Map(Rc::new(RefCell::new(HashMap::new()))))
}

fn builtin_map_set(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 3 {
        return Err(CompilerError::ArityMismatch {
            name: "map_set".into(),
            expected: 3,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Map(map) = &args[0] {
        let key = match &args[1] {
            RuntimeValue::String(s) => s.clone(),
            other => format!("{other}"),
        };
        map.borrow_mut().insert(key, args[2].clone());
        return Ok(args[0].clone());
    }
    Err(CompilerError::TypeMismatch {
        operation: "map_set()".into(),
        expected: "Map".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_map_get(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "map_get".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Map(map) = &args[0] {
        let key = match &args[1] {
            RuntimeValue::String(s) => s.clone(),
            other => format!("{other}"),
        };
        return Ok(map
            .borrow()
            .get(&key)
            .cloned()
            .unwrap_or(RuntimeValue::Null));
    }
    Err(CompilerError::TypeMismatch {
        operation: "map_get()".into(),
        expected: "Map".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_channel(
    _args: Vec<RuntimeValue>,
    _line: usize,
    _column: usize,
) -> Result<RuntimeValue, CompilerError> {
    let (tx, rx) = std::sync::mpsc::channel::<RuntimeValue>();
    let sender = RuntimeValue::ChannelSender(Arc::new(Mutex::new(tx)));
    let receiver = RuntimeValue::Channel(Arc::new(Mutex::new(rx)));
    Ok(RuntimeValue::Array(Rc::new(RefCell::new(vec![
        sender, receiver,
    ]))))
}

fn builtin_send(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 2 {
        return Err(CompilerError::ArityMismatch {
            name: "send".into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::ChannelSender(tx) = &args[0] {
        tx.lock()
            .unwrap()
            .send(args[1].clone())
            .map_err(|_| CompilerError::RuntimeException {
                message: "channel closed".to_string(),
                line,
                column,
            })?;
        return Ok(RuntimeValue::Null);
    }
    Err(CompilerError::TypeMismatch {
        operation: "send()".into(),
        expected: "ChannelSender".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}

fn builtin_recv(
    args: Vec<RuntimeValue>,
    line: usize,
    column: usize,
) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "recv".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    if let RuntimeValue::Channel(rx) = &args[0] {
        let val = rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|_| CompilerError::RuntimeException {
                message: "channel closed".to_string(),
                line,
                column,
            })?;
        return Ok(val);
    }
    Err(CompilerError::TypeMismatch {
        operation: "recv()".into(),
        expected: "Channel".into(),
        found: args[0].type_name().to_string(),
        line,
        column,
    })
}
