//! IO module: read_line, read_char.

use std::collections::HashMap;
use std::io::{self, Read};

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "read_line".to_string(),
        make_builtin("io.read_line", |_args, l, c| {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => Ok(RuntimeValue::String(input.trim_end().to_string())),
                Err(e) => Err(CompilerError::RuntimeException { message: format!("failed to read line: {e}"), line: l, column: c }),
            }
        }),
    );
    map.insert(
        "read_char".to_string(),
        make_builtin("io.read_char", |_args, l, c| {
            let mut buf = [0; 1];
            match io::stdin().read(&mut buf) {
                Ok(_) => {
                    let char_str = String::from_utf8_lossy(&buf).to_string();
                    Ok(RuntimeValue::String(char_str))
                }
                Err(e) => Err(CompilerError::RuntimeException { message: format!("failed to read character: {e}"), line: l, column: c }),
            }
        }),
    );
    module_map(map)
}
