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
                write!(
                    f,
                    "error[BZ0001]\n\nFile not found:\n{}",
                    path.display(),
                )
            }
            CompilerError::Io(err) => {
                write!(f, "error[BZ0002]\n\nI/O error:\n{err}")
            }
            CompilerError::UnexpectedCharacter { character, line, column } => {
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
            CompilerError::UnexpectedToken { expected, found, line, column } => {
                write!(
                    f,
                    "error[BZ0006]\n\nUnexpected token: expected {expected}, found {found}\n  --> line {line}, column {column}",
                )
            }
            CompilerError::ExpectedExpression { found, line, column } => {
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
            CompilerError::TypeMismatch { operation, expected, found, line, column } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn display_file_not_found() {
        let err = CompilerError::FileNotFound(PathBuf::from("missing.bz"));
        let message = format!("{err}");

        assert!(message.contains("BZ0001"), "should contain error code");
        assert!(message.contains("missing.bz"), "should contain file path");
    }

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = CompilerError::Io(io_err);
        let message = format!("{err}");

        assert!(message.contains("BZ0002"), "should contain error code");
        assert!(message.contains("access denied"), "should contain cause");
    }

    #[test]
    fn display_unexpected_character() {
        let err = CompilerError::UnexpectedCharacter {
            character: '@',
            line: 5,
            column: 12,
        };
        let message = format!("{err}");

        assert!(message.contains("BZ0003"), "should contain error code");
        assert!(message.contains('@'), "should contain the character");
        assert!(message.contains("line 5"), "should contain line number");
        assert!(message.contains("column 12"), "should contain column number");
    }

    #[test]
    fn display_unterminated_string() {
        let err = CompilerError::UnterminatedString { line: 3, column: 8 };
        let message = format!("{err}");

        assert!(message.contains("BZ0004"), "should contain error code");
        assert!(message.contains("line 3"), "should contain line number");
        assert!(message.contains("Hint"), "should contain a fix suggestion");
    }

    #[test]
    fn display_unterminated_comment() {
        let err = CompilerError::UnterminatedComment { line: 1, column: 1 };
        let message = format!("{err}");

        assert!(message.contains("BZ0005"), "should contain error code");
        assert!(message.contains("line 1"), "should contain line number");
        assert!(message.contains("*/"), "should suggest closing delimiter");
    }

    #[test]
    fn display_unexpected_token() {
        let err = CompilerError::UnexpectedToken {
            expected: "variable name".to_string(),
            found: "'='".to_string(),
            line: 1,
            column: 5,
        };
        let message = format!("{err}");

        assert!(message.contains("BZ0006"), "should contain error code");
        assert!(message.contains("variable name"), "should contain expected");
        assert!(message.contains("'='"), "should contain found");
        assert!(message.contains("line 1"), "should contain line number");
    }

    #[test]
    fn display_expected_expression() {
        let err = CompilerError::ExpectedExpression {
            found: "'}'".to_string(),
            line: 2,
            column: 3,
        };
        let message = format!("{err}");

        assert!(message.contains("BZ0007"), "should contain error code");
        assert!(message.contains("Expected expression"), "should contain message");
        assert!(message.contains("'}'"), "should contain found token");
    }

    #[test]
    fn display_undefined_variable() {
        let err = CompilerError::UndefinedVariable {
            name: "x".to_string(),
            line: 4,
            column: 10,
        };
        let message = format!("{err}");
        assert!(message.contains("BZ1001"));
        assert!(message.contains("Undefined variable \"x\""));
        assert!(message.contains("line 4"));
    }

    #[test]
    fn display_const_reassignment() {
        let err = CompilerError::ConstReassignment {
            name: "PI".to_string(),
            line: 2,
            column: 3,
        };
        let message = format!("{err}");
        assert!(message.contains("BZ1002"));
        assert!(message.contains("Cannot reassign to constant variable \"PI\""));
    }

    #[test]
    fn display_type_mismatch() {
        let err = CompilerError::TypeMismatch {
            operation: "addition".to_string(),
            expected: "number".to_string(),
            found: "String".to_string(),
            line: 10,
            column: 5,
        };
        let message = format!("{err}");
        assert!(message.contains("BZ1003"));
        assert!(message.contains("Type mismatch during addition"));
    }

    #[test]
    fn display_division_by_zero() {
        let err = CompilerError::DivisionByZero {
            line: 5,
            column: 12,
        };
        let message = format!("{err}");
        assert!(message.contains("BZ1004"));
        assert!(message.contains("Division by zero"));
    }

    #[test]
    fn display_duplicate_declaration() {
        let err = CompilerError::DuplicateDeclaration {
            name: "y".to_string(),
            line: 3,
            column: 1,
        };
        let message = format!("{err}");
        assert!(message.contains("BZ1005"));
        assert!(message.contains("Duplicate declaration of variable \"y\""));
    }
}
