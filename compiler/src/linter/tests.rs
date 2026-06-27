use super::LintWarning;

fn lint_code(src: &str) -> Vec<LintWarning> {
    let tokens = crate::lexer::tokenize(src).unwrap();
    let program = crate::parser::parse(tokens).unwrap();
    super::lint(&program)
}

#[test]
fn test_naming_variables() {
    let warnings = lint_code("let valid_var = 1");
    assert!(warnings.iter().all(|w| w.code != "LN0001"));

    let warnings = lint_code("let invalidName = 1");
    assert!(warnings.iter().any(|w| w.code == "LN0001"));
}

#[test]
fn test_naming_constants() {
    let warnings = lint_code("const VALID_CONST = 1");
    assert!(warnings.iter().all(|w| w.code != "LN0004"));

    let warnings = lint_code("const invalidConst = 1");
    assert!(warnings.iter().any(|w| w.code == "LN0004"));
}

#[test]
fn test_naming_functions() {
    let warnings = lint_code("func valid_func() {\nlet x = 1\nprint(x)\n}");
    assert!(warnings.iter().all(|w| w.code != "LN0002"));

    let warnings = lint_code("func invalidFunc() {\nlet x = 1\nprint(x)\n}");
    assert!(warnings.iter().any(|w| w.code == "LN0002"));
}

#[test]
fn test_naming_types() {
    let warnings = lint_code("class ValidClass {}");
    assert!(warnings.iter().all(|w| w.code != "LN0003"));

    let warnings = lint_code("class invalid_class {}");
    assert!(warnings.iter().any(|w| w.code == "LN0003"));
}

#[test]
fn test_naming_booleans() {
    let warnings = lint_code("let is_active = true");
    assert!(warnings.iter().all(|w| w.code != "LN0005"));

    let warnings = lint_code("let active = true");
    assert!(warnings.iter().any(|w| w.code == "LN0005"));
}

#[test]
fn test_empty_blocks() {
    let warnings = lint_code("func foo() {}");
    assert!(warnings.iter().any(|w| w.code == "LQ0001"));

    let warnings = lint_code("if true {}");
    assert!(warnings.iter().any(|w| w.code == "LQ0001"));

    let warnings = lint_code("try {\nlet x = 1\nprint(x)\n} catch e {}");
    assert!(warnings.iter().any(|w| w.code == "LQ0001"));
}

#[test]
fn test_unused_variables() {
    let warnings = lint_code("let x = 1");
    assert!(warnings.iter().any(|w| w.code == "LQ0002"));

    let warnings = lint_code("let x = 1\nprint(x)");
    assert!(warnings.iter().all(|w| w.code != "LQ0002"));
}

#[test]
fn test_magic_numbers() {
    let warnings = lint_code("let x = 100");
    assert!(warnings.iter().all(|w| w.code != "LQ0003"));

    let warnings = lint_code("let x = 1 + 2\nprint(x)");
    assert!(warnings.iter().any(|w| w.code == "LQ0003"));

    let warnings = lint_code("print(100)");
    assert!(warnings.iter().any(|w| w.code == "LQ0003"));
}
