//! Compiler diagnostics and error reporting.
//!
//! This module defines structured error types for the Bunzo compiler.
//! Every compiler error carries an error code (e.g. `BZ0001`) and a
//! human-readable message with source location, following the diagnostic
//! format described in the Bunzo architecture documentation.

use std::fmt;
use std::path::PathBuf;

/// Represents a compiler error that can occur during any stage of compilation.
///
/// Each variant maps to a specific error code:
///
/// | Code     | Variant                | Description                        |
/// |----------|------------------------|------------------------------------|
/// | `BZ0001` | `FileNotFound`         | Source file does not exist          |
/// | `BZ0002` | `Io`                   | General I/O failure                |
/// | `BZ0003` | `UnexpectedCharacter`  | Unrecognized character in source   |
/// | `BZ0004` | `UnterminatedString`   | String literal missing closing `"` |
/// | `BZ0005` | `UnterminatedComment`  | Block comment missing closing `*/` |
/// | `BZ0006` | `UnexpectedToken`      | Parser encountered unexpected token |
/// | `BZ0007` | `ExpectedExpression`   | Parser expected an expression       |
/// | `BZ1001` | `UndefinedVariable`    | Reference to an undefined variable |
/// | `BZ1002` | `ConstReassignment`    | Attempt to reassign a constant     |
/// | `BZ1003` | `TypeMismatch`         | Operand type mismatch on operation |
/// | `BZ1004` | `DivisionByZero`       | Division or modulo by zero         |
/// | `BZ1005` | `DuplicateDeclaration` | Variable declared twice in scope   |
#[derive(Debug)]
pub enum CompilerError {
    /// The requested source file was not found on disk.
    FileNotFound(PathBuf),

    /// A general I/O error occurred while reading a source file.
    Io(std::io::Error),

    /// An unrecognized character was encountered during lexing.
    UnexpectedCharacter {
        /// The unexpected character.
        character: char,
        /// The 1-based line number where the character was found.
        line: usize,
        /// The 1-based column number where the character was found.
        column: usize,
    },

    /// A string literal is missing its closing double-quote.
    UnterminatedString {
        /// The 1-based line number of the opening `"`.
        line: usize,
        /// The 1-based column number of the opening `"`.
        column: usize,
    },

    /// A block comment (`/* ... */`) is missing its closing delimiter.
    UnterminatedComment {
        /// The 1-based line number of the opening `/*`.
        line: usize,
        /// The 1-based column number of the opening `/*`.
        column: usize,
    },

    /// The parser encountered a token it did not expect.
    UnexpectedToken {
        /// Human-readable description of what was expected.
        expected: String,
        /// Human-readable description of what was found.
        found: String,
        /// The 1-based line number of the unexpected token.
        line: usize,
        /// The 1-based column number of the unexpected token.
        column: usize,
    },

    /// The parser expected an expression but found something else.
    ExpectedExpression {
        /// Human-readable description of what was found.
        found: String,
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },

    /// Reference to an undefined variable (BZ1001).
    UndefinedVariable {
        /// The name of the variable.
        name: String,
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },

    /// Attempt to reassign a constant variable (BZ1002).
    ConstReassignment {
        /// The name of the constant variable.
        name: String,
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },

    /// Operand type mismatch during evaluation (BZ1003).
    TypeMismatch {
        /// The operation name or symbol.
        operation: String,
        /// The expected type(s).
        expected: String,
        /// The type that was actually found.
        found: String,
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },

    /// Division or modulo by zero (BZ1004).
    DivisionByZero {
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },

    /// Variable declared twice in the same scope (BZ1005).
    DuplicateDeclaration {
        /// The name of the variable.
        name: String,
        /// The 1-based line number.
        line: usize,
        /// The 1-based column number.
        column: usize,
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::FileNotFound(path) => {
                write!(f, "error[BZ0001]\n\nFile not found:\n{}", path.display(),)
            }
            CompilerError::Io(err) => {
                write!(f, "error[BZ0002]\n\nI/O error:\n{err}")
            }
            CompilerError::UnexpectedCharacter {
                character,
                line,
                column,
            } => {
                write!(
                    f,
                    "error[BZ0003]\n\nUnexpected character: '{character}'\n  --> line {line}, column {column}",
                )
            }
            CompilerError::UnterminatedString { line, column } => {
                write!(
                    f,
                    "error[BZ0004]\n\nUnterminated string literal\n  --> line {line}, column {column}\n\nHint: add a closing '\"' to complete the string.",
                )
            }
            CompilerError::UnterminatedComment { line, column } => {
                write!(
                    f,
                    "error[BZ0005]\n\nUnterminated block comment\n  --> line {line}, column {column}\n\nHint: add '*/' to close the comment.",
                )
            }
            CompilerError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "error[BZ0006]\n\nUnexpected token: expected {expected}, found {found}\n  --> line {line}, column {column}",
                )
            }
            CompilerError::ExpectedExpression {
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "error[BZ0007]\n\nExpected expression, found {found}\n  --> line {line}, column {column}",
                )
            }
            CompilerError::UndefinedVariable { name, line, column } => {
                write!(
                    f,
                    "error[BZ1001]\n\nUndefined variable \"{name}\"\n  --> line {line}, column {column}\n\nHint: declare the variable using 'let' or 'const' before using it.",
                )
            }
            CompilerError::ConstReassignment { name, line, column } => {
                write!(
                    f,
                    "error[BZ1002]\n\nCannot reassign to constant variable \"{name}\"\n  --> line {line}, column {column}\n\nHint: constant variables declared with 'const' cannot be reassigned. Use 'let' if mutability is required.",
                )
            }
            CompilerError::TypeMismatch {
                operation,
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "error[BZ1003]\n\nType mismatch during {operation}: expected {expected}, found {found}\n  --> line {line}, column {column}",
                )
            }
            CompilerError::DivisionByZero { line, column } => {
                write!(
                    f,
                    "error[BZ1004]\n\nDivision by zero\n  --> line {line}, column {column}",
                )
            }
            CompilerError::DuplicateDeclaration { name, line, column } => {
                write!(
                    f,
                    "error[BZ1005]\n\nDuplicate declaration of variable \"{name}\"\n  --> line {line}, column {column}\n\nHint: variable \"{name}\" has already been declared in this scope. Use assignment '=' to update its value, or use a different name.",
                )
            }
        }
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompilerError::Io(err) => Some(err),
            _ => None,
        }
    }
}
