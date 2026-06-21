//! Math module: `import math`.

use std::collections::HashMap;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    let fns: &[(
        &str,
        fn(Vec<RuntimeValue>, usize, usize) -> Result<RuntimeValue, CompilerError>,
    )] = &[
        ("sqrt", |args, l, c| {
            one_float(args, "sqrt", l, c).map(|f| RuntimeValue::Float(f.sqrt()))
        }),
        ("abs", |args, l, c| {
            one_float(args, "abs", l, c).map(|f| RuntimeValue::Float(f.abs()))
        }),
        ("floor", |args, l, c| {
            one_float(args, "floor", l, c).map(|f| RuntimeValue::Float(f.floor()))
        }),
        ("ceil", |args, l, c| {
            one_float(args, "ceil", l, c).map(|f| RuntimeValue::Float(f.ceil()))
        }),
        ("round", |args, l, c| {
            one_float(args, "round", l, c).map(|f| RuntimeValue::Float(f.round()))
        }),
        ("sin", |args, l, c| {
            one_float(args, "sin", l, c).map(|f| RuntimeValue::Float(f.sin()))
        }),
        ("cos", |args, l, c| {
            one_float(args, "cos", l, c).map(|f| RuntimeValue::Float(f.cos()))
        }),
        ("pow", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "pow".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let base = to_f64(&args[0], "pow", l, c)?;
            let exp = to_f64(&args[1], "pow", l, c)?;
            Ok(RuntimeValue::Float(base.powf(exp)))
        }),
        ("max", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "max".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let a = to_f64(&args[0], "max", l, c)?;
            let b = to_f64(&args[1], "max", l, c)?;
            Ok(RuntimeValue::Float(a.max(b)))
        }),
        ("min", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "min".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let a = to_f64(&args[0], "min", l, c)?;
            let b = to_f64(&args[1], "min", l, c)?;
            Ok(RuntimeValue::Float(a.min(b)))
        }),
    ];
    for (name, f) in fns {
        map.insert(name.to_string(), make_builtin(name, *f));
    }
    map.insert("PI".to_string(), RuntimeValue::Float(std::f64::consts::PI));
    map.insert("E".to_string(), RuntimeValue::Float(std::f64::consts::E));
    module_map(map)
}

fn one_float(
    args: Vec<RuntimeValue>,
    name: &str,
    line: usize,
    column: usize,
) -> Result<f64, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: name.into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    to_f64(&args[0], name, line, column)
}

fn to_f64(val: &RuntimeValue, op: &str, line: usize, column: usize) -> Result<f64, CompilerError> {
    match val {
        RuntimeValue::Integer(n) => Ok(*n as f64),
        RuntimeValue::Float(f) => Ok(*f),
        other => Err(CompilerError::TypeMismatch {
            operation: op.to_string(),
            expected: "Number".to_string(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}
