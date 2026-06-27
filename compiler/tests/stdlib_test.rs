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

#[test]
fn test_import_http_and_json_modules() {
    let out = run_source(
        r#"
        import http
        import json
        print(type(http.get))
        print(type(http.post))
        print(type(http.put))
        print(type(http.delete))
        print(type(http.patch))
        let s = json.encode(42)
        print(s)
        "#,
    )
    .unwrap();
    assert_eq!(out, "Builtin\nBuiltin\nBuiltin\nBuiltin\nBuiltin\n42\n");
}

#[test]
fn test_import_db_module() {
    let out = run_source(
        r#"
        import db
        print(type(db.open))
        print(type(db.execute))
        print(type(db.query))
        let conn = db.open(":memory:")
        print(type(conn))
        db.execute(conn, "CREATE TABLE t (v INTEGER)")
        db.execute(conn, "INSERT INTO t VALUES (7)")
        let rows = db.query(conn, "SELECT v FROM t")
        print(type(rows))
        db.close(conn)
        "#,
    )
    .unwrap();
    assert!(out.contains("Builtin"));
    assert!(out.contains("DbConnection"));
    assert!(out.contains("Array"));
}

#[test]
fn test_new_stdlib_modules() {
    let out = run_source(
        r#"
        import vector
        import stack
        import set
        import map
        import string
        import path
        import time
        import random
        import crypto
        import encoding

        let v = vector.new()
        vector.push(v, 10)
        vector.push(v, 20)
        print(vector.len(v))

        let stk = stack.new()
        stack.push(stk, 100)
        print(stack.top(stk))

        let s = set.new()
        set.insert(s, "hello")
        print(set.contains(s, "hello"))

        let m = map.new()
        map.insert(m, "key", "val")
        print(map.get(m, "key"))

        print(string.len("test"))
        print(path.join("a", "b"))
        print(encoding.hex_encode("A"))
        print(type(random.int))
        print(type(crypto.uuid))
        "#,
    )
    .unwrap();
    let out_normalized = out.replace('\\', "/");
    assert_eq!(out_normalized, "2\n100\ntrue\nval\n4\na/b\n41\nBuiltin\nBuiltin\n");
}

