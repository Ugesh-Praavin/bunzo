//! JSON encode/decode module: `import json`.

use std::collections::HashMap;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "encode".to_string(),
        make_builtin("json.encode", |args, line, column| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "json.encode".into(),
                    expected: 1,
                    found: args.len(),
                    line,
                    column,
                });
            }
            Ok(RuntimeValue::String(runtime_value_to_json(&args[0])))
        }),
    );
    map.insert(
        "decode".to_string(),
        make_builtin("json.decode", |args, line, column| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "json.decode".into(),
                    expected: 1,
                    found: args.len(),
                    line,
                    column,
                });
            }
            match &args[0] {
                RuntimeValue::String(s) => json_str_to_value(s, line, column),
                other => Err(CompilerError::TypeMismatch {
                    operation: "json.decode()".into(),
                    expected: "String".into(),
                    found: other.type_name().to_string(),
                    line,
                    column,
                }),
            }
        }),
    );
    module_map(map)
}

fn runtime_value_to_json(val: &RuntimeValue) -> String {
    match val {
        RuntimeValue::Null => "null".to_string(),
        RuntimeValue::Boolean(b) => b.to_string(),
        RuntimeValue::Integer(n) => n.to_string(),
        RuntimeValue::Float(f) => f.to_string(),
        RuntimeValue::String(s) => format!(
            "\"{}\"",
            s.replace('\\', "\\\\").replace('"', "\\\"")
        ),
        RuntimeValue::Array(arr) => {
            let parts: Vec<String> = arr.borrow().iter().map(runtime_value_to_json).collect();
            format!("[{}]", parts.join(","))
        }
        RuntimeValue::Map(map) => {
            let mut keys: Vec<String> = map.borrow().keys().cloned().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "\"{}\":{}",
                        k,
                        runtime_value_to_json(map.borrow().get(k).unwrap())
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        RuntimeValue::Struct { fields, .. } => {
            let mut keys: Vec<String> = fields.keys().cloned().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "\"{}\":{}",
                        k,
                        runtime_value_to_json(fields.get(k).unwrap())
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        other => format!("\"{other}\""),
    }
}

fn json_str_to_value(s: &str, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    let s = s.trim();
    if s == "null" {
        return Ok(RuntimeValue::Null);
    }
    if s == "true" {
        return Ok(RuntimeValue::Boolean(true));
    }
    if s == "false" {
        return Ok(RuntimeValue::Boolean(false));
    }
    if let Ok(n) = s.parse::<i64>() {
        return Ok(RuntimeValue::Integer(n));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(RuntimeValue::Float(f));
    }
    if s.starts_with('"') && s.ends_with('"') {
        return Ok(RuntimeValue::String(
            s[1..s.len() - 1]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"),
        ));
    }
    Ok(RuntimeValue::String(s.to_string()))
}
