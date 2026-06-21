//! Integration tests for loops (`while`, `for`, `break`, `continue`) in Bunzo.

use bzc::ast::*;
use bzc::diagnostics::CompilerError;
use bzc::lexer;
use bzc::parser::parse;
use bzc::runtime::eval::Interpreter;
use bzc::semantic::analyze;

// ── Helpers ───────────────────────────────────────────────────────────────

fn parse_source(src: &str) -> Result<Program, CompilerError> {
    let tokens = lexer::tokenize(src)?;
    parse(tokens)
}

fn run_source(src: &str) -> Result<String, CompilerError> {
    let program = parse_source(src)?;
    analyze(&program)?;
    let mut buf = Vec::new();
    let mut interp = Interpreter::new(&mut buf);
    interp.interpret(program)?;
    Ok(String::from_utf8(buf).expect("invalid utf-8"))
}

fn assert_output(src: &str, expected: &str) {
    let out = run_source(src).unwrap_or_else(|e| panic!("run failed: {e}"));
    assert_eq!(out, expected, "source:\n{src}");
}

fn assert_error(src: &str, code: &str) {
    let result = run_source(src);
    assert!(result.is_err(), "expected error for:\n{src}");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains(code), "expected '{code}' in:\n{msg}");
}

fn assert_parse_error(src: &str, code: &str) {
    let result = parse_source(src);
    assert!(result.is_err(), "expected parse error for:\n{src}");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains(code), "expected '{code}' in:\n{msg}");
}

// ══════════════════════════════════════════════════════════════════════════
// WHILE - Parser Tests (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_while_empty() {
    let prog = parse_source("while true { }").unwrap();
    assert_eq!(prog.statements.len(), 1);
    assert!(matches!(
        prog.statements[0],
        Statement::WhileStatement { .. }
    ));
}

#[test]
fn parse_while_with_body() {
    let prog = parse_source("while count < 10 { print(count) }").unwrap();
    assert_eq!(prog.statements.len(), 1);
    if let Statement::WhileStatement {
        condition, body, ..
    } = &prog.statements[0]
    {
        assert!(matches!(condition, Expression::BinaryOp { .. }));
        assert_eq!(body.len(), 1);
    } else {
        panic!("Expected WhileStatement");
    }
}

#[test]
fn parse_break() {
    let prog = parse_source("while true { break }").unwrap();
    if let Statement::WhileStatement { body, .. } = &prog.statements[0] {
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Statement::BreakStatement { .. }));
    } else {
        panic!("Expected WhileStatement");
    }
}

#[test]
fn parse_continue() {
    let prog = parse_source("while true { continue }").unwrap();
    if let Statement::WhileStatement { body, .. } = &prog.statements[0] {
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Statement::ContinueStatement { .. }));
    } else {
        panic!("Expected WhileStatement");
    }
}

#[test]
fn parse_while_nested() {
    let prog = parse_source("while true { while false { break } }").unwrap();
    assert_eq!(prog.statements.len(), 1);
}

// ══════════════════════════════════════════════════════════════════════════
// WHILE - Runtime Tests (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn run_while_false() {
    assert_output("let x = 1\nwhile false { x = 2 }\nprint(x)", "1\n");
}

#[test]
fn run_while_countdown() {
    assert_output(
        "let x = 3\nwhile x > 0 {\n  print(x)\n  x = x - 1\n}",
        "3\n2\n1\n",
    );
}

#[test]
fn run_while_nested_loops() {
    assert_output(
        "let i = 1\nwhile i <= 2 {\n  let j = 1\n  while j <= 2 {\n    print(i * 10 + j)\n    j = j + 1\n  }\n  i = i + 1\n}",
        "11\n12\n21\n22\n",
    );
}

#[test]
fn run_while_mutate_outer_var() {
    assert_output(
        "let sum = 0\nlet i = 1\nwhile i <= 4 {\n  sum = sum + i\n  i = i + 1\n}\nprint(sum)",
        "10\n",
    );
}

#[test]
fn run_while_with_complex_condition() {
    assert_output(
        "let x = 0\nlet y = 0\nwhile x < 2 && y < 3 {\n  print(x)\n  x = x + 1\n  y = y + 1\n}",
        "0\n1\n",
    );
}

#[test]
fn run_while_zero_iterations() {
    assert_output(
        "let flag = false\nwhile flag {\n  print(42)\n}\nprint(99)",
        "99\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// WHILE - Break / Continue Tests (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn run_while_break_immediately() {
    assert_output("while true { break\nprint(1) }\nprint(2)", "2\n");
}

#[test]
fn run_while_break_conditional() {
    assert_output(
        "let i = 0\nwhile true {\n  i = i + 1\n  if i == 3 { break }\n  print(i)\n}",
        "1\n2\n",
    );
}

#[test]
fn run_while_continue() {
    assert_output(
        "let i = 0\nwhile i < 4 {\n  i = i + 1\n  if i == 2 { continue }\n  print(i)\n}",
        "1\n3\n4\n",
    );
}

#[test]
fn run_while_nested_break_continue() {
    assert_output(
        "let i = 0\nwhile i < 3 {\n  i = i + 1\n  let j = 0\n  while true {\n    j = j + 1\n    if j == 2 { continue }\n    if j == 3 { break }\n    print(i * 10 + j)\n  }\n}",
        "11\n21\n31\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// FOR - Parser Tests (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_for_empty() {
    let prog = parse_source("for i in 1..5 { }").unwrap();
    assert_eq!(prog.statements.len(), 1);
    assert!(matches!(
        prog.statements[0],
        Statement::ForStatement { .. }
    ));
}

#[test]
fn parse_for_with_body() {
    let prog = parse_source("for x in 0..10 { print(x) }").unwrap();
    assert_eq!(prog.statements.len(), 1);
    if let Statement::ForStatement {
        variable,
        start,
        end,
        body,
        ..
    } = &prog.statements[0]
    {
        assert_eq!(variable, "x");
        assert!(matches!(start, Expression::IntegerLiteral { value: 0, .. }));
        assert!(matches!(end, Expression::IntegerLiteral { .. }));
        assert_eq!(body.len(), 1);
    } else {
        panic!("Expected ForStatement");
    }
}

#[test]
fn parse_for_nested() {
    let prog = parse_source("for i in 1..3 { for j in 1..3 { print(i) } }").unwrap();
    assert_eq!(prog.statements.len(), 1);
}

#[test]
fn parse_for_expressions_in_range() {
    let prog = parse_source("for i in (a+1)..(b*2) { }").unwrap();
    assert_eq!(prog.statements.len(), 1);
}

#[test]
fn parse_for_invalid_range_symbol() {
    assert_parse_error("for i in 1...5 { }", "BZ0007");
}

// ══════════════════════════════════════════════════════════════════════════
// FOR - Runtime Tests (7 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn run_for_simple() {
    assert_output("for i in 1..4 {\n  print(i)\n}", "1\n2\n3\n");
}

#[test]
fn run_for_empty_range() {
    assert_output("for i in 5..5 {\n  print(i)\n}\nprint(9)", "9\n");
}

#[test]
fn run_for_reversed_range_zero_iterations() {
    assert_output("for i in 5..2 {\n  print(i)\n}\nprint(0)", "0\n");
}

#[test]
fn run_for_expression_bounds() {
    assert_output(
        "let start = 2\nlet end = 5\nfor i in (start - 1)..(end + 1) {\n  print(i)\n}",
        "1\n2\n3\n4\n5\n",
    );
}

#[test]
fn run_for_variable_scoping() {
    assert_output(
        "let i = 42\nfor i in 1..3 {\n  print(i)\n}\nprint(i)",
        "1\n2\n42\n",
    );
}

#[test]
fn run_for_mutate_outer_variable() {
    assert_output(
        "let sum = 0\nfor i in 1..5 {\n  sum = sum + i\n}\nprint(sum)",
        "10\n",
    );
}

#[test]
fn run_for_nested_loops() {
    assert_output(
        "for i in 1..3 {\n  for j in 1..3 {\n    print(i * 10 + j)\n  }\n}",
        "11\n12\n21\n22\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// FOR - Break / Continue Tests (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn run_for_break() {
    assert_output(
        "for i in 1..5 {\n  if i == 3 { break }\n  print(i)\n}",
        "1\n2\n",
    );
}

#[test]
fn run_for_continue() {
    assert_output(
        "for i in 1..5 {\n  if i == 3 { continue }\n  print(i)\n}",
        "1\n2\n4\n",
    );
}

#[test]
fn run_for_break_with_outer_var() {
    assert_output(
        "let last = 0\nfor i in 1..10 {\n  last = i\n  if i == 4 { break }\n}\nprint(last)",
        "4\n",
    );
}

#[test]
fn run_for_nested_break() {
    assert_output(
        "for i in 1..3 {\n  for j in 1..5 {\n    if j == 3 { break }\n    print(i * 10 + j)\n  }\n}",
        "11\n12\n21\n22\n",
    );
}

// ══════════════════════════════════════════════════════════════════════════
// SEMANTIC - Scope & Error Tests (9 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn semantic_undefined_var_in_while_condition() {
    assert_error("while undefined_var { }", "BZ1001");
}

#[test]
fn semantic_undefined_var_in_while_body() {
    assert_error("while true { print(undefined_var) }", "BZ1001");
}

#[test]
fn semantic_undefined_var_in_for_iterable() {
    assert_error("for i in undefined_var..10 { }", "BZ1001");
}

#[test]
fn semantic_undefined_var_in_for_body() {
    assert_error("for i in 1..10 { print(undefined_var) }", "BZ1001");
}

#[test]
fn semantic_const_reassignment_in_while() {
    assert_error("const c = 10\nwhile true { c = 20 }", "BZ1002");
}

#[test]
fn semantic_const_reassignment_in_for() {
    assert_error("const c = 10\nfor i in 1..5 { c = 20 }", "BZ1002");
}

#[test]
fn semantic_loop_var_not_visible_after_for() {
    assert_error("for x in 1..5 { }\nprint(x)", "BZ1001");
}

#[test]
fn semantic_duplicate_declaration_in_for_body() {
    assert_error("for i in 1..5 { let i = 10 }", "BZ1005");
}

#[test]
fn run_error_non_boolean_while_condition() {
    assert_error("while 42 { }", "BZ1003");
}

#[test]
fn run_error_non_integer_range_bounds() {
    assert_error("for i in 1.5..5 { }", "BZ1003");
    assert_error("for i in 1..\"5\" { }", "BZ1003");
}
