//! Unit tests for the Bunzo code formatter.

use super::format;

fn assert_formatted(input: &str, expected: &str) {
    let formatted = format(input).expect("Failed to format input");
    assert_eq!(formatted, expected);
    // Assert idempotency
    let double_formatted = format(&formatted).expect("Failed to double-format");
    assert_eq!(double_formatted, formatted);
}

#[test]
fn test_format_empty() {
    assert_formatted("", "");
    assert_formatted("\n\n  \n", "");
}

#[test]
fn test_format_variables_and_constants() {
    assert_formatted("let   x=42", "let x = 42\n");
    assert_formatted("const  y   =  \"hello\"", "const y = \"hello\"\n");
}

#[test]
fn test_format_print() {
    assert_formatted("print(  x  )", "print(x)\n");
}

#[test]
fn test_format_functions() {
    assert_formatted(
        "func   add(a:int,b:string)->int{return a}",
        "func add(a: int, b: string) -> int {\n    return a\n}\n",
    );
}

#[test]
fn test_format_if_else() {
    assert_formatted(
        "if x{print(x)}else if y{print(y)}else{print(z)}",
        "if x {\n    print(x)\n} else if y {\n    print(y)\n} else {\n    print(z)\n}\n",
    );
}

#[test]
fn test_format_loops() {
    assert_formatted("while true{break}", "while true {\n    break\n}\n");
    assert_formatted(
        "for i in 0..10{continue}",
        "for i in 0..10 {\n    continue\n}\n",
    );
}

#[test]
fn test_format_structs() {
    assert_formatted(
        "struct Point{x:int\ny:int}",
        "struct Point {\n    x: int\n    y: int\n}\n",
    );
    assert_formatted(
        "let p = Point{x:1,y:2}",
        "let p = Point {\n    x: 1,\n    y: 2\n}\n",
    );
}

#[test]
fn test_format_classes() {
    assert_formatted(
        "class Counter extends Base implements Foo, Bar {value:int private name:string func get()->int{return this.value}}",
        "class Counter extends Base implements Foo, Bar {\n    value: int\n    private name: string\n\n    func get() -> int {\n        return this.value\n    }\n}\n",
    );
}

#[test]
fn test_format_interfaces() {
    assert_formatted(
        "interface Speakable{func speak()func name()->string}",
        "interface Speakable {\n    func speak()\n    func name() -> string\n}\n",
    );
}

#[test]
fn test_format_try_catch() {
    assert_formatted(
        "try{print(1)}catch err{print(err)}",
        "try {\n    print(1)\n} catch err {\n    print(err)\n}\n",
    );
}

#[test]
fn test_format_comments_before() {
    assert_formatted("// A comment\nlet x = 1", "// A comment\nlet x = 1\n");
    assert_formatted(
        "/* Block comment */\nconst x = 2",
        "/* Block comment */\nconst x = 2\n",
    );
}

#[test]
fn test_format_comments_trailing() {
    assert_formatted("let x = 1 // comment", "let x = 1 // comment\n");
}

#[test]
fn test_format_nested_block_comments() {
    assert_formatted(
        "if x {\n    // comment inside\n    print(x)\n}",
        "if x {\n    // comment inside\n    print(x)\n}\n",
    );
}
