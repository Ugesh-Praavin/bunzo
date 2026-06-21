//! Database module: `import db` — embedded SQL (SQLite-compatible subset, pure Rust).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::diagnostics::CompilerError;
use crate::runtime::db_state::{DbState, Table};
use crate::runtime::value::RuntimeValue;

use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("open".to_string(), make_builtin("db.open", db_open));
    map.insert(
        "execute".to_string(),
        make_builtin("db.execute", db_execute),
    );
    map.insert("query".to_string(), make_builtin("db.query", db_query));
    map.insert("close".to_string(), make_builtin("db.close", db_close));
    module_map(map)
}

fn require_connection(
    args: &[RuntimeValue],
    op: &str,
    line: usize,
    column: usize,
) -> Result<Arc<Mutex<DbState>>, CompilerError> {
    if args.is_empty() {
        return Err(CompilerError::ArityMismatch {
            name: op.into(),
            expected: 1,
            found: 0,
            line,
            column,
        });
    }
    match &args[0] {
        RuntimeValue::DbConnection(conn) => Ok(conn.clone()),
        other => Err(CompilerError::TypeMismatch {
            operation: format!("{op}()"),
            expected: "DbConnection".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn require_sql(
    args: &[RuntimeValue],
    op: &str,
    line: usize,
    column: usize,
) -> Result<String, CompilerError> {
    if args.len() < 2 {
        return Err(CompilerError::ArityMismatch {
            name: op.into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    match &args[1] {
        RuntimeValue::String(s) => Ok(s.clone()),
        other => Err(CompilerError::TypeMismatch {
            operation: format!("{op}()"),
            expected: "String".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn runtime_err(msg: impl Into<String>, line: usize, column: usize) -> CompilerError {
    CompilerError::RuntimeException {
        message: msg.into(),
        line,
        column,
    }
}

fn parse_sql_value(token: &str) -> RuntimeValue {
    let t = token.trim();
    if t.eq_ignore_ascii_case("null") {
        return RuntimeValue::Null;
    }
    if t.eq_ignore_ascii_case("true") {
        return RuntimeValue::Boolean(true);
    }
    if t.eq_ignore_ascii_case("false") {
        return RuntimeValue::Boolean(false);
    }
    if (t.starts_with('\'') && t.ends_with('\'')) || (t.starts_with('"') && t.ends_with('"')) {
        return RuntimeValue::String(t[1..t.len() - 1].to_string());
    }
    if let Ok(n) = t.parse::<i64>() {
        return RuntimeValue::Integer(n);
    }
    if let Ok(f) = t.parse::<f64>() {
        return RuntimeValue::Float(f);
    }
    RuntimeValue::String(t.to_string())
}

fn split_csv_list(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote = '\0';
    for ch in inner.chars() {
        match ch {
            '\'' | '"' if !in_quote => {
                in_quote = true;
                quote = ch;
                current.push(ch);
            }
            c if in_quote && c == quote => {
                in_quote = false;
                current.push(c);
            }
            ',' if !in_quote => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn parse_where(sql: &str) -> Option<(String, RuntimeValue)> {
    let upper = sql.to_uppercase();
    let idx = upper.find(" WHERE ")?;
    let rest = sql[idx + 7..].trim();
    let eq = rest.find('=')?;
    let col = rest[..eq].trim().to_string();
    let val = parse_sql_value(rest[eq + 1..].trim());
    Some((col, val))
}

fn values_match(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    match (a, b) {
        (RuntimeValue::Integer(x), RuntimeValue::Integer(y)) => x == y,
        (RuntimeValue::Float(x), RuntimeValue::Float(y)) => x == y,
        (RuntimeValue::String(x), RuntimeValue::String(y)) => x == y,
        (RuntimeValue::Boolean(x), RuntimeValue::Boolean(y)) => x == y,
        (RuntimeValue::Null, RuntimeValue::Null) => true,
        _ => format!("{a}") == format!("{b}"),
    }
}

fn row_to_map(row: &HashMap<String, RuntimeValue>) -> RuntimeValue {
    RuntimeValue::Map(Rc::new(RefCell::new(row.clone())))
}

fn execute_sql(state: &mut DbState, sql: &str, line: usize, column: usize) -> Result<i64, CompilerError> {
    let sql_trim = sql.trim().trim_end_matches(';');
    let upper = sql_trim.to_uppercase();

    if upper.starts_with("CREATE TABLE") {
        let rest = sql_trim[12..].trim();
        let (name, cols_part) = rest
            .split_once('(')
            .ok_or_else(|| runtime_err("invalid CREATE TABLE syntax", line, column))?;
        let name = name.trim().to_string();
        let cols_inner = cols_part
            .trim_end_matches(')')
            .trim();
        let columns: Vec<String> = split_csv_list(cols_inner)
            .into_iter()
            .map(|c| c.split_whitespace().next().unwrap_or("").to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if columns.is_empty() {
            return Err(runtime_err("CREATE TABLE requires columns", line, column));
        }
        state.tables.insert(
            name,
            Table {
                columns,
                rows: Vec::new(),
            },
        );
        return Ok(0);
    }

    if upper.starts_with("INSERT INTO") {
        let rest = sql_trim[11..].trim();
        let values_idx = rest.to_uppercase().find(" VALUES ");
        let (head, vals_section) = values_idx
            .map(|i| (&rest[..i], &rest[i + 8..]))
            .ok_or_else(|| runtime_err("INSERT requires VALUES", line, column))?;

        let table_name = if let Some(paren) = head.find('(') {
            head[..paren].trim().to_string()
        } else {
            head.trim().to_string()
        };

        let table = state
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| runtime_err(format!("table not found: {table_name}"), line, column))?;

        let cols_part = if head.contains('(') {
            let start = head.find('(').unwrap() + 1;
            let end = head.find(')').unwrap_or(head.len());
            head[start..end].trim()
        } else {
            ""
        };

        let vals_inner = vals_section
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();
        let values = split_csv_list(vals_inner);

        let columns: Vec<String> = if cols_part.is_empty() {
            table.columns.clone()
        } else {
            split_csv_list(cols_part)
        };

        if columns.len() != values.len() {
            return Err(runtime_err("INSERT column/value count mismatch", line, column));
        }

        let mut row = HashMap::new();
        for (col, val_token) in columns.iter().zip(values.iter()) {
            row.insert(col.clone(), parse_sql_value(val_token));
        }
        table.rows.push(row);
        return Ok(1);
    }

    if upper.starts_with("DELETE FROM") {
        let rest = sql_trim[11..].trim();
        let table_name = rest
            .split_whitespace()
            .next()
            .ok_or_else(|| runtime_err("invalid DELETE syntax", line, column))?
            .to_string();
        let table = state
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| runtime_err(format!("table not found: {table_name}"), line, column))?;

        if let Some((col, val)) = parse_where(sql_trim) {
            let before = table.rows.len();
            table.rows.retain(|row| {
                row.get(&col)
                    .map(|v| !values_match(v, &val))
                    .unwrap_or(true)
            });
            return Ok((before - table.rows.len()) as i64);
        }
        let count = table.rows.len() as i64;
        table.rows.clear();
        return Ok(count);
    }

    if upper.starts_with("UPDATE ") {
        let rest = sql_trim[7..].trim();
        let (table_name, rest) = rest
            .split_once(" SET ")
            .or_else(|| rest.split_once(" set "))
            .ok_or_else(|| runtime_err("invalid UPDATE syntax", line, column))?;
        let table_name = table_name.trim().to_string();
        let table = state
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| runtime_err(format!("table not found: {table_name}"), line, column))?;

        let where_idx = rest.to_uppercase().find(" WHERE ");
        let (set_part, where_part) = if let Some(i) = where_idx {
            (&rest[..i], Some(&rest[i + 7..]))
        } else {
            (rest, None)
        };
        let eq = set_part
            .find('=')
            .ok_or_else(|| runtime_err("UPDATE SET requires col = value", line, column))?;
        let col = set_part[..eq].trim().to_string();
        let val = parse_sql_value(set_part[eq + 1..].trim());

        let mut affected = 0i64;
        for row in &mut table.rows {
            let matches = if let Some(w) = where_part {
                let (wcol, wval) = {
                    let eq = w.find('=').ok_or_else(|| runtime_err("invalid WHERE", line, column))?;
                    (w[..eq].trim().to_string(), parse_sql_value(w[eq + 1..].trim()))
                };
                row.get(&wcol).map(|v| values_match(v, &wval)).unwrap_or(false)
            } else {
                true
            };
            if matches {
                row.insert(col.clone(), val.clone());
                affected += 1;
            }
        }
        return Ok(affected);
    }

    Err(runtime_err(
        format!("unsupported SQL for execute: {sql_trim}"),
        line,
        column,
    ))
}

fn query_sql(
    state: &DbState,
    sql: &str,
    line: usize,
    column: usize,
) -> Result<Vec<RuntimeValue>, CompilerError> {
    let sql_trim = sql.trim().trim_end_matches(';');
    let upper = sql_trim.to_uppercase();
    if !upper.starts_with("SELECT ") {
        return Err(runtime_err("query() expects a SELECT statement", line, column));
    }

    let rest = sql_trim[7..].trim();
    let from_idx = rest.to_uppercase().find(" FROM ");
    let (select_part, from_part) = from_idx
        .map(|i| (&rest[..i], &rest[i + 6..]))
        .ok_or_else(|| runtime_err("SELECT requires FROM", line, column))?;

    let table_name = from_part
        .split_whitespace()
        .next()
        .ok_or_else(|| runtime_err("invalid FROM clause", line, column))?
        .to_string();
    let table = state
        .tables
        .get(&table_name)
        .ok_or_else(|| runtime_err(format!("table not found: {table_name}"), line, column))?;

    let where_clause = parse_where(from_part);
    let select_all = select_part.trim() == "*";
    let selected_cols: Vec<String> = if select_all {
        table.columns.clone()
    } else {
        split_csv_list(select_part)
    };

    let mut result = Vec::new();
    for row in &table.rows {
        if let Some((ref wcol, ref wval)) = where_clause {
            if row
                .get(wcol)
                .map(|v| !values_match(v, wval))
                .unwrap_or(true)
            {
                continue;
            }
        }
        let mut out = HashMap::new();
        for col in &selected_cols {
            out.insert(
                col.clone(),
                row.get(col).cloned().unwrap_or(RuntimeValue::Null),
            );
        }
        result.push(row_to_map(&out));
    }
    Ok(result)
}

fn load_state(path: &str) -> Result<DbState, CompilerError> {
    if path == ":memory:" {
        return Ok(DbState {
            path: path.to_string(),
            tables: HashMap::new(),
        });
    }
    if !fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
        return Ok(DbState {
            path: path.to_string(),
            tables: HashMap::new(),
        });
    }
    let raw = fs::read_to_string(path).map_err(|e| CompilerError::Io(e))?;
    if raw.trim().is_empty() {
        return Ok(DbState {
            path: path.to_string(),
            tables: HashMap::new(),
        });
    }
    // Minimal JSON persistence: { "table": { "columns": [...], "rows": [...] } }
    // For bootstrap, start empty if file isn't our format.
    Ok(DbState {
        path: path.to_string(),
        tables: HashMap::new(),
    })
}

fn save_state(state: &DbState) -> Result<(), CompilerError> {
    if state.path == ":memory:" {
        return Ok(());
    }
    // Placeholder: tables live in memory for this session; file path reserves the API.
    let _ = &state.tables;
    Ok(())
}

fn db_open(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    if args.len() != 1 {
        return Err(CompilerError::ArityMismatch {
            name: "db.open".into(),
            expected: 1,
            found: args.len(),
            line,
            column,
        });
    }
    let path = match &args[0] {
        RuntimeValue::String(s) => s.clone(),
        other => {
            return Err(CompilerError::TypeMismatch {
                operation: "db.open()".into(),
                expected: "String".into(),
                found: other.type_name().to_string(),
                line,
                column,
            });
        }
    };
    let state = load_state(&path)?;
    Ok(RuntimeValue::DbConnection(Arc::new(Mutex::new(state))))
}

fn db_execute(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    let conn = require_connection(&args, "db.execute", line, column)?;
    let sql = require_sql(&args, "db.execute", line, column)?;
    let mut guard = conn.lock().unwrap();
    let rows = execute_sql(&mut guard, &sql, line, column)?;
    Ok(RuntimeValue::Integer(rows))
}

fn db_query(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    let conn = require_connection(&args, "db.query", line, column)?;
    let sql = require_sql(&args, "db.query", line, column)?;
    let guard = conn.lock().unwrap();
    let rows = query_sql(&guard, &sql, line, column)?;
    Ok(RuntimeValue::Array(Rc::new(RefCell::new(rows))))
}

fn db_close(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    let conn = require_connection(&args, "db.close", line, column)?;
    let guard = conn.lock().unwrap();
    save_state(&guard)?;
    Ok(RuntimeValue::Null)
}
