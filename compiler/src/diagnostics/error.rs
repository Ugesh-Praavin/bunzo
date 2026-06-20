//! Compiler diagnostics and error reporting.
//!
//! This module defines structured error types for the Bunzo compiler.
//! Every compiler error carries an error code (e.g. `BZ0001`) and a
//! human-readable message, following the diagnostic format described
//! in the Bunzo architecture documentation.

use std::fmt;
use std::path::PathBuf;

/// Represents a compiler error that can occur during any stage of compilation.
///
/// Each variant maps to a specific error code:
///
/// | Code     | Variant        | Description              |
/// |----------|----------------|--------------------------|
/// | `BZ0001` | `FileNotFound` | Source file does not exist |
/// | `BZ0002` | `Io`           | General I/O failure       |
#[derive(Debug)]
pub enum CompilerError {
    /// The requested source file was not found on disk.
    FileNotFound(PathBuf),

    /// A general I/O error occurred while reading a source file.
    Io(std::io::Error),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::FileNotFound(path) => {
                write!(
                    f,
                    "error[BZ0001]\n\nFile not found:\n{}",
                    path.display(),
                )
            }
            CompilerError::Io(err) => {
                write!(f, "error[BZ0002]\n\nI/O error:\n{err}")
            }
        }
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompilerError::FileNotFound(_) => None,
            CompilerError::Io(err) => Some(err),
        }
    }
}
