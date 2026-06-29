//! Numeric operations: min, max, clamp, abs, gcd, lcm, factorial, average, sum, product, accumulate.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

fn to_array(
    val: &RuntimeValue,
    op: &str,
    l: usize,
    c: usize,
) -> Result<Rc<RefCell<Vec<RuntimeValue>>>, CompilerError> {
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

fn gcd_val(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a.abs()
}

fn compare_values(x: &RuntimeValue, y: &RuntimeValue) -> std::cmp::Ordering {
    match (x, y) {
        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a.cmp(b),
        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        }
        (RuntimeValue::Integer(a), RuntimeValue::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (RuntimeValue::Float(a), RuntimeValue::Integer(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => format!("{x}").cmp(&format!("{y}")),
    }
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "min".to_string(),
        make_builtin("numeric.min", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.min".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if compare_values(&args[0], &args[1]) == std::cmp::Ordering::Less {
                Ok(args[0].clone())
            } else {
                Ok(args[1].clone())
            }
        }),
    );
    map.insert(
        "max".to_string(),
        make_builtin("numeric.max", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.max".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if compare_values(&args[0], &args[1]) == std::cmp::Ordering::Greater {
                Ok(args[0].clone())
            } else {
                Ok(args[1].clone())
            }
        }),
    );
    map.insert(
        "clamp".to_string(),
        make_builtin("numeric.clamp", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.clamp".into(),
                    expected: 3,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if compare_values(&args[0], &args[1]) == std::cmp::Ordering::Less {
                Ok(args[1].clone())
            } else if compare_values(&args[0], &args[2]) == std::cmp::Ordering::Greater {
                Ok(args[2].clone())
            } else {
                Ok(args[0].clone())
            }
        }),
    );
    map.insert(
        "abs".to_string(),
        make_builtin("numeric.abs", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.abs".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            match &args[0] {
                RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(n.abs())),
                RuntimeValue::Float(f) => Ok(RuntimeValue::Float(f.abs())),
                _ => Err(CompilerError::TypeMismatch {
                    operation: "numeric.abs".into(),
                    expected: "Integer or Float".into(),
                    found: args[0].type_name().to_string(),
                    line: l,
                    column: c,
                }),
            }
        }),
    );
    map.insert(
        "gcd".to_string(),
        make_builtin("numeric.gcd", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.gcd".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) = (&args[0], &args[1]) {
                return Ok(RuntimeValue::Integer(gcd_val(*a, *b)));
            }
            Err(CompilerError::TypeMismatch {
                operation: "numeric.gcd".into(),
                expected: "Integer and Integer".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "lcm".to_string(),
        make_builtin("numeric.lcm", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.lcm".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) = (&args[0], &args[1]) {
                let g = gcd_val(*a, *b);
                if g == 0 {
                    return Ok(RuntimeValue::Integer(0));
                }
                return Ok(RuntimeValue::Integer((a * b).abs() / g));
            }
            Err(CompilerError::TypeMismatch {
                operation: "numeric.lcm".into(),
                expected: "Integer and Integer".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "factorial".to_string(),
        make_builtin("numeric.factorial", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.factorial".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::Integer(n) = &args[0] {
                let mut fact: i64 = 1;
                for i in 2..=*n {
                    fact = fact.wrapping_mul(i);
                }
                return Ok(RuntimeValue::Integer(fact));
            }
            Err(CompilerError::TypeMismatch {
                operation: "numeric.factorial".into(),
                expected: "Integer".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "average".to_string(),
        make_builtin("numeric.average", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.average".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let arr = to_array(&args[0], "numeric.average", l, c)?;
            let b = arr.borrow();
            if b.is_empty() {
                return Ok(RuntimeValue::Float(0.0));
            }
            let mut sum = 0.0;
            for x in b.iter() {
                match x {
                    RuntimeValue::Integer(n) => sum += *n as f64,
                    RuntimeValue::Float(f) => sum += *f,
                    _ => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "numeric.average".into(),
                            expected: "Integer or Float".into(),
                            found: x.type_name().to_string(),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Ok(RuntimeValue::Float(sum / b.len() as f64))
        }),
    );
    map.insert(
        "sum".to_string(),
        make_builtin("numeric.sum", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.sum".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let arr = to_array(&args[0], "numeric.sum", l, c)?;
            let b = arr.borrow();
            let mut sum_int = 0;
            let mut sum_float = 0.0;
            let mut has_float = false;
            for x in b.iter() {
                match x {
                    RuntimeValue::Integer(n) => {
                        sum_int += *n;
                        sum_float += *n as f64;
                    }
                    RuntimeValue::Float(f) => {
                        sum_float += *f;
                        has_float = true;
                    }
                    _ => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "numeric.sum".into(),
                            expected: "Integer or Float".into(),
                            found: x.type_name().to_string(),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            if has_float {
                Ok(RuntimeValue::Float(sum_float))
            } else {
                Ok(RuntimeValue::Integer(sum_int))
            }
        }),
    );
    map.insert(
        "product".to_string(),
        make_builtin("numeric.product", |args, l, c| {
            if args.is_empty() {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.product".into(),
                    expected: 1,
                    found: 0,
                    line: l,
                    column: c,
                });
            }
            let arr = to_array(&args[0], "numeric.product", l, c)?;
            let b = arr.borrow();
            let mut prod_int = 1;
            let mut prod_float = 1.0;
            let mut has_float = false;
            for x in b.iter() {
                match x {
                    RuntimeValue::Integer(n) => {
                        prod_int *= *n;
                        prod_float *= *n as f64;
                    }
                    RuntimeValue::Float(f) => {
                        prod_float *= *f;
                        has_float = true;
                    }
                    _ => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "numeric.product".into(),
                            expected: "Integer or Float".into(),
                            found: x.type_name().to_string(),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            if has_float {
                Ok(RuntimeValue::Float(prod_float))
            } else {
                Ok(RuntimeValue::Integer(prod_int))
            }
        }),
    );
    map.insert(
        "accumulate".to_string(),
        make_builtin("numeric.accumulate", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "numeric.accumulate".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            let arr = to_array(&args[0], "numeric.accumulate", l, c)?;
            let init = &args[1];
            let b = arr.borrow();
            let mut current = init.clone();
            for x in b.iter() {
                match (&current, x) {
                    (RuntimeValue::Integer(c_val), RuntimeValue::Integer(x_val)) => {
                        current = RuntimeValue::Integer(c_val + x_val)
                    }
                    (RuntimeValue::Float(c_val), RuntimeValue::Float(x_val)) => {
                        current = RuntimeValue::Float(c_val + x_val)
                    }
                    (RuntimeValue::Float(c_val), RuntimeValue::Integer(x_val)) => {
                        current = RuntimeValue::Float(c_val + *x_val as f64)
                    }
                    (RuntimeValue::Integer(c_val), RuntimeValue::Float(x_val)) => {
                        current = RuntimeValue::Float(*c_val as f64 + x_val)
                    }
                    (RuntimeValue::String(c_val), RuntimeValue::String(x_val)) => {
                        current = RuntimeValue::String(format!("{}{}", c_val, x_val))
                    }
                    _ => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "numeric.accumulate".into(),
                            expected: "numeric or string types".into(),
                            found: format!("{}, {}", current.type_name(), x.type_name()),
                            line: l,
                            column: c,
                        });
                    }
                }
            }
            Ok(current)
        }),
    );
    module_map(map)
}
