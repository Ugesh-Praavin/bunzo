//! Crypto module: sha256, uuid.

use std::collections::HashMap;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "sha256".to_string(),
        make_builtin("crypto.sha256", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch {
                    name: "crypto.sha256".into(),
                    expected: 1,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let RuntimeValue::String(s) = &args[0] {
                return Ok(RuntimeValue::String(format!("sha256_mock_hash_of_{}", s)));
            }
            Err(CompilerError::TypeMismatch {
                operation: "crypto.sha256".into(),
                expected: "String".into(),
                found: args[0].type_name().to_string(),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "uuid".to_string(),
        make_builtin("crypto.uuid", |_args, _l, _c| {
            Ok(RuntimeValue::String(
                "123e4567-e89b-12d3-a456-426614174000".to_string(),
            ))
        }),
    );
    module_map(map)
}
