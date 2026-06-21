//! Runtime and evaluation test suite.


use crate::ast::{
    BinaryOperator, Expression, Program, Statement, UnaryOperator,
};
use crate::diagnostics::CompilerError;
use super::environment::Environment;
use super::eval::Interpreter;
use super::value::RuntimeValue;

// Helper to evaluate a single expression
fn eval_expr(expr: Expression) -> Result<RuntimeValue, CompilerError> {
    let mut interpreter = Interpreter::new(Vec::new());
    interpreter.eval_expr(&expr)
}

// Helper to execute a sequence of statements and get stdout
fn execute_statements(stmts: Vec<Statement>) -> Result<String, CompilerError> {
    let mut buffer = Vec::new();
    let mut interpreter = Interpreter::new(&mut buffer);
    let program = Program { statements: stmts };
    interpreter.interpret(program)?;
    Ok(String::from_utf8(buffer).expect("invalid utf-8 output"))
}

#[test]
fn test_literal_eval() {
    assert_eq!(
        eval_expr(Expression::IntegerLiteral { value: 42, line: 1, column: 1 }).unwrap(),
        RuntimeValue::Integer(42)
    );
    assert_eq!(
        eval_expr(Expression::FloatLiteral { value: 3.14, line: 1, column: 1 }).unwrap(),
        RuntimeValue::Float(3.14)
    );
    assert_eq!(
        eval_expr(Expression::StringLiteral { value: "hello".to_string(), line: 1, column: 1 }).unwrap(),
        RuntimeValue::String("hello".to_string())
    );
    assert_eq!(
        eval_expr(Expression::BooleanLiteral { value: true, line: 1, column: 1 }).unwrap(),
        RuntimeValue::Boolean(true)
    );
    assert_eq!(
        eval_expr(Expression::NullLiteral { line: 1, column: 1 }).unwrap(),
        RuntimeValue::Null
    );
}

#[test]
fn test_unary_negate() {
    assert_eq!(
        eval_expr(Expression::UnaryOp {
            operator: UnaryOperator::Negate,
            operand: Box::new(Expression::IntegerLiteral { value: 5, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Integer(-5)
    );

    assert_eq!(
        eval_expr(Expression::UnaryOp {
            operator: UnaryOperator::Negate,
            operand: Box::new(Expression::FloatLiteral { value: 1.5, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Float(-1.5)
    );

    let err = eval_expr(Expression::UnaryOp {
        operator: UnaryOperator::Negate,
        operand: Box::new(Expression::StringLiteral { value: "x".to_string(), line: 1, column: 1 }),
        line: 1,
        column: 1,
    });
    assert!(matches!(err, Err(CompilerError::TypeMismatch { .. })));
}

#[test]
fn test_unary_not() {
    assert_eq!(
        eval_expr(Expression::UnaryOp {
            operator: UnaryOperator::LogicalNot,
            operand: Box::new(Expression::BooleanLiteral { value: false, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Boolean(true)
    );

    let err = eval_expr(Expression::UnaryOp {
        operator: UnaryOperator::LogicalNot,
        operand: Box::new(Expression::IntegerLiteral { value: 0, line: 1, column: 1 }),
        line: 1,
        column: 1,
    });
    assert!(matches!(err, Err(CompilerError::TypeMismatch { .. })));
}

#[test]
fn test_binary_arithmetic() {
    // Integer addition
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::IntegerLiteral { value: 2, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: 3, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Integer(5)
    );

    // Float addition
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::FloatLiteral { value: 1.5, line: 1, column: 1 }),
            right: Box::new(Expression::FloatLiteral { value: 2.0, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Float(3.5)
    );

    // Coercion addition
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::IntegerLiteral { value: 2, line: 1, column: 1 }),
            right: Box::new(Expression::FloatLiteral { value: 1.5, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Float(3.5)
    );

    // String concat
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::StringLiteral { value: "hello ".to_string(), line: 1, column: 1 }),
            right: Box::new(Expression::StringLiteral { value: "world".to_string(), line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::String("hello world".to_string())
    );

    // Modulo
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Modulo,
            left: Box::new(Expression::IntegerLiteral { value: 10, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: 3, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Integer(1)
    );
}

#[test]
fn test_short_circuit_and() {
    // false && expr -> short circuits
    // We test this by using an undefined identifier on the right. If it evaluated, it would error.
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::And,
            left: Box::new(Expression::BooleanLiteral { value: false, line: 1, column: 1 }),
            right: Box::new(Expression::Identifier { name: "undefined_var".to_string(), line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Boolean(false)
    );

    // true && false -> false
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::And,
            left: Box::new(Expression::BooleanLiteral { value: true, line: 1, column: 1 }),
            right: Box::new(Expression::BooleanLiteral { value: false, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Boolean(false)
    );

    // true && non-boolean -> TypeMismatch
    let err = eval_expr(Expression::BinaryOp {
        operator: BinaryOperator::And,
        left: Box::new(Expression::BooleanLiteral { value: true, line: 1, column: 1 }),
        right: Box::new(Expression::IntegerLiteral { value: 42, line: 1, column: 1 }),
        line: 1,
        column: 1,
    });
    assert!(matches!(err, Err(CompilerError::TypeMismatch { .. })));
}

#[test]
fn test_short_circuit_or() {
    // true || expr -> short circuits
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Or,
            left: Box::new(Expression::BooleanLiteral { value: true, line: 1, column: 1 }),
            right: Box::new(Expression::Identifier { name: "undefined_var".to_string(), line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Boolean(true)
    );

    // false || false -> false
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Or,
            left: Box::new(Expression::BooleanLiteral { value: false, line: 1, column: 1 }),
            right: Box::new(Expression::BooleanLiteral { value: false, line: 1, column: 1 }),
            line: 1,
            column: 1,
        }).unwrap(),
        RuntimeValue::Boolean(false)
    );
}

#[test]
fn test_division_by_zero() {
    let err = eval_expr(Expression::BinaryOp {
        operator: BinaryOperator::Divide,
        left: Box::new(Expression::IntegerLiteral { value: 5, line: 1, column: 1 }),
        right: Box::new(Expression::IntegerLiteral { value: 0, line: 1, column: 1 }),
        line: 1,
        column: 1,
    });
    assert!(matches!(err, Err(CompilerError::DivisionByZero { .. })));

    let err_float = eval_expr(Expression::BinaryOp {
        operator: BinaryOperator::Divide,
        left: Box::new(Expression::FloatLiteral { value: 5.5, line: 1, column: 1 }),
        right: Box::new(Expression::FloatLiteral { value: 0.0, line: 1, column: 1 }),
        line: 1,
        column: 1,
    });
    assert!(matches!(err_float, Err(CompilerError::DivisionByZero { .. })));
}

#[test]
fn test_integer_overflow_uses_wrapping_behavior() {
    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::IntegerLiteral { value: i64::MAX, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: 1, line: 1, column: 1 }),
            line: 1,
            column: 1,
        })
        .unwrap(),
        RuntimeValue::Integer(i64::MIN)
    );

    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Subtract,
            left: Box::new(Expression::IntegerLiteral { value: i64::MIN, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: 1, line: 1, column: 1 }),
            line: 1,
            column: 1,
        })
        .unwrap(),
        RuntimeValue::Integer(i64::MAX)
    );

    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Multiply,
            left: Box::new(Expression::IntegerLiteral { value: i64::MAX, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: 2, line: 1, column: 1 }),
            line: 1,
            column: 1,
        })
        .unwrap(),
        RuntimeValue::Integer(-2)
    );

    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Divide,
            left: Box::new(Expression::IntegerLiteral { value: i64::MIN, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: -1, line: 1, column: 1 }),
            line: 1,
            column: 1,
        })
        .unwrap(),
        RuntimeValue::Integer(i64::MIN)
    );

    assert_eq!(
        eval_expr(Expression::BinaryOp {
            operator: BinaryOperator::Modulo,
            left: Box::new(Expression::IntegerLiteral { value: i64::MIN, line: 1, column: 1 }),
            right: Box::new(Expression::IntegerLiteral { value: -1, line: 1, column: 1 }),
            line: 1,
            column: 1,
        })
        .unwrap(),
        RuntimeValue::Integer(0)
    );
}

#[test]
fn test_variables_and_print() {
    let stmts = vec![
        Statement::LetDeclaration {
            name: "x".to_string(),
            initializer: Expression::IntegerLiteral { value: 10, line: 1, column: 1 },
            line: 1,
            column: 1,
        },
        Statement::ConstDeclaration {
            name: "Y".to_string(),
            initializer: Expression::IntegerLiteral { value: 20, line: 2, column: 1 },
            line: 2,
            column: 1,
        },
        Statement::PrintStatement {
            argument: Expression::BinaryOp {
                operator: BinaryOperator::Add,
                left: Box::new(Expression::Identifier { name: "x".to_string(), line: 3, column: 1 }),
                right: Box::new(Expression::Identifier { name: "Y".to_string(), line: 3, column: 1 }),
                line: 3,
                column: 1,
            },
            line: 3,
            column: 1,
        },
    ];

    let output = execute_statements(stmts).unwrap();
    assert_eq!(output, "30\n");
}

#[test]
fn test_duplicate_declaration() {
    let stmts = vec![
        Statement::LetDeclaration {
            name: "x".to_string(),
            initializer: Expression::IntegerLiteral { value: 10, line: 1, column: 1 },
            line: 1,
            column: 1,
        },
        Statement::LetDeclaration {
            name: "x".to_string(),
            initializer: Expression::IntegerLiteral { value: 20, line: 2, column: 1 },
            line: 2,
            column: 1,
        },
    ];

    let err = execute_statements(stmts);
    assert!(matches!(err, Err(CompilerError::DuplicateDeclaration { .. })));
}

#[test]
fn test_const_reassignment_in_environment() {
    let mut env = Environment::new();
    env.define("PI".to_string(), RuntimeValue::Float(3.14), true, 1, 1).unwrap();
    let err = env.assign("PI".to_string(), RuntimeValue::Float(3.15), 2, 1);
    assert!(matches!(err, Err(CompilerError::ConstReassignment { .. })));
}
