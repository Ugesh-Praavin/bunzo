//! Integration tests for static type checking in Bunzo.

use bzc::diagnostics::CompilerError;
use bzc::lexer;
use bzc::parser;
use bzc::semantic;
use bzc::typechecker;

fn check_source(source: &str) -> Result<(), CompilerError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    semantic::analyze(&program)?;
    typechecker::check(&program)?;
    Ok(())
}

#[test]
fn test_typechecker_primitives_valid() {
    let result = check_source(
        r#"
        let a = 10
        let b = 3.14
        let c = "hello"
        let d = true
        let e = null
        "#,
    );
    assert!(
        result.is_ok(),
        "Expected valid primitives to typecheck successfully, got: {:?}",
        result
    );
}

#[test]
fn test_typechecker_variable_reassignment_mismatch() {
    // Reassigning int to string
    let result = check_source(
        r#"
        let x = 10
        x = "hello"
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(
            operation.contains("assignment"),
            "Expected assignment mismatch, got operation: {}",
            operation
        );
        assert_eq!(expected, "int");
        assert_eq!(found, "string");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_const_reassignment_mismatch() {
    // Reassigning const is a semantic/const error, but typechecker should check compatibility first or alongside it.
    let result = check_source(
        r#"
        const y = 3.14
        y = "world"
        "#,
    );
    // Note: semantic analysis runs before typechecker. Const reassignment is actually caught in semantic analyzer.
    // Let's verify that it returns an error.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CompilerError::ConstReassignment { .. }));
}

#[test]
fn test_typechecker_binary_op_mismatch() {
    // Adding string and integer
    let result = check_source(
        r#"
        let x = "hello" + 5
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(operation.contains("+"));
        assert_eq!(expected, "string");
        assert_eq!(found, "string and int");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_unary_op_mismatch() {
    // Logical NOT on an integer
    let result = check_source(
        r#"
        let x = !10
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(operation.contains("!"));
        assert_eq!(expected, "bool");
        assert_eq!(found, "int");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_function_args_mismatch() {
    let result = check_source(
        r#"
        func add_one(x: int) -> int {
            return x + 1
        }
        let res = add_one("hello")
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(
            operation.contains("argument 1"),
            "Expected argument mismatch, got operation: {}",
            operation
        );
        assert_eq!(expected, "int");
        assert_eq!(found, "string");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_function_return_mismatch() {
    let result = check_source(
        r#"
        func get_name() -> string {
            return 42
        }
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(
            operation.contains("return"),
            "Expected return mismatch, got operation: {}",
            operation
        );
        assert_eq!(expected, "string");
        assert_eq!(found, "int");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_struct_fields_mismatch() {
    let result = check_source(
        r#"
        struct Point {
            x: int
            y: int
        }
        let p = Point { x: 10, y: "twenty" }
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(
            operation.contains("field 'y'"),
            "Expected field initialization mismatch, got operation: {}",
            operation
        );
        assert_eq!(expected, "int");
        assert_eq!(found, "string");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_class_field_assignment_mismatch() {
    let result = check_source(
        r#"
        class Account {
            balance: int
            func init() {
                this.balance = 0
            }
        }
        let acc = Account()
        acc.balance = "one million"
        "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    if let CompilerError::TypeMismatch {
        operation,
        expected,
        found,
        ..
    } = err
    {
        assert!(
            operation.contains("class field 'balance'"),
            "Expected field assignment mismatch, got operation: {}",
            operation
        );
        assert_eq!(expected, "int");
        assert_eq!(found, "string");
    } else {
        panic!("Expected TypeMismatch error, got: {:?}", err);
    }
}

#[test]
fn test_typechecker_inheritance_subtyping() {
    let result = check_source(
        r#"
        class Animal {
            func speak() {}
        }
        class Dog extends Animal {
            func bark() {}
        }
        
        func play(a: Animal) {}
        
        // Pass subclass instance to parent parameter - should be compatible
        play(Dog())
        "#,
    );
    assert!(
        result.is_ok(),
        "Expected subclass Dog to be compatible with parent parameter type Animal, got: {:?}",
        result
    );
}

#[test]
fn test_typechecker_interface_subtyping() {
    let result = check_source(
        r#"
        interface Runnable {
            func run()
        }
        class Athlete implements Runnable {
            func run() {}
        }
        
        func start_race(r: Runnable) {}
        
        start_race(Athlete())
        "#,
    );
    assert!(
        result.is_ok(),
        "Expected class implementing Runnable interface to be compatible, got: {:?}",
        result
    );
}

#[test]
fn test_typechecker_module_imports() {
    std::fs::write(
        "temp_tc_module.bz",
        "export func double(x: int) -> int { return x * 2 }\n",
    )
    .unwrap();

    let result = check_source(
        r#"
        import temp_tc_module
        let result = temp_tc_module.double(10)
        "#,
    );
    assert!(
        result.is_ok(),
        "Expected valid type checking on module import to pass, got: {:?}",
        result
    );

    let _ = std::fs::remove_file("temp_tc_module.bz");
}
