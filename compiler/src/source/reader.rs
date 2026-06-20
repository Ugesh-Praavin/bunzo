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
