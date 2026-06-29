use crate::lsp::handlers::{compile_and_get_diagnostics, handle_definition, handle_hover};
use crate::lsp::protocol::Request;
use std::collections::HashMap;

#[test]
fn test_lsp_diagnostics_valid() {
    let code = "
let x = 42
const y = 3.14
print(x)
";
    let diags = compile_and_get_diagnostics(code);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics for valid code, found: {:?}",
        diags
    );
}

#[test]
fn test_lsp_diagnostics_syntax_error() {
    let code = "
let x =
";
    let diags = compile_and_get_diagnostics(code);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("Expected expression"));
    assert_eq!(diags[0].range.start.line, 2); // 0-indexed line 2 (which is 1-based line 3)
}

#[test]
fn test_lsp_diagnostics_type_error() {
    let code = "
let x = 10
x = \"hello\"
";
    let diags = compile_and_get_diagnostics(code);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("Type mismatch"));
}

#[test]
fn test_lsp_hover() {
    let code = "let my_var = 100\nprint(my_var)";

    // Hover over my_var definition: line 0, char 4 (which is "my_var")
    let hover_def = handle_hover(code, 0, 4).unwrap();
    assert!(hover_def.contents.value.contains("let my_var"));

    // Hover over my_var usage: line 1, char 6 (which is inside "my_var" in print(my_var))
    let hover_usage = handle_hover(code, 1, 6).unwrap();
    assert!(hover_usage.contents.value.contains("let my_var"));
}

#[test]
fn test_lsp_definition() {
    let code = "let my_var = 100\nprint(my_var)";
    let uri = "file:///test.bz";

    // Go to definition of my_var in print(my_var): line 1, char 6
    let loc = handle_definition(uri, code, 1, 6).unwrap();
    assert_eq!(loc.uri, uri);
    // Should point to "my_var" declaration at line 0 (1-based line 1), character 4
    assert_eq!(loc.range.start.line, 0);
    assert_eq!(loc.range.start.character, 4);
}

#[test]
fn test_lsp_jsonrpc_initialize() {
    let req_str = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}";
    let req: Request = serde_json::from_str(req_str).unwrap();

    let mut docs = HashMap::new();
    let mut output = Vec::new();
    super::handle_request(&req, &mut output, &mut docs).unwrap();

    let resp_str = String::from_utf8(output).unwrap();
    // Headers prefix "Content-Length: ...\r\n\r\n"
    assert!(resp_str.contains("Content-Length:"));
    assert!(resp_str.contains("\"result\":"));
    assert!(resp_str.contains("hoverProvider"));
    assert!(resp_str.contains("definitionProvider"));
}
