//! End-to-end tests for control flow and assignment (if/else, while,
//! for, break, continue, `name = expr`).
//!
//! These run the full pipeline (lexer -> parser -> semantic -> runtime)
//! against small Bunzo source snippets, the same way the CLI does.

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

    // `runtime::execute` writes to stdout directly; for tests we re-run
    // the same stages against an interpreter targeting a buffer.
    let mut buffer = Vec::new();
    {
        let mut interpreter = runtime::eval::Interpreter::new(&mut buffer);
        interpreter.interpret(program).map_err(|e| e.to_string())?;
    }
    Ok(String::from_utf8(buffer).expect("invalid utf-8 output"))
}

#[test]
fn if_else_chooses_correct_branch() {
    let out = run_source(
        r#"
        if 1 < 2 {
            print("yes")
        } else {
            print("no")
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "yes\n");
}

#[test]
fn else_if_chain() {
    let out = run_source(
        r#"
        let n = 0
        if n < 0 {
            print("negative")
        } else if n == 0 {
            print("zero")
        } else {
            print("positive")
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "zero\n");
}

#[test]
fn while_loop_counts_down() {
    let out = run_source(
        r#"
        let x = 3
        while x > 0 {
            print(x)
            x = x - 1
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "3\n2\n1\n");
}

#[test]
fn for_loop_over_range_is_exclusive_of_end() {
    let out = run_source(
        r#"
        for i in 0..3 {
            print(i)
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n1\n2\n");
}

#[test]
fn break_exits_loop_early() {
    let out = run_source(
        r#"
        for i in 0..10 {
            if i == 3 {
                break
            }
            print(i)
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n1\n2\n");
}

#[test]
fn continue_skips_current_iteration() {
    let out = run_source(
        r#"
        for i in 0..5 {
            if i == 2 {
                continue
            }
            print(i)
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n1\n3\n4\n");
}

#[test]
fn assignment_mutates_existing_let_binding() {
    let out = run_source(
        r#"
        let x = 1
        x = 2
        print(x)
        "#,
    )
    .unwrap();
    assert_eq!(out, "2\n");
}

#[test]
fn assignment_to_undefined_variable_errors() {
    let err = run_source("x = 1").unwrap_err();
    assert!(err.contains("BZ1001"));
}

#[test]
fn assignment_to_const_errors() {
    let err = run_source("const x = 1\nx = 2").unwrap_err();
    assert!(err.contains("BZ1002"));
}

#[test]
fn break_outside_loop_is_rejected_at_semantic_analysis() {
    let err = run_source("break").unwrap_err();
    assert!(err.contains("BZ1010"));
}

#[test]
fn continue_outside_loop_is_rejected_at_semantic_analysis() {
    let err = run_source("continue").unwrap_err();
    assert!(err.contains("BZ1011"));
}

#[test]
fn return_inside_loop_inside_function_propagates_through_loop() {
    let out = run_source(
        r#"
        func first_even(limit: int) -> int {
            for i in 0..limit {
                if i % 2 == 0 {
                    return i
                }
            }
            return -1
        }
        print(first_even(10))
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n");
}

#[test]
fn nested_loops_break_only_innermost() {
    let out = run_source(
        r#"
        for i in 0..2 {
            for j in 0..3 {
                if j == 1 {
                    break
                }
                print(j)
            }
        }
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n0\n");
}
