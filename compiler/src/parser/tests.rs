//! Parser test suite for the Bunzo compiler.
//!
//! Organized by category: variable declarations, print statements,
//! literals, binary/unary expressions, operator precedence, grouping,
//! multi-statement programs, error cases, and edge cases.

use crate::ast::*;
use crate::diagnostics::CompilerError;
use crate::lexer;

use super::parse;

// ── Test Helpers ──────────────────────────────────────────────────────────

/// Tokenizes and parses source code in one step.
fn parse_source(source: &str) -> Result<Program, CompilerError> {
    let tokens = lexer::tokenize(source)?;
    parse(tokens)
}

/// Parses source and returns the first statement (panics if none).
fn first_stmt(source: &str) -> Statement {
    let program = parse_source(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(!program.statements.is_empty(), "expected at least one statement");
    program.statements.into_iter().next().unwrap()
}

/// Parses source and returns the first statement's expression
/// (only works for ExpressionStatements).
fn first_expr(source: &str) -> Expression {
    match first_stmt(source) {
        Statement::ExpressionStatement { expression } => expression,
        other => panic!("expected ExpressionStatement, got {other:?}"),
    }
}

/// Asserts that parsing the given source fails with an error containing
/// the expected error code.
fn assert_parse_error(source: &str, expected_code: &str) {
    let result = parse_source(source);
    assert!(result.is_err(), "expected error for {source:?}");
    let message = format!("{}", result.unwrap_err());
    assert!(
        message.contains(expected_code),
        "expected {expected_code} in error: {message}"
    );
}

// ══════════════════════════════════════════════════════════════════════════
// LET DECLARATIONS (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn let_integer() {
    let stmt = first_stmt("let x = 42");
    match stmt {
        Statement::LetDeclaration { name, initializer, .. } => {
            assert_eq!(name, "x");
            assert!(matches!(initializer, Expression::IntegerLiteral { value: 42, .. }));
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

#[test]
fn let_string() {
    let stmt = first_stmt("let name = \"Bunzo\"");
    match stmt {
        Statement::LetDeclaration { name, initializer, .. } => {
            assert_eq!(name, "name");
            match initializer {
                Expression::StringLiteral { value, .. } => assert_eq!(value, "Bunzo"),
                other => panic!("expected StringLiteral, got {other:?}"),
            }
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

#[test]
fn let_float() {
    let stmt = first_stmt("let price = 99.99");
    match stmt {
        Statement::LetDeclaration { name, initializer, .. } => {
            assert_eq!(name, "price");
            assert!(matches!(initializer, Expression::FloatLiteral { .. }));
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

#[test]
fn let_boolean() {
    let stmt = first_stmt("let active = true");
    match stmt {
        Statement::LetDeclaration { name, initializer, .. } => {
            assert_eq!(name, "active");
            assert!(matches!(initializer, Expression::BooleanLiteral { value: true, .. }));
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

#[test]
fn let_with_expression() {
    let stmt = first_stmt("let sum = a + b");
    match stmt {
        Statement::LetDeclaration { name, initializer, .. } => {
            assert_eq!(name, "sum");
            assert!(matches!(initializer, Expression::BinaryOp { operator: BinaryOperator::Add, .. }));
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// CONST DECLARATIONS (3 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn const_integer() {
    let stmt = first_stmt("const MAX = 100");
    match stmt {
        Statement::ConstDeclaration { name, initializer, .. } => {
            assert_eq!(name, "MAX");
            assert!(matches!(initializer, Expression::IntegerLiteral { value: 100, .. }));
        }
        other => panic!("expected ConstDeclaration, got {other:?}"),
    }
}

#[test]
fn const_float() {
    let stmt = first_stmt("const PI = 3.14");
    match stmt {
        Statement::ConstDeclaration { name, initializer, .. } => {
            assert_eq!(name, "PI");
            assert!(matches!(initializer, Expression::FloatLiteral { .. }));
        }
        other => panic!("expected ConstDeclaration, got {other:?}"),
    }
}

#[test]
fn const_null() {
    let stmt = first_stmt("const NOTHING = null");
    match stmt {
        Statement::ConstDeclaration { name, initializer, .. } => {
            assert_eq!(name, "NOTHING");
            assert!(matches!(initializer, Expression::NullLiteral { .. }));
        }
        other => panic!("expected ConstDeclaration, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// PRINT STATEMENTS (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn print_string() {
    let stmt = first_stmt("print(\"Hello\")");
    match stmt {
        Statement::PrintStatement { argument, .. } => {
            match argument {
                Expression::StringLiteral { value, .. } => assert_eq!(value, "Hello"),
                other => panic!("expected StringLiteral, got {other:?}"),
            }
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

#[test]
fn print_integer() {
    let stmt = first_stmt("print(42)");
    match stmt {
        Statement::PrintStatement { argument, .. } => {
            assert!(matches!(argument, Expression::IntegerLiteral { value: 42, .. }));
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

#[test]
fn print_identifier() {
    let stmt = first_stmt("print(x)");
    match stmt {
        Statement::PrintStatement { argument, .. } => {
            match argument {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                other => panic!("expected Identifier, got {other:?}"),
            }
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

#[test]
fn print_expression() {
    let stmt = first_stmt("print(x + 1)");
    match stmt {
        Statement::PrintStatement { argument, .. } => {
            assert!(matches!(argument, Expression::BinaryOp { operator: BinaryOperator::Add, .. }));
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

#[test]
fn print_boolean() {
    let stmt = first_stmt("print(false)");
    match stmt {
        Statement::PrintStatement { argument, .. } => {
            assert!(matches!(argument, Expression::BooleanLiteral { value: false, .. }));
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// LITERALS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn literal_integer() {
    let expr = first_expr("42");
    assert!(matches!(expr, Expression::IntegerLiteral { value: 42, .. }));
}

#[test]
fn literal_float() {
    let expr = first_expr("3.14");
    match expr {
        Expression::FloatLiteral { value, .. } => {
            assert!((value - 3.14).abs() < f64::EPSILON);
        }
        other => panic!("expected FloatLiteral, got {other:?}"),
    }
}

#[test]
fn literal_string() {
    let expr = first_expr("\"hello world\"");
    match expr {
        Expression::StringLiteral { value, .. } => assert_eq!(value, "hello world"),
        other => panic!("expected StringLiteral, got {other:?}"),
    }
}

#[test]
fn literal_true() {
    let expr = first_expr("true");
    assert!(matches!(expr, Expression::BooleanLiteral { value: true, .. }));
}

#[test]
fn literal_false() {
    let expr = first_expr("false");
    assert!(matches!(expr, Expression::BooleanLiteral { value: false, .. }));
}

#[test]
fn literal_null() {
    let expr = first_expr("null");
    assert!(matches!(expr, Expression::NullLiteral { .. }));
}

// ══════════════════════════════════════════════════════════════════════════
// IDENTIFIER (2 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn identifier_simple() {
    let expr = first_expr("myVar");
    match expr {
        Expression::Identifier { name, .. } => assert_eq!(name, "myVar"),
        other => panic!("expected Identifier, got {other:?}"),
    }
}

#[test]
fn identifier_underscore() {
    let expr = first_expr("_count");
    match expr {
        Expression::Identifier { name, .. } => assert_eq!(name, "_count"),
        other => panic!("expected Identifier, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// BINARY OPERATORS — Arithmetic (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn binop_add() {
    let expr = first_expr("1 + 2");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Add, .. }));
}

#[test]
fn binop_subtract() {
    let expr = first_expr("5 - 3");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Subtract, .. }));
}

#[test]
fn binop_multiply() {
    let expr = first_expr("4 * 2");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Multiply, .. }));
}

#[test]
fn binop_divide() {
    let expr = first_expr("10 / 3");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Divide, .. }));
}

#[test]
fn binop_modulo() {
    let expr = first_expr("10 % 3");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Modulo, .. }));
}

// ══════════════════════════════════════════════════════════════════════════
// BINARY OPERATORS — Comparison (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn binop_equal() {
    let expr = first_expr("a == b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Equal, .. }));
}

#[test]
fn binop_not_equal() {
    let expr = first_expr("a != b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::NotEqual, .. }));
}

#[test]
fn binop_less() {
    let expr = first_expr("a < b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Less, .. }));
}

#[test]
fn binop_greater() {
    let expr = first_expr("a > b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Greater, .. }));
}

#[test]
fn binop_less_equal() {
    let expr = first_expr("a <= b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::LessEqual, .. }));
}

#[test]
fn binop_greater_equal() {
    let expr = first_expr("a >= b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::GreaterEqual, .. }));
}

// ══════════════════════════════════════════════════════════════════════════
// BINARY OPERATORS — Logical (2 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn binop_and() {
    let expr = first_expr("a && b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::And, .. }));
}

#[test]
fn binop_or() {
    let expr = first_expr("a || b");
    assert!(matches!(expr, Expression::BinaryOp { operator: BinaryOperator::Or, .. }));
}

// ══════════════════════════════════════════════════════════════════════════
// UNARY OPERATORS (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn unary_negate() {
    let expr = first_expr("-x");
    match expr {
        Expression::UnaryOp { operator, operand, .. } => {
            assert_eq!(operator, UnaryOperator::Negate);
            assert!(matches!(*operand, Expression::Identifier { .. }));
        }
        other => panic!("expected UnaryOp, got {other:?}"),
    }
}

#[test]
fn unary_not() {
    let expr = first_expr("!flag");
    match expr {
        Expression::UnaryOp { operator, operand, .. } => {
            assert_eq!(operator, UnaryOperator::LogicalNot);
            assert!(matches!(*operand, Expression::Identifier { .. }));
        }
        other => panic!("expected UnaryOp, got {other:?}"),
    }
}

#[test]
fn unary_double_negate() {
    // --x parses as -(-(x)), not (--) x, because -- is MinusMinus token.
    // But since we check for Minus (single) in unary, let's use "- -x".
    // Actually, the lexer produces MinusMinus for "--". So "- - x" works.
    let expr = first_expr("- - x");
    match expr {
        Expression::UnaryOp { operator: UnaryOperator::Negate, operand, .. } => {
            assert!(matches!(*operand, Expression::UnaryOp { operator: UnaryOperator::Negate, .. }));
        }
        other => panic!("expected nested UnaryOp, got {other:?}"),
    }
}

#[test]
fn unary_double_not() {
    let expr = first_expr("!!b");
    match expr {
        Expression::UnaryOp { operator: UnaryOperator::LogicalNot, operand, .. } => {
            assert!(matches!(*operand, Expression::UnaryOp { operator: UnaryOperator::LogicalNot, .. }));
        }
        other => panic!("expected nested UnaryOp, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// OPERATOR PRECEDENCE (6 tests)
// ══════════════════════════════════════════════════════════════════════════

/// `1 + 2 * 3` should parse as `1 + (2 * 3)`.
#[test]
fn precedence_mul_before_add() {
    let expr = first_expr("1 + 2 * 3");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Add, right, .. } => {
            assert!(matches!(*right, Expression::BinaryOp { operator: BinaryOperator::Multiply, .. }));
        }
        other => panic!("expected Add at top, got {other:?}"),
    }
}

/// `2 * 3 + 1` should parse as `(2 * 3) + 1`.
#[test]
fn precedence_mul_before_add_reversed() {
    let expr = first_expr("2 * 3 + 1");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Add, left, .. } => {
            assert!(matches!(*left, Expression::BinaryOp { operator: BinaryOperator::Multiply, .. }));
        }
        other => panic!("expected Add at top, got {other:?}"),
    }
}

/// `a < b == c` should parse as `(a < b) == c`.
#[test]
fn precedence_comparison_before_equality() {
    let expr = first_expr("a < b == c");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Equal, left, .. } => {
            assert!(matches!(*left, Expression::BinaryOp { operator: BinaryOperator::Less, .. }));
        }
        other => panic!("expected Equal at top, got {other:?}"),
    }
}

/// `a == b && c` should parse as `(a == b) && c`.
#[test]
fn precedence_equality_before_and() {
    let expr = first_expr("a == b && c");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::And, left, .. } => {
            assert!(matches!(*left, Expression::BinaryOp { operator: BinaryOperator::Equal, .. }));
        }
        other => panic!("expected And at top, got {other:?}"),
    }
}

/// `a && b || c` should parse as `(a && b) || c`.
#[test]
fn precedence_and_before_or() {
    let expr = first_expr("a && b || c");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Or, left, .. } => {
            assert!(matches!(*left, Expression::BinaryOp { operator: BinaryOperator::And, .. }));
        }
        other => panic!("expected Or at top, got {other:?}"),
    }
}

/// `1 + 2 + 3` should parse as `(1 + 2) + 3` (left-associative).
#[test]
fn precedence_left_associativity() {
    let expr = first_expr("1 + 2 + 3");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Add, left, right, .. } => {
            assert!(matches!(*left, Expression::BinaryOp { operator: BinaryOperator::Add, .. }));
            assert!(matches!(*right, Expression::IntegerLiteral { value: 3, .. }));
        }
        other => panic!("expected left-associative Add, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// GROUPING (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn grouping_simple() {
    let expr = first_expr("(42)");
    match expr {
        Expression::Grouping { expression, .. } => {
            assert!(matches!(*expression, Expression::IntegerLiteral { value: 42, .. }));
        }
        other => panic!("expected Grouping, got {other:?}"),
    }
}

/// `(1 + 2) * 3` — grouping overrides default precedence.
#[test]
fn grouping_overrides_precedence() {
    let expr = first_expr("(1 + 2) * 3");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Multiply, left, .. } => {
            assert!(matches!(*left, Expression::Grouping { .. }));
        }
        other => panic!("expected Multiply at top, got {other:?}"),
    }
}

#[test]
fn grouping_nested() {
    let expr = first_expr("((x))");
    match expr {
        Expression::Grouping { expression, .. } => {
            assert!(matches!(*expression, Expression::Grouping { .. }));
        }
        other => panic!("expected nested Grouping, got {other:?}"),
    }
}

#[test]
fn grouping_complex() {
    let expr = first_expr("(a + b) * (c - d)");
    match expr {
        Expression::BinaryOp { operator: BinaryOperator::Multiply, left, right, .. } => {
            assert!(matches!(*left, Expression::Grouping { .. }));
            assert!(matches!(*right, Expression::Grouping { .. }));
        }
        other => panic!("expected Multiply with two Groupings, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// MULTI-STATEMENT PROGRAMS (4 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn multi_let_and_print() {
    let program = parse_source("let x = 42\nprint(x)").unwrap();
    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.statements[0], Statement::LetDeclaration { .. }));
    assert!(matches!(program.statements[1], Statement::PrintStatement { .. }));
}

#[test]
fn multi_several_lets() {
    let program = parse_source("let a = 1\nlet b = 2\nlet c = 3").unwrap();
    assert_eq!(program.statements.len(), 3);
    for stmt in &program.statements {
        assert!(matches!(stmt, Statement::LetDeclaration { .. }));
    }
}

#[test]
fn multi_let_const_print() {
    let program = parse_source("let x = 10\nconst Y = 20\nprint(x + Y)").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(program.statements[0], Statement::LetDeclaration { .. }));
    assert!(matches!(program.statements[1], Statement::ConstDeclaration { .. }));
    assert!(matches!(program.statements[2], Statement::PrintStatement { .. }));
}

#[test]
fn multi_expression_statements() {
    let program = parse_source("42\n\"hello\"\ntrue").unwrap();
    assert_eq!(program.statements.len(), 3);
    for stmt in &program.statements {
        assert!(matches!(stmt, Statement::ExpressionStatement { .. }));
    }
}

// ══════════════════════════════════════════════════════════════════════════
// ERROR CASES (8 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn error_let_missing_name() {
    assert_parse_error("let = 42", "BZ0006");
}

#[test]
fn error_let_missing_equals() {
    assert_parse_error("let x 42", "BZ0006");
}

#[test]
fn error_let_missing_initializer() {
    assert_parse_error("let x =", "BZ0007");
}

#[test]
fn error_const_missing_name() {
    assert_parse_error("const = 42", "BZ0006");
}

#[test]
fn error_print_missing_left_paren() {
    assert_parse_error("print 42)", "BZ0006");
}

#[test]
fn error_print_missing_right_paren() {
    assert_parse_error("print(42", "BZ0006");
}

#[test]
fn error_unclosed_grouping() {
    assert_parse_error("(1 + 2", "BZ0006");
}

#[test]
fn error_unexpected_token() {
    assert_parse_error(")", "BZ0007");
}

// ══════════════════════════════════════════════════════════════════════════
// EDGE CASES (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_program() {
    let program = parse_source("").unwrap();
    assert!(program.statements.is_empty());
}

#[test]
fn edge_only_whitespace() {
    let program = parse_source("   \n\n   ").unwrap();
    assert!(program.statements.is_empty());
}

#[test]
fn edge_only_comments() {
    let program = parse_source("// nothing here\n/* also nothing */").unwrap();
    assert!(program.statements.is_empty());
}

#[test]
fn edge_single_literal() {
    let program = parse_source("42").unwrap();
    assert_eq!(program.statements.len(), 1);
    assert!(matches!(program.statements[0], Statement::ExpressionStatement { .. }));
}

#[test]
fn edge_deeply_nested_grouping() {
    let expr = first_expr("(((1 + 2)))");
    match expr {
        Expression::Grouping { expression, .. } => {
            match *expression {
                Expression::Grouping { expression, .. } => {
                    match *expression {
                        Expression::Grouping { expression, .. } => {
                            assert!(matches!(*expression, Expression::BinaryOp { .. }));
                        }
                        other => panic!("expected inner Grouping, got {other:?}"),
                    }
                }
                other => panic!("expected middle Grouping, got {other:?}"),
            }
        }
        other => panic!("expected outer Grouping, got {other:?}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════
// LINE/COLUMN TRACKING (3 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn tracking_let_position() {
    let stmt = first_stmt("let x = 42");
    match stmt {
        Statement::LetDeclaration { line, column, .. } => {
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        }
        other => panic!("expected LetDeclaration, got {other:?}"),
    }
}

#[test]
fn tracking_print_position() {
    let program = parse_source("let x = 1\nprint(x)").unwrap();
    match &program.statements[1] {
        Statement::PrintStatement { line, column, .. } => {
            assert_eq!(*line, 2);
            assert_eq!(*column, 1);
        }
        other => panic!("expected PrintStatement, got {other:?}"),
    }
}

#[test]
fn tracking_expression_position() {
    let expr = first_expr("42");
    match expr {
        Expression::IntegerLiteral { line, column, .. } => {
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        }
        other => panic!("expected IntegerLiteral, got {other:?}"),
    }
}
