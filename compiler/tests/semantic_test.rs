//! Unit tests for static semantic analysis.

use bzc::ast::{BinaryOperator, Expression, Program, Statement};
use bzc::diagnostics::CompilerError;
use bzc::semantic::analyze;

#[test]
fn test_valid_program() {
    let program = Program {
        statements: vec![
            Statement::LetDeclaration {
                name: "x".to_string(),
                initializer: Expression::IntegerLiteral {
                    value: 10,
                    line: 1,
                    column: 9,
                },
                line: 1,
                column: 1,
            },
            Statement::ConstDeclaration {
                name: "Y".to_string(),
                initializer: Expression::Identifier {
                    name: "x".to_string(),
                    line: 2,
                    column: 13,
                },
                line: 2,
                column: 1,
            },
            Statement::PrintStatement {
                argument: Expression::BinaryOp {
                    operator: BinaryOperator::Add,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        line: 3,
                        column: 7,
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "Y".to_string(),
                        line: 3,
                        column: 11,
                    }),
                    line: 3,
                    column: 9,
                },
                line: 3,
                column: 1,
            },
        ],
    };

    assert!(analyze(&program).is_ok());
}

#[test]
fn test_undefined_variable_in_let_initializer() {
    let program = Program {
        statements: vec![Statement::LetDeclaration {
            name: "x".to_string(),
            initializer: Expression::Identifier {
                name: "undefined_var".to_string(),
                line: 1,
                column: 9,
            },
            line: 1,
            column: 1,
        }],
    };

    let result = analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::UndefinedVariable { name, line, column } = err {
        assert_eq!(name, "undefined_var");
        assert_eq!(line, 1);
        assert_eq!(column, 9);
    } else {
        panic!("expected UndefinedVariable error, found: {:?}", err);
    }
}

#[test]
fn test_undefined_variable_in_const_initializer() {
    let program = Program {
        statements: vec![Statement::ConstDeclaration {
            name: "PI".to_string(),
            initializer: Expression::Identifier {
                name: "undefined_var".to_string(),
                line: 1,
                column: 12,
            },
            line: 1,
            column: 1,
        }],
    };

    let result = analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CompilerError::UndefinedVariable { .. }));
}

#[test]
fn test_undefined_variable_in_print() {
    let program = Program {
        statements: vec![Statement::PrintStatement {
            argument: Expression::Identifier {
                name: "x".to_string(),
                line: 1,
                column: 7,
            },
            line: 1,
            column: 1,
        }],
    };

    let result = analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CompilerError::UndefinedVariable { .. }));
}

#[test]
fn test_duplicate_declaration_let_let() {
    let program = Program {
        statements: vec![
            Statement::LetDeclaration {
                name: "x".to_string(),
                initializer: Expression::IntegerLiteral {
                    value: 1,
                    line: 1,
                    column: 9,
                },
                line: 1,
                column: 1,
            },
            Statement::LetDeclaration {
                name: "x".to_string(),
                initializer: Expression::IntegerLiteral {
                    value: 2,
                    line: 2,
                    column: 9,
                },
                line: 2,
                column: 1,
            },
        ],
    };

    let result = analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::DuplicateDeclaration { name, line, column } = err {
        assert_eq!(name, "x");
        assert_eq!(line, 2);
        assert_eq!(column, 1);
    } else {
        panic!("expected DuplicateDeclaration error, found: {:?}", err);
    }
}

#[test]
fn test_duplicate_declaration_let_const() {
    let program = Program {
        statements: vec![
            Statement::LetDeclaration {
                name: "x".to_string(),
                initializer: Expression::IntegerLiteral {
                    value: 1,
                    line: 1,
                    column: 9,
                },
                line: 1,
                column: 1,
            },
            Statement::ConstDeclaration {
                name: "x".to_string(),
                initializer: Expression::IntegerLiteral {
                    value: 2,
                    line: 2,
                    column: 11,
                },
                line: 2,
                column: 1,
            },
        ],
    };

    let result = analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CompilerError::DuplicateDeclaration { .. }));
}

#[test]
fn test_semantic_module_imports() {
    std::fs::write(
        "temp_module.bz",
        "export func greet() {}\nfunc private_helper() {}\n",
    )
    .unwrap();

    let tokens = bzc::lexer::tokenize("import temp_module\ntemp_module.greet()").unwrap();
    let program = bzc::parser::parse(tokens).unwrap();
    let result = bzc::semantic::analyze(&program);
    assert!(
        result.is_ok(),
        "Expected valid module import semantic analysis to pass, got: {:?}",
        result
    );

    let tokens_invalid =
        bzc::lexer::tokenize("import temp_module\ntemp_module.private_helper()").unwrap();
    let program_invalid = bzc::parser::parse(tokens_invalid).unwrap();
    let result_invalid = bzc::semantic::analyze(&program_invalid);
    assert!(
        result_invalid.is_err(),
        "Expected error for unexported member access"
    );
    assert!(matches!(
        result_invalid.unwrap_err(),
        CompilerError::UnexportedMemberAccess { .. }
    ));

    let _ = std::fs::remove_file("temp_module.bz");
}
