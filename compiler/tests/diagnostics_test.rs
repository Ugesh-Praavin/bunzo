use std::path::PathBuf;

use bzc::diagnostics::CompilerError;

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
    assert!(
        message.contains("column 12"),
        "should contain column number"
    );
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
    assert!(
        message.contains("Expected expression"),
        "should contain message"
    );
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
