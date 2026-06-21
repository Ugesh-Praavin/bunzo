//! End-to-end integration tests for classes and OOP concepts (declarations, instantiation, field access, methods, and this binding).
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
fn test_class_declaration_and_instantiation() {
    let out = run_source(
        r#"
        class Counter {
            val: int
            func init() {
                this.val = 0
            }
            func inc() {
                this.val = this.val + 1
            }
            func get() -> int {
                return this.val
            }
        }
        let c = Counter()
        c.init()
        print(c.get())
        c.inc()
        print(c.get())
        "#,
    )
    .unwrap();
    assert_eq!(out, "0\n1\n");
}

#[test]
fn test_class_field_modification() {
    let out = run_source(
        r#"
        class Point {
            x: int
            y: int
        }
        let p = Point()
        p.x = 10
        p.y = 20
        print(p.x)
        print(p.y)
        "#,
    )
    .unwrap();
    assert_eq!(out, "10\n20\n");
}

#[test]
fn test_inheritance_super_and_auto_constructor() {
    let out = run_source(
        r#"
        abstract class Base {
            value: int
            abstract func bump()
            func init() {
                this.value = 1
            }
        }
        class Child extends Base {
            func bump() {
                this.value = this.value + 10
            }
            func show() -> int {
                return this.value
            }
        }
        let c = Child()
        c.bump()
        print(c.show())
        "#,
    )
    .unwrap();
    assert_eq!(out, "11\n");
}

#[test]
fn test_interface_implements_and_private_field() {
    let result = run_source(
        r#"
        interface Greetable {
            func greet()
        }
        class Greeter implements Greetable {
            private secret: string
            func init() {
                this.secret = "hidden"
            }
            func greet() {
                print(this.secret)
            }
        }
        let g = Greeter()
        g.greet()
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hidden\n");
}

#[test]
fn test_abstract_class_instantiation_rejected() {
    let result = run_source(
        r#"
        abstract class Thing {
            abstract func go()
        }
        let t = Thing()
        "#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("BZ1023"));
}
