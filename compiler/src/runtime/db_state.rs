//! Embedded database state for `import db`.

use std::collections::HashMap;

use crate::runtime::value::RuntimeValue;

#[derive(Debug, Clone)]
pub struct DbState {
    pub path: String,
    pub tables: HashMap<String, Table>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, RuntimeValue>>,
}
