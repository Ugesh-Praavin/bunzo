//! Encoding module: hex_encode, hex_decode.

use std::collections::HashMap;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(hex: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut chars = hex.chars().filter(|c| !c.is_whitespace());
    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        let s = format!("{}{}", c1, c2);
        let b = u8::from_str_radix(&s, 16).map_err(|e| e.to_string())?;
        bytes.push(b);
    }
    Ok(bytes)
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "hex_encode".to_string(),
        make_builtin("encoding.hex_encode", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "encoding.hex_encode".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(s) = &args[0] {
                return Ok(RuntimeValue::String(to_hex(s.as_bytes())));
            }
            Err(CompilerError::TypeMismatch { operation: "encoding.hex_encode".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "hex_decode".to_string(),
        make_builtin("encoding.hex_decode", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "encoding.hex_decode".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(s) = &args[0] {
                match from_hex(s) {
                    Ok(bytes) => {
                        let decoded = String::from_utf8_lossy(&bytes).to_string();
                        return Ok(RuntimeValue::String(decoded));
                    }
                    Err(e) => return Err(CompilerError::RuntimeException { message: format!("invalid hex string: {e}"), line: l, column: c }),
                }
            }
            Err(CompilerError::TypeMismatch { operation: "encoding.hex_decode".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    module_map(map)
}
