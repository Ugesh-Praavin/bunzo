//! Lexer test suite for the Bunzo compiler.
//!
//! Organized by category: keywords, identifiers, operators, literals,
//! comments, error cases, multi-token sequences, and edge cases.
//!
//! Target: 100+ individual test functions as required by the build plan.

use super::*;

// ── Test Helpers ──────────────────────────────────────────────────────────

/// Tokenizes source and asserts it produces exactly one token (plus Eof)
/// with the expected kind and lexeme.
fn assert_single_token(source: &str, expected_kind: TokenKind, expected_lexeme: &str) {
    let tokens = tokenize(source).unwrap_or_else(|e| panic!("tokenize failed: {e}"));
    assert_eq!(tokens.len(), 2, "expected 1 token + Eof, got {tokens:?}");
    assert_eq!(tokens[0].kind, expected_kind);
    assert_eq!(tokens[0].lexeme, expected_lexeme);
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

/// Tokenizes source and asserts the token kinds match (including trailing Eof).
fn assert_token_kinds(source: &str, expected: &[TokenKind]) {
    let tokens = tokenize(source).unwrap_or_else(|e| panic!("tokenize failed: {e}"));
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
    assert_eq!(kinds, expected, "source: {source:?}");
}

/// Tokenizes source and asserts that an error is returned.
fn assert_tokenize_error(source: &str, expected_code: &str) {
    let result = tokenize(source);
    assert!(result.is_err(), "expected error for {source:?}");
    let message = format!("{}", result.unwrap_err());
    assert!(
        message.contains(expected_code),
        "expected {expected_code} in error message: {message}"
    );
}

// ══════════════════════════════════════════════════════════════════════════
// KEYWORDS — Current (17 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn keyword_let() {
    assert_single_token("let", TokenKind::Let, "let");
}

#[test]
fn keyword_const() {
    assert_single_token("const", TokenKind::Const, "const");
}

#[test]
fn keyword_if() {
    assert_single_token("if", TokenKind::If, "if");
}

#[test]
fn keyword_else() {
    assert_single_token("else", TokenKind::Else, "else");
}

#[test]
fn keyword_while() {
    assert_single_token("while", TokenKind::While, "while");
}

#[test]
fn keyword_for() {
    assert_single_token("for", TokenKind::For, "for");
}

#[test]
fn keyword_in() {
    assert_single_token("in", TokenKind::In, "in");
}

#[test]
fn keyword_break() {
    assert_single_token("break", TokenKind::Break, "break");
}

#[test]
fn keyword_continue() {
    assert_single_token("continue", TokenKind::Continue, "continue");
}

#[test]
fn keyword_return() {
    assert_single_token("return", TokenKind::Return, "return");
}

#[test]
fn keyword_func() {
    assert_single_token("func", TokenKind::Func, "func");
}

#[test]
fn keyword_true() {
    assert_single_token("true", TokenKind::True, "true");
}

#[test]
fn keyword_false() {
    assert_single_token("false", TokenKind::False, "false");
}

#[test]
fn keyword_null() {
    assert_single_token("null", TokenKind::Null, "null");
}

#[test]
fn keyword_import() {
    assert_single_token("import", TokenKind::Import, "import");
}

#[test]
fn keyword_export() {
    assert_single_token("export", TokenKind::Export, "export");
}

#[test]
fn keyword_print() {
    assert_single_token("print", TokenKind::Print, "print");
}

// ══════════════════════════════════════════════════════════════════════════
// KEYWORDS — Reserved Future (11 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn keyword_class() {
    assert_single_token("class", TokenKind::Class, "class");
}

#[test]
fn keyword_enum() {
    assert_single_token("enum", TokenKind::Enum, "enum");
}

#[test]
fn keyword_struct() {
    assert_single_token("struct", TokenKind::Struct, "struct");
}

#[test]
fn keyword_interface() {
    assert_single_token("interface", TokenKind::Interface, "interface");
}

#[test]
fn keyword_async() {
    assert_single_token("async", TokenKind::Async, "async");
}

#[test]
fn keyword_await() {
    assert_single_token("await", TokenKind::Await, "await");
}

#[test]
fn keyword_match() {
    assert_single_token("match", TokenKind::Match, "match");
}

#[test]
fn keyword_switch() {
    assert_single_token("switch", TokenKind::Switch, "switch");
}

#[test]
fn keyword_try() {
    assert_single_token("try", TokenKind::Try, "try");
}

#[test]
fn keyword_catch() {
    assert_single_token("catch", TokenKind::Catch, "catch");
}

#[test]
fn keyword_throw() {
    assert_single_token("throw", TokenKind::Throw, "throw");
}

// ══════════════════════════════════════════════════════════════════════════
// IDENTIFIERS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn identifier_simple() {
    assert_single_token("myVar", TokenKind::Identifier, "myVar");
}

#[test]
fn identifier_with_underscore() {
    assert_single_token("_private", TokenKind::Identifier, "_private");
}

#[test]
fn identifier_with_digits() {
    assert_single_token("count2", TokenKind::Identifier, "count2");
}

#[test]
fn identifier_all_underscores() {
    assert_single_token("___", TokenKind::Identifier, "___");
}

#[test]
fn identifier_camel_case() {
    assert_single_token("isLoggedIn", TokenKind::Identifier, "isLoggedIn");
}

/// An identifier that starts with a keyword prefix must not be
/// misrecognized as a keyword. "letVar" is an identifier, not "let" + "Var".
#[test]
fn identifier_with_keyword_prefix() {
    assert_single_token("letVar", TokenKind::Identifier, "letVar");
}

// ══════════════════════════════════════════════════════════════════════════
// INTEGER LITERALS (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn integer_zero() {
    assert_single_token("0", TokenKind::IntegerLiteral, "0");
}

#[test]
fn integer_single_digit() {
    assert_single_token("7", TokenKind::IntegerLiteral, "7");
}

#[test]
fn integer_multi_digit() {
    assert_single_token("42", TokenKind::IntegerLiteral, "42");
}

#[test]
fn integer_large() {
    assert_single_token("12345", TokenKind::IntegerLiteral, "12345");
}

#[test]
fn integer_leading_zeros() {
    assert_single_token("007", TokenKind::IntegerLiteral, "007");
}

// ══════════════════════════════════════════════════════════════════════════
// FLOAT LITERALS (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn float_simple() {
    assert_single_token("3.14", TokenKind::FloatLiteral, "3.14");
}

#[test]
fn float_leading_zero() {
    assert_single_token("0.5", TokenKind::FloatLiteral, "0.5");
}

#[test]
fn float_long_fraction() {
    assert_single_token("99.99", TokenKind::FloatLiteral, "99.99");
}

#[test]
fn float_many_decimals() {
    assert_single_token("3.14159", TokenKind::FloatLiteral, "3.14159");
}

#[test]
fn float_single_decimal_digit() {
    assert_single_token("1.0", TokenKind::FloatLiteral, "1.0");
}

// ══════════════════════════════════════════════════════════════════════════
// STRING LITERALS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn string_simple() {
    assert_single_token("\"hello\"", TokenKind::StringLiteral, "hello");
}

#[test]
fn string_empty() {
    assert_single_token("\"\"", TokenKind::StringLiteral, "");
}

#[test]
fn string_with_spaces() {
    assert_single_token("\"hello world\"", TokenKind::StringLiteral, "hello world");
}

#[test]
fn string_with_digits() {
    assert_single_token("\"abc123\"", TokenKind::StringLiteral, "abc123");
}

#[test]
fn string_with_special_chars() {
    assert_single_token("\"a+b=c\"", TokenKind::StringLiteral, "a+b=c");
}

#[test]
fn string_with_newline_inside() {
    let source = "\"line1\nline2\"";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, "line1\nline2");
}

// ══════════════════════════════════════════════════════════════════════════
// ARITHMETIC OPERATORS (7 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn op_plus() {
    assert_single_token("+", TokenKind::Plus, "+");
}

#[test]
fn op_minus() {
    assert_single_token("-", TokenKind::Minus, "-");
}

#[test]
fn op_star() {
    assert_single_token("*", TokenKind::Star, "*");
}

#[test]
fn op_slash() {
    assert_single_token("/", TokenKind::Slash, "/");
}

#[test]
fn op_percent() {
    assert_single_token("%", TokenKind::Percent, "%");
}

#[test]
fn op_plus_plus() {
    assert_single_token("++", TokenKind::PlusPlus, "++");
}

#[test]
fn op_minus_minus() {
    assert_single_token("--", TokenKind::MinusMinus, "--");
}

// ══════════════════════════════════════════════════════════════════════════
// COMPARISON OPERATORS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn op_equal_equal() {
    assert_single_token("==", TokenKind::EqualEqual, "==");
}

#[test]
fn op_bang_equal() {
    assert_single_token("!=", TokenKind::BangEqual, "!=");
}

#[test]
fn op_less() {
    assert_single_token("<", TokenKind::Less, "<");
}

#[test]
fn op_greater() {
    assert_single_token(">", TokenKind::Greater, ">");
}

#[test]
fn op_less_equal() {
    assert_single_token("<=", TokenKind::LessEqual, "<=");
}

#[test]
fn op_greater_equal() {
    assert_single_token(">=", TokenKind::GreaterEqual, ">=");
}

// ══════════════════════════════════════════════════════════════════════════
// LOGICAL OPERATORS (3 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn op_and() {
    assert_single_token("&&", TokenKind::AmpersandAmpersand, "&&");
}

#[test]
fn op_or() {
    assert_single_token("||", TokenKind::PipePipe, "||");
}

#[test]
fn op_bang() {
    assert_single_token("!", TokenKind::Bang, "!");
}

// ══════════════════════════════════════════════════════════════════════════
// ASSIGNMENT OPERATORS (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn op_equal() {
    assert_single_token("=", TokenKind::Equal, "=");
}

#[test]
fn op_plus_equal() {
    assert_single_token("+=", TokenKind::PlusEqual, "+=");
}

#[test]
fn op_minus_equal() {
    assert_single_token("-=", TokenKind::MinusEqual, "-=");
}

#[test]
fn op_star_equal() {
    assert_single_token("*=", TokenKind::StarEqual, "*=");
}

#[test]
fn op_slash_equal() {
    assert_single_token("/=", TokenKind::SlashEqual, "/=");
}

// ══════════════════════════════════════════════════════════════════════════
// DELIMITERS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn delim_left_paren() {
    assert_single_token("(", TokenKind::LeftParen, "(");
}

#[test]
fn delim_right_paren() {
    assert_single_token(")", TokenKind::RightParen, ")");
}

#[test]
fn delim_left_brace() {
    assert_single_token("{", TokenKind::LeftBrace, "{");
}

#[test]
fn delim_right_brace() {
    assert_single_token("}", TokenKind::RightBrace, "}");
}

#[test]
fn delim_left_bracket() {
    assert_single_token("[", TokenKind::LeftBracket, "[");
}

#[test]
fn delim_right_bracket() {
    assert_single_token("]", TokenKind::RightBracket, "]");
}

// ══════════════════════════════════════════════════════════════════════════
// PUNCTUATION (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn punct_comma() {
    assert_single_token(",", TokenKind::Comma, ",");
}

#[test]
fn punct_dot() {
    assert_single_token(".", TokenKind::Dot, ".");
}

#[test]
fn punct_dot_dot() {
    assert_single_token("..", TokenKind::DotDot, "..");
}

#[test]
fn punct_semicolon() {
    assert_single_token(";", TokenKind::Semicolon, ";");
}

#[test]
fn punct_colon() {
    assert_single_token(":", TokenKind::Colon, ":");
}

// ══════════════════════════════════════════════════════════════════════════
// COMMENTS (6 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn comment_single_line_only() {
    let tokens = tokenize("// this is a comment").unwrap();
    assert_eq!(tokens.len(), 1, "comment should produce only Eof");
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn comment_single_line_before_code() {
    assert_token_kinds(
        "// comment\nlet",
        &[TokenKind::Let, TokenKind::Eof],
    );
}

#[test]
fn comment_block_only() {
    let tokens = tokenize("/* block comment */").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn comment_block_multiline() {
    assert_token_kinds(
        "/* line1\nline2 */\nlet",
        &[TokenKind::Let, TokenKind::Eof],
    );
}

#[test]
fn comment_block_between_tokens() {
    assert_token_kinds(
        "let /* comment */ x",
        &[TokenKind::Let, TokenKind::Identifier, TokenKind::Eof],
    );
}

#[test]
fn comment_single_line_at_end_of_code() {
    assert_token_kinds(
        "let x // comment",
        &[TokenKind::Let, TokenKind::Identifier, TokenKind::Eof],
    );
}

// ══════════════════════════════════════════════════════════════════════════
// WHITESPACE AND LINE TRACKING (5 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn whitespace_spaces_between_tokens() {
    assert_token_kinds(
        "let   x",
        &[TokenKind::Let, TokenKind::Identifier, TokenKind::Eof],
    );
}

#[test]
fn whitespace_tabs_between_tokens() {
    assert_token_kinds(
        "let\t\tx",
        &[TokenKind::Let, TokenKind::Identifier, TokenKind::Eof],
    );
}

#[test]
fn line_tracking_single_line() {
    let tokens = tokenize("let x = 42").unwrap();
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 1); // let
    assert_eq!(tokens[1].line, 1);
    assert_eq!(tokens[1].column, 5); // x
    assert_eq!(tokens[2].line, 1);
    assert_eq!(tokens[2].column, 7); // =
    assert_eq!(tokens[3].line, 1);
    assert_eq!(tokens[3].column, 9); // 42
}

#[test]
fn line_tracking_multi_line() {
    let tokens = tokenize("let\nx").unwrap();
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 1); // let
    assert_eq!(tokens[1].line, 2);
    assert_eq!(tokens[1].column, 1); // x
}

#[test]
fn carriage_return_ignored() {
    assert_token_kinds(
        "let\r\nx",
        &[TokenKind::Let, TokenKind::Identifier, TokenKind::Eof],
    );
}

// ══════════════════════════════════════════════════════════════════════════
// ERROR CASES (7 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn error_unexpected_character_at_sign() {
    assert_tokenize_error("@", "BZ0003");
}

#[test]
fn error_unexpected_character_hash() {
    assert_tokenize_error("#", "BZ0003");
}

#[test]
fn error_unexpected_character_tilde() {
    assert_tokenize_error("~", "BZ0003");
}

#[test]
fn error_lone_ampersand() {
    assert_tokenize_error("&", "BZ0003");
}

#[test]
fn error_lone_pipe() {
    assert_tokenize_error("|", "BZ0003");
}

#[test]
fn error_unterminated_string() {
    assert_tokenize_error("\"hello", "BZ0004");
}

#[test]
fn error_unterminated_block_comment() {
    assert_tokenize_error("/* open forever", "BZ0005");
}

// ══════════════════════════════════════════════════════════════════════════
// MULTI-TOKEN SEQUENCES (12 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn sequence_print_hello() {
    assert_token_kinds(
        "print(\"Hello\")",
        &[
            TokenKind::Print,
            TokenKind::LeftParen,
            TokenKind::StringLiteral,
            TokenKind::RightParen,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_variable_declaration() {
    assert_token_kinds(
        "let x = 42",
        &[
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::IntegerLiteral,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_const_declaration() {
    assert_token_kinds(
        "const PI = 3.14",
        &[
            TokenKind::Const,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::FloatLiteral,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_for_in_range() {
    assert_token_kinds(
        "for i in 1..10",
        &[
            TokenKind::For,
            TokenKind::Identifier,
            TokenKind::In,
            TokenKind::IntegerLiteral,
            TokenKind::DotDot,
            TokenKind::IntegerLiteral,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_if_else() {
    assert_token_kinds(
        "if x >= 18 { } else { }",
        &[
            TokenKind::If,
            TokenKind::Identifier,
            TokenKind::GreaterEqual,
            TokenKind::IntegerLiteral,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Else,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_while_loop() {
    assert_token_kinds(
        "while x < 10 { x++ }",
        &[
            TokenKind::While,
            TokenKind::Identifier,
            TokenKind::Less,
            TokenKind::IntegerLiteral,
            TokenKind::LeftBrace,
            TokenKind::Identifier,
            TokenKind::PlusPlus,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_function_declaration() {
    assert_token_kinds(
        "func greet(name) { }",
        &[
            TokenKind::Func,
            TokenKind::Identifier,
            TokenKind::LeftParen,
            TokenKind::Identifier,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_arithmetic_expression() {
    assert_token_kinds(
        "a + b * c - d / e % f",
        &[
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::Identifier,
            TokenKind::Star,
            TokenKind::Identifier,
            TokenKind::Minus,
            TokenKind::Identifier,
            TokenKind::Slash,
            TokenKind::Identifier,
            TokenKind::Percent,
            TokenKind::Identifier,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_comparison_chain() {
    assert_token_kinds(
        "a == b != c",
        &[
            TokenKind::Identifier,
            TokenKind::EqualEqual,
            TokenKind::Identifier,
            TokenKind::BangEqual,
            TokenKind::Identifier,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_logical_expression() {
    assert_token_kinds(
        "a && b || !c",
        &[
            TokenKind::Identifier,
            TokenKind::AmpersandAmpersand,
            TokenKind::Identifier,
            TokenKind::PipePipe,
            TokenKind::Bang,
            TokenKind::Identifier,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_compound_assignment() {
    assert_token_kinds(
        "x += 1",
        &[
            TokenKind::Identifier,
            TokenKind::PlusEqual,
            TokenKind::IntegerLiteral,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn sequence_array_literal() {
    assert_token_kinds(
        "[1, 2, 3]",
        &[
            TokenKind::LeftBracket,
            TokenKind::IntegerLiteral,
            TokenKind::Comma,
            TokenKind::IntegerLiteral,
            TokenKind::Comma,
            TokenKind::IntegerLiteral,
            TokenKind::RightBracket,
            TokenKind::Eof,
        ],
    );
}

// ══════════════════════════════════════════════════════════════════════════
// EDGE CASES (8 tests)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_source() {
    let tokens = tokenize("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn edge_only_whitespace() {
    let tokens = tokenize("   \t\n\n  ").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn edge_only_comment() {
    let tokens = tokenize("// nothing here").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn edge_eof_position_after_content() {
    let tokens = tokenize("x").unwrap();
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    assert_eq!(tokens.last().unwrap().line, 1);
    assert_eq!(tokens.last().unwrap().column, 2);
}

#[test]
fn edge_adjacent_operators() {
    // Two separate tokens without whitespace.
    assert_token_kinds(
        "()",
        &[TokenKind::LeftParen, TokenKind::RightParen, TokenKind::Eof],
    );
}

#[test]
fn edge_number_followed_by_dot_dot() {
    // `1..10` should be int, dotdot, int — not a float.
    assert_token_kinds(
        "1..10",
        &[
            TokenKind::IntegerLiteral,
            TokenKind::DotDot,
            TokenKind::IntegerLiteral,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn edge_keyword_as_identifier_prefix() {
    // "formats" starts with "for" but is an identifier.
    assert_single_token("formats", TokenKind::Identifier, "formats");
}

#[test]
fn edge_print_hello_bunzo_lexemes() {
    let tokens = tokenize("print(\"Hello Bunzo\")").unwrap();
    assert_eq!(tokens.len(), 5);

    assert_eq!(tokens[0].kind, TokenKind::Print);
    assert_eq!(tokens[0].lexeme, "print");
    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 1);

    assert_eq!(tokens[1].kind, TokenKind::LeftParen);
    assert_eq!(tokens[1].column, 6);

    assert_eq!(tokens[2].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[2].lexeme, "Hello Bunzo");
    assert_eq!(tokens[2].column, 7);

    assert_eq!(tokens[3].kind, TokenKind::RightParen);
    assert_eq!(tokens[3].column, 20);

    assert_eq!(tokens[4].kind, TokenKind::Eof);
    assert_eq!(tokens[4].column, 21);
}
