//! Time module: now, sleep, timestamp.

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "now".to_string(),
        make_builtin("time.now", |_args, _l, _c| {
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
            Ok(RuntimeValue::Integer(since_the_epoch.as_millis() as i64))
        }),
    );
    map.insert(
        "sleep".to_string(),
        make_builtin("time.sleep", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "time.sleep".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::Integer(ms) = &args[0] {
                thread::sleep(Duration::from_millis(*ms as u64));
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch { operation: "time.sleep".into(), expected: "Integer".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "timestamp".to_string(),
        make_builtin("time.timestamp", |_args, _l, _c| {
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
            Ok(RuntimeValue::Float(since_the_epoch.as_secs_f64()))
        }),
    );
    module_map(map)
}
