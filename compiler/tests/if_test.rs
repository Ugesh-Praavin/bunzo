//! Integration tests for `if` / `else` control flow in Bunzo.
//!
//! Tests are organised by category:
//!   - Parser: structure, errors
//!   - Runtime: execution paths, scoping, else-if chaining
//!   - Semantic: undefined variables inside branches

use bzc::ast::*;
use bzc::diagnostics::CompilerError;
use bzc::lexer;
use bzc::parser::parse;
use bzc::runtime::eval::Interpreter;
use bzc::semantic::analyze;

// ── Helpers ───────────────────────────────────────────────────────────────

/// Lex + parse source, return the Program.
fn parse_source(src: &str) -> Result<Program, CompilerError> {
    let tokens = lexer::tokenize(src)?;
    parse(tokens)
}

/// Parse + semantic-check source.
fn analyze_source(src: &str) -> Result<(), CompilerError> {
    let program = parse_source(src)?;
    analyze(&program)
}

/// Parse, semantic-check, and execute source; capture stdout.
fn run_source(src: &str) -> Result<String, CompilerError> {
    let program = parse_source(src)?;
    analyze(&program)?;
    let mut buf = Vec::new();
    let mut interp = Interpreter::new(&mut buf);
    interp.interpret(program)?;
    Ok(String::from_utf8(buf).expect("invalid utf-8"))
}

/// Assert source runs and stdout equals `expected`.
fn assert_output(src: &str, expected: &str) {
    let out = run_source(src).unwrap_or_else(|e| panic!("run failed: {e}"));
    assert_eq!(out, expected, "source:\n{src}");
}

/// Assert source execution fails with a runtime/semantic error whose
/// Display contains `code`.
fn assert_error(src: &str, code: &str) {
    // Try semantic first, then runtime
    let result = run_source(src);
    assert!(result.is_err(), "expected error for:\n{src}");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains(code), "expected '{code}' in:\n{msg}");
}

/// Assert parsing fails with an error code.
fn assert_parse_error(src: &str, code: &str) {
    let result = parse_source(src);
    assert!(result.is_err(), "expected parse error for:\n{src}");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains(code), "expected '{code}' in:\n{msg}");
}

// ══════════════════════════════════════════════════════════════════════════
// PARSER — Structure (10 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_if_no_else() {
    let prog = parse_source("if true { }").unwrap();
    assert_eq!(prog.statements.len(), 1);
    assert!(matches!(
        prog.statements[0],
        Statement::IfStatement {
            else_branch: None,
            ..
        }
    ));
}

#[test]
fn parse_if_with_else() {
    let prog = parse_source("if true { } else { }").unwrap();
    assert!(matches!(
        prog.statements[0],
        Statement::IfStatement {
            else_branch: Some(_),
            ..
        }
    ));
}

#[test]
fn parse_if_body_has_statements() {
    let prog = parse_source("if true { let x = 1 }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement { then_branch, .. } => {
            assert_eq!(then_branch.statements.len(), 1);
        }
        other => panic!("expected IfStatement, got {other:?}"),
    }
}

#[test]
fn parse_else_body_has_statements() {
    let prog = parse_source("if false { } else { let y = 2 }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement {
            else_branch: Some(blk),
            ..
        } => {
            assert_eq!(blk.statements.len(), 1);
        }
        other => panic!("expected IfStatement with else, got {other:?}"),
    }
}

#[test]
fn parse_else_if_chain() {
    // else if is represented as else { if ... }
    let prog = parse_source("if false { } else if true { }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement {
            else_branch: Some(blk),
            ..
        } => {
            assert_eq!(blk.statements.len(), 1);
            assert!(matches!(blk.statements[0], Statement::IfStatement { .. }));
        }
        other => panic!("expected else-if structure, got {other:?}"),
    }
}

#[test]
fn parse_nested_if_in_then_branch() {
    let prog = parse_source("if true { if false { } }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement { then_branch, .. } => {
            assert!(matches!(
                then_branch.statements[0],
                Statement::IfStatement { .. }
            ));
        }
        other => panic!("expected nested IfStatement, got {other:?}"),
    }
}

#[test]
fn parse_if_condition_is_expression() {
    let prog = parse_source("if 1 == 1 { }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement { condition, .. } => {
            assert!(matches!(
                condition,
                Expression::BinaryOp {
                    operator: BinaryOperator::Equal,
                    ..
                }
            ));
        }
        other => panic!("expected IfStatement, got {other:?}"),
    }
}

#[test]
fn parse_if_line_column_tracking() {
    let prog = parse_source("if true { }").unwrap();
    match &prog.statements[0] {
        Statement::IfStatement { line, column, .. } => {
            assert_eq!(*line, 1);
            assert_eq!(*column, 1);
        }
        other => panic!("expected IfStatement, got {other:?}"),
    }
}

#[test]
fn parse_error_if_missing_brace() {
    assert_parse_error("if true let x = 1", "BZ0006");
}

#[test]
fn parse_error_if_missing_closing_brace() {
    assert_parse_error("if true { let x = 1", "BZ0006");
}

// ══════════════════════════════════════════════════════════════════════════
// RUNTIME — Basic Execution (8 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn if_true_runs_then_branch() {
    assert_output("if true { print(\"yes\") }", "yes\n");
}

#[test]
fn if_false_skips_then_branch() {
    assert_output("if false { print(\"no\") }", "");
}

#[test]
fn if_else_true_runs_then() {
    assert_output(
        "if true { print(\"then\") } else { print(\"else\") }",
        "then\n",
    );
}

#[test]
fn if_else_false_runs_else() {
    assert_output(
        "if false { print(\"then\") } else { print(\"else\") }",
        "else\n",
    );
}

#[test]
fn if_with_comparison_condition_true() {
    assert_output("let x = 10\nif x > 5 { print(\"big\") }", "big\n");
}

#[test]
fn if_with_comparison_condition_false() {
    assert_output(
        "let x = 3\nif x > 5 { print(\"big\") } else { print(\"small\") }",
        "small\n",
    );
}

#[test]
fn if_multiple_prints_in_body() {
    assert_output(
        "if true { print(\"a\") print(\"b\") print(\"c\") }",
        "a\nb\nc\n",
    );
}

#[test]
fn if_condition_variable() {
    assert_output(
        "let flag = true\nif flag { print(\"on\") } else { print(\"off\") }",
        "on\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// RUNTIME — else if Chaining (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn else_if_first_branch_taken() {
    assert_output(
        "let x = 1\nif x == 1 { print(\"one\") } else if x == 2 { print(\"two\") } else { print(\"other\") }",
        "one\n",
    );
}

#[test]
fn else_if_second_branch_taken() {
    assert_output(
        "let x = 2\nif x == 1 { print(\"one\") } else if x == 2 { print(\"two\") } else { print(\"other\") }",
        "two\n",
    );
}

#[test]
fn else_if_fallthrough_to_else() {
    assert_output(
        "let x = 99\nif x == 1 { print(\"one\") } else if x == 2 { print(\"two\") } else { print(\"other\") }",
        "other\n",
    );
}

#[test]
fn else_if_no_final_else_skips_when_no_match() {
    assert_output(
        "let x = 5\nif x == 1 { print(\"one\") } else if x == 2 { print(\"two\") }",
        "",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// RUNTIME — Scoping (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn outer_variable_readable_inside_if() {
    assert_output("let msg = \"hello\"\nif true { print(msg) }", "hello\n");
}

#[test]
fn outer_variable_readable_inside_else() {
    assert_output("let msg = \"hi\"\nif false { } else { print(msg) }", "hi\n");
}

#[test]
fn variable_inside_if_not_visible_after() {
    // Accessing `inner` after the if block should produce UndefinedVariable.
    let result = run_source("if true { let inner = 42 }\nprint(inner)");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("BZ1001"), "expected BZ1001, got: {msg}");
}

#[test]
fn variable_inside_else_not_visible_after() {
    let result = run_source("if false { } else { let x = 1 }\nprint(x)");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("BZ1001"), "expected BZ1001, got: {msg}");
}

#[test]
fn nested_if_accesses_outer_scope() {
    assert_output(
        "let x = 10\nif true { if x > 5 { print(\"big\") } }",
        "big\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// RUNTIME — Error Cases (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn non_boolean_integer_condition_runtime_error() {
    // Semantic analysis passes (no type checking at compile time),
    // but runtime raises TypeMismatch.
    let tokens = lexer::tokenize("if 42 { }").unwrap();
    let program = parse(tokens).unwrap();
    // Skip semantic analysis (it doesn't check types).
    let mut buf = Vec::new();
    let mut interp = Interpreter::new(&mut buf);
    let err = interp.interpret(program).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("BZ1003"), "expected BZ1003 in: {msg}");
}

#[test]
fn non_boolean_string_condition_runtime_error() {
    let tokens = lexer::tokenize("if \"hello\" { }").unwrap();
    let program = parse(tokens).unwrap();
    let mut buf = Vec::new();
    let mut interp = Interpreter::new(&mut buf);
    let err = interp.interpret(program).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("BZ1003"), "expected BZ1003 in: {msg}");
}

#[test]
fn non_boolean_null_condition_runtime_error() {
    let tokens = lexer::tokenize("if null { }").unwrap();
    let program = parse(tokens).unwrap();
    let mut buf = Vec::new();
    let mut interp = Interpreter::new(&mut buf);
    let err = interp.interpret(program).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("BZ1003"), "expected BZ1003 in: {msg}");
}

#[test]
fn duplicate_declaration_inside_if_error() {
    assert_error("if true { let x = 1 let x = 2 }", "BZ1005");
}

// ══════════════════════════════════════════════════════════════════════════
// SEMANTIC — Undefined Variables (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn semantic_error_undefined_in_condition() {
    let result = analyze_source("if undefined_var { }");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("BZ1001"), "expected BZ1001 in: {msg}");
}

#[test]
fn semantic_error_undefined_in_then_branch() {
    let result = analyze_source("if true { print(x) }");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("BZ1001"), "expected BZ1001 in: {msg}");
}

#[test]
fn semantic_error_undefined_in_else_branch() {
    let result = analyze_source("if false { } else { print(x) }");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("BZ1001"), "expected BZ1001 in: {msg}");
}

#[test]
fn semantic_ok_variable_declared_before_if_used_inside() {
    assert!(analyze_source("let x = 1\nif true { print(x) }").is_ok());
}
