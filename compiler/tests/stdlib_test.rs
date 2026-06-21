//! End-to-end integration tests for standard library builtins.

use bzc::lexer;
use bzc::parser;
use bzc::runtime;
use bzc::semantic;

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
fn test_stdlib_builtins() {
    let out = run_source(
        r#"
        let s = "hello"
        print(len(s))
        print(type(s))
        print(type(42))
        print(str(100))
        print(to_int("123") + 1)
        "#,
    )
    .unwrap();
    assert_eq!(out, "5\nString\nInteger\n100\n124\n");
}
