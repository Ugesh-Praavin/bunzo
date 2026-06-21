//! End-to-end integration tests for structs (declarations, literals, field access).
//!
//! These run the full pipeline (lexer -> parser -> semantic -> runtime)
//! against small Bunzo source snippets.

use bzc::lexer;
use bzc::parser;
use bzc::runtime;
use bzc::semantic;

/// Runs a Bunzo source string through the full pipeline and returns
/// everything written by `print` statements.
fn run_source(source: &str) -> Result<String, String> {
    let tokens = lexer::tokenize(source).map_err(|e| e.to_string())?;
    let program = parser::parse(tokens).map_err(|e| e.to_string())?;
    semantic::analyze(&program).map_err(|e| e.to_string())?;

    let mut buffer = Vec::new();
    {
        let mut interpreter = runtime::eval::Interpreter::new(&mut buffer);
        interpreter.interpret(program).map_err(|e| e.to_string())?;
    }
    Ok(String::from_utf8(buffer).expect("invalid utf-8 output"))
}

#[test]
fn test_struct_literal_and_field_access() {
    let out = run_source(
        r#"
        struct User {
            id: int
            name: string
        }
        let user = User {
            id: 1,
            name: "Bart"
        }
        print(user.id)
        print(user.name)
        "#,
    )
    .unwrap();
    assert_eq!(out, "1\nBart\n");
}

#[test]
fn test_print_struct_itself() {
    let out = run_source(
        r#"
        struct User {
            id: int
            name: string
        }
        let user = User {
            id: 1,
            name: "Bart"
        }
        print(user)
        "#,
    )
    .unwrap();
    assert_eq!(out, "User { id: 1, name: Bart }\n");
}

#[test]
fn test_unknown_struct_error() {
    let err = run_source(
        r#"
        let user = User { id: 1 }
        "#,
    )
    .unwrap_err();
    assert!(err.contains("BZ1012")); // Unknown struct type
}

#[test]
fn test_struct_field_mismatch_error() {
    let err = run_source(
        r#"
        struct User {
            id: int
            name: string
        }
        let user = User { id: 1 }
        "#,
    )
    .unwrap_err();
    assert!(err.contains("BZ1013")); // Struct field mismatch
}

#[test]
fn test_no_such_field_error() {
    let err = run_source(
        r#"
        struct User {
            id: int
            name: string
        }
        let user = User {
            id: 1,
            name: "Bart"
        }
        print(user.nonexistent)
        "#,
    )
    .unwrap_err();
    assert!(err.contains("BZ1014")); // Struct "User" has no field "nonexistent"
}

#[test]
fn test_field_access_on_non_struct_error() {
    let err = run_source(
        r#"
        let x = 42
        print(x.name)
        "#,
    )
    .unwrap_err();
    assert!(err.contains("BZ1003")); // Type mismatch during field access
}
