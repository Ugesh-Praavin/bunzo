//! Source file reading and management.
//!
//! This module handles reading Bunzo source files (`.bz`) from disk,
//! performing UTF-8 validation, and returning the source text for
//! downstream compiler stages.
//!
//! No output is produced by this module; it returns results to the caller.

use std::path::Path;

use crate::diagnostics::CompilerError;

/// Reads a Bunzo source file and returns its contents as a string.
///
/// # Errors
///
/// Returns [`CompilerError::FileNotFound`] if the file does not exist.
/// Returns [`CompilerError::Io`] for any other I/O failure.
pub fn read_source(path: &Path) -> Result<String, CompilerError> {
    if !path.exists() {
        return Err(CompilerError::FileNotFound(path.to_path_buf()));
    }

    std::fs::read_to_string(path).map_err(CompilerError::Io)
}

/// Resolves a module name and optional path into a file path and its content.
///
/// Looks up in:
/// 1. `path.bz` (if path is specified)
/// 2. `name.bz`
/// 3. `modules/name/mod.bz`
/// 4. `modules/name.bz`
/// 5. `stdlib/name.bz`
pub fn resolve_module(
    name: &str,
    path: Option<&str>,
    line: usize,
    column: usize,
) -> Result<(String, String), CompilerError> {
    let candidates: Vec<String> = if let Some(p) = path {
        vec![if p.ends_with(".bz") {
            p.to_string()
        } else {
            format!("{p}.bz")
        }]
    } else {
        vec![
            format!("{name}.bz"),
            format!("modules/{name}/mod.bz"),
            format!("modules/{name}.bz"),
            format!("stdlib/{name}.bz"),
        ]
    };

    for file_path in &candidates {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            return Ok((file_path.clone(), content));
        }
    }

    Err(CompilerError::ModuleNotFound {
        name: name.to_string(),
        line,
        column,
    })
}
