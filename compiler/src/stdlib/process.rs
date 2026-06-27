//! Process module: exec, pid.

use std::collections::HashMap;
use std::process::Command;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "exec".to_string(),
        make_builtin("process.exec", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "process.exec".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(cmd) = &args[0] {
                // Split command into executable and args
                let mut parts = cmd.split_whitespace();
                if let Some(exe) = parts.next() {
                    let mut command = Command::new(exe);
                    for arg in parts {
                        command.arg(arg);
                    }
                    match command.output() {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            return Ok(RuntimeValue::String(stdout));
                        }
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to execute command '{cmd}': {e}"), line: l, column: c }),
                    }
                }
                return Ok(RuntimeValue::String(String::new()));
            }
            Err(CompilerError::TypeMismatch { operation: "process.exec".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "pid".to_string(),
        make_builtin("process.pid", |_args, _l, _c| {
            Ok(RuntimeValue::Integer(std::process::id() as i64))
        }),
    );
    module_map(map)
}
