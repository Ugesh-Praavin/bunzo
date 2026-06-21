//! Lexical analysis (tokenization) for Bunzo source code.
//!
//! This module converts raw source text into a stream of [`Token`]s.
//! It handles keywords, identifiers, literals, operators, comments,
//! and whitespace as specified in the Bunzo language specification.
//!
//! # Usage
//!
//! ```ignore
//! use crate::lexer::tokenize;
//!
//! let tokens = tokenize("let x = 42")?;
//! ```

pub mod token;

pub use token::{Token, TokenKind};

use crate::diagnostics::CompilerError;
use token::lookup_keyword;

/// Tokenizes Bunzo source code into a vector of tokens.
///
/// The returned token stream always ends with a [`TokenKind::Eof`] token.
///
/// # Errors
///
/// Returns a [`CompilerError`] if the source contains:
/// - An unrecognized character ([`CompilerError::UnexpectedCharacter`])
/// - An unterminated string literal ([`CompilerError::UnterminatedString`])
/// - An unterminated block comment ([`CompilerError::UnterminatedComment`])
pub fn tokenize(source: &str) -> Result<Vec<Token>, CompilerError> {
    let mut lexer = Lexer::new(source);
    lexer.scan_tokens()
}

// ── Internal Lexer ────────────────────────────────────────────────────────

/// Internal lexer state. Not exposed publicly — callers use [`tokenize`].
struct Lexer {
    /// Source code as a character array for random access.
    source: Vec<char>,
    /// Current read position in `source`.
    position: usize,
    /// Current 1-based line number.
    line: usize,
    /// Current 1-based column number.
    column: usize,
    /// Accumulated output tokens.
    tokens: Vec<Token>,
}

impl Lexer {
    /// Creates a new lexer for the given source text.
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    /// Scans all tokens from the source and returns them.
    fn scan_tokens(&mut self) -> Result<Vec<Token>, CompilerError> {
        while !self.is_at_end() {
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenKind::Eof, "", self.line, self.column));
        Ok(self.tokens.clone())
    }

    /// Scans a single token (after skipping whitespace).
    fn scan_token(&mut self) -> Result<(), CompilerError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(());
        }

        let start_line = self.line;
        let start_col = self.column;
        let c = self.advance();

        match c {
            // ── Delimiters ────────────────────────────────────────────
            '(' => self.push(TokenKind::LeftParen, "(", start_line, start_col),
            ')' => self.push(TokenKind::RightParen, ")", start_line, start_col),
            '{' => self.push(TokenKind::LeftBrace, "{", start_line, start_col),
            '}' => self.push(TokenKind::RightBrace, "}", start_line, start_col),
            '[' => self.push(TokenKind::LeftBracket, "[", start_line, start_col),
            ']' => self.push(TokenKind::RightBracket, "]", start_line, start_col),

            // ── Punctuation ───────────────────────────────────────────
            ',' => self.push(TokenKind::Comma, ",", start_line, start_col),
            ';' => self.push(TokenKind::Semicolon, ";", start_line, start_col),
            '%' => self.push(TokenKind::Percent, "%", start_line, start_col),

            // ── Dot / DotDot ──────────────────────────────────────────
            '.' => {
                if self.peek() == '.' {
                    self.advance();
                    self.push(TokenKind::DotDot, "..", start_line, start_col);
                } else {
                    self.push(TokenKind::Dot, ".", start_line, start_col);
                }
            }

            // ── Colon / DoubleColon ───────────────────────────────────
            ':' => {
                if self.peek() == ':' {
                    self.advance();
                    self.push(TokenKind::DoubleColon, "::", start_line, start_col);
                } else {
                    self.push(TokenKind::Colon, ":", start_line, start_col);
                }
            }

            // ── QuestionMark ──────────────────────────────────────────
            '?' => self.push(TokenKind::QuestionMark, "?", start_line, start_col), // ── Plus / PlusPlus / PlusEqual ────────────────────────────
            '+' => {
                if self.peek() == '+' {
                    self.advance();
                    self.push(TokenKind::PlusPlus, "++", start_line, start_col);
                } else if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::PlusEqual, "+=", start_line, start_col);
                } else {
                    self.push(TokenKind::Plus, "+", start_line, start_col);
                }
            }

            // ── Minus / MinusMinus / MinusEqual ────────────────────────
            '-' => {
                if self.peek() == '-' {
                    self.advance();
                    self.push(TokenKind::MinusMinus, "--", start_line, start_col);
                } else if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::MinusEqual, "-=", start_line, start_col);
                } else if self.peek() == '>' {
                    self.advance();
                    self.push(TokenKind::Arrow, "->", start_line, start_col);
                } else {
                    self.push(TokenKind::Minus, "-", start_line, start_col);
                }
            }

            // ── Star / StarEqual ──────────────────────────────────────
            '*' => {
                if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::StarEqual, "*=", start_line, start_col);
                } else {
                    self.push(TokenKind::Star, "*", start_line, start_col);
                }
            }

            // ── Slash / SlashEqual / Comments ─────────────────────────
            '/' => {
                if self.peek() == '/' {
                    self.skip_line_comment();
                } else if self.peek() == '*' {
                    self.skip_block_comment(start_line, start_col)?;
                } else if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::SlashEqual, "/=", start_line, start_col);
                } else {
                    self.push(TokenKind::Slash, "/", start_line, start_col);
                }
            }

            // ── Equal / EqualEqual / FatArrow ────────────────────────
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::EqualEqual, "==", start_line, start_col);
                } else if self.peek() == '>' {
                    self.advance();
                    self.push(TokenKind::FatArrow, "=>", start_line, start_col);
                } else {
                    self.push(TokenKind::Equal, "=", start_line, start_col);
                }
            }

            // ── Bang / BangEqual ──────────────────────────────────────
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::BangEqual, "!=", start_line, start_col);
                } else {
                    self.push(TokenKind::Bang, "!", start_line, start_col);
                }
            }

            // ── Less / LessEqual ──────────────────────────────────────
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::LessEqual, "<=", start_line, start_col);
                } else {
                    self.push(TokenKind::Less, "<", start_line, start_col);
                }
            }

            // ── Greater / GreaterEqual ────────────────────────────────
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    self.push(TokenKind::GreaterEqual, ">=", start_line, start_col);
                } else {
                    self.push(TokenKind::Greater, ">", start_line, start_col);
                }
            }

            // ── AmpersandAmpersand / Ampersand ────────────────────────
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    self.push(TokenKind::AmpersandAmpersand, "&&", start_line, start_col);
                } else {
                    return Err(CompilerError::UnexpectedCharacter {
                        character: '&',
                        line: start_line,
                        column: start_col,
                    });
                }
            }

            // ── PipePipe ──────────────────────────────────────────────
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    self.push(TokenKind::PipePipe, "||", start_line, start_col);
                } else {
                    return Err(CompilerError::UnexpectedCharacter {
                        character: '|',
                        line: start_line,
                        column: start_col,
                    });
                }
            }

            // ── String Literals ───────────────────────────────────────
            '"' => self.scan_string(start_line, start_col)?,

            // ── Number Literals ───────────────────────────────────────
            c if c.is_ascii_digit() => self.scan_number(c, start_line, start_col),

            // ── Identifiers and Keywords ──────────────────────────────
            c if is_identifier_start(c) => self.scan_identifier(c, start_line, start_col),

            // ── Unrecognized Character ────────────────────────────────
            _ => {
                return Err(CompilerError::UnexpectedCharacter {
                    character: c,
                    line: start_line,
                    column: start_col,
                });
            }
        }

        Ok(())
    }

    // ── Character Navigation ──────────────────────────────────────────────

    /// Consumes and returns the current character, advancing the position.
    fn advance(&mut self) -> char {
        let c = self.source[self.position];
        self.position += 1;
        self.column += 1;
        c
    }

    /// Returns the current character without consuming it, or `'\0'` at end.
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.position]
        }
    }

    /// Returns the character after the current one, or `'\0'` if unavailable.
    fn peek_next(&self) -> char {
        if self.position + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.position + 1]
        }
    }

    /// Returns `true` if the lexer has consumed all source characters.
    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Pushes a token onto the output list.
    fn push(&mut self, kind: TokenKind, lexeme: &str, line: usize, column: usize) {
        self.tokens.push(Token::new(kind, lexeme, line, column));
    }

    // ── Whitespace and Comments ───────────────────────────────────────────

    /// Skips whitespace characters, updating line/column as needed.
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.source[self.position] {
                ' ' | '\t' | '\r' => {
                    self.position += 1;
                    self.column += 1;
                }
                '\n' => {
                    self.position += 1;
                    self.line += 1;
                    self.column = 1;
                }
                _ => break,
            }
        }
    }

    /// Skips a single-line comment (`// ...` to end of line).
    ///
    /// Assumes the first `/` has already been consumed by [`scan_token`].
    fn skip_line_comment(&mut self) {
        // Consume the second '/'.
        self.advance();

        // Skip everything until newline or end of file.
        while !self.is_at_end() && self.source[self.position] != '\n' {
            self.position += 1;
            self.column += 1;
        }
        // The newline itself will be consumed by skip_whitespace on the
        // next call to scan_token.
    }

    /// Skips a block comment (`/* ... */`).
    ///
    /// Assumes the first `/` has already been consumed by [`scan_token`].
    fn skip_block_comment(
        &mut self,
        start_line: usize,
        start_col: usize,
    ) -> Result<(), CompilerError> {
        // Consume the '*' after '/'.
        self.advance();

        while !self.is_at_end() {
            if self.source[self.position] == '*' && self.peek_next() == '/' {
                // Consume '*' and '/'.
                self.advance();
                self.advance();
                return Ok(());
            }

            if self.source[self.position] == '\n' {
                self.position += 1;
                self.line += 1;
                self.column = 1;
            } else {
                self.position += 1;
                self.column += 1;
            }
        }

        Err(CompilerError::UnterminatedComment {
            line: start_line,
            column: start_col,
        })
    }

    // ── Literal Scanners ──────────────────────────────────────────────────

    /// Scans a string literal (the opening `"` has already been consumed).
    fn scan_string(&mut self, start_line: usize, start_col: usize) -> Result<(), CompilerError> {
        let mut value = String::new();

        while !self.is_at_end() && self.source[self.position] != '"' {
            if self.source[self.position] == '\n' {
                value.push('\n');
                self.position += 1;
                self.line += 1;
                self.column = 1;
            } else {
                value.push(self.source[self.position]);
                self.position += 1;
                self.column += 1;
            }
        }

        if self.is_at_end() {
            return Err(CompilerError::UnterminatedString {
                line: start_line,
                column: start_col,
            });
        }

        // Consume the closing '"'.
        self.position += 1;
        self.column += 1;

        self.push(TokenKind::StringLiteral, &value, start_line, start_col);
        Ok(())
    }

    /// Scans a number literal (integer or float).
    ///
    /// The first digit has already been consumed by [`scan_token`].
    fn scan_number(&mut self, first_digit: char, start_line: usize, start_col: usize) {
        let mut lexeme = String::new();
        lexeme.push(first_digit);

        // Consume remaining integer digits.
        while !self.is_at_end() && self.source[self.position].is_ascii_digit() {
            lexeme.push(self.source[self.position]);
            self.position += 1;
            self.column += 1;
        }

        // Check for a fractional part: `.` followed by at least one digit.
        // This distinguishes `1.5` (float) from `1..10` (int, dotdot, int).
        let is_float = !self.is_at_end()
            && self.source[self.position] == '.'
            && self.peek_next().is_ascii_digit();

        if is_float {
            lexeme.push('.');
            self.position += 1;
            self.column += 1;

            while !self.is_at_end() && self.source[self.position].is_ascii_digit() {
                lexeme.push(self.source[self.position]);
                self.position += 1;
                self.column += 1;
            }
        }

        let kind = if is_float {
            TokenKind::FloatLiteral
        } else {
            TokenKind::IntegerLiteral
        };

        self.push(kind, &lexeme, start_line, start_col);
    }

    /// Scans an identifier or keyword.
    ///
    /// The first character has already been consumed by [`scan_token`].
    fn scan_identifier(&mut self, first_char: char, start_line: usize, start_col: usize) {
        let mut lexeme = String::new();
        lexeme.push(first_char);

        while !self.is_at_end() && is_identifier_continue(self.source[self.position]) {
            lexeme.push(self.source[self.position]);
            self.position += 1;
            self.column += 1;
        }

        let kind = lookup_keyword(&lexeme).unwrap_or(TokenKind::Identifier);
        self.push(kind, &lexeme, start_line, start_col);
    }
}

// ── Helper Functions ──────────────────────────────────────────────────────

/// Returns `true` if `c` can start an identifier (`a-z`, `A-Z`, `_`).
fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Returns `true` if `c` can continue an identifier (`a-z`, `A-Z`, `0-9`, `_`).
fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
