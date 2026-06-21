//! Recursive descent parser for Bunzo source code.
//!
//! This module converts a token stream (produced by the lexer) into an
//! Abstract Syntax Tree. It implements a recursive descent parser with
//! one function per grammar precedence level.
//!
//! # Grammar (Phase 7)
//!
//! ```text
//! program        → statement* EOF
//! statement      → let_decl | const_decl | print_stmt | if_stmt
//!                  | while_stmt | for_stmt | break_stmt | cont_stmt
//!                  | assign_stmt | expr_stmt
//! let_decl       → "let" IDENTIFIER "=" expression
//! const_decl     → "const" IDENTIFIER "=" expression
//! print_stmt     → "print" "(" expression ")"
//! if_stmt        → "if" expression block ( "else" ( if_stmt | block ) )?
//! while_stmt     → "while" expression block
//! for_stmt       → "for" IDENTIFIER "in" range_expr block
//! range_expr     → expression ( ".." | "..=" ) expression
//! break_stmt     → "break"
//! cont_stmt      → "continue"
//! assign_stmt    → IDENTIFIER "=" expression   (when IDENTIFIER followed by "=")
//! block          → "{" statement* "}"
//! expr_stmt      → expression
//!
//! expression     → logic_or
//! logic_or       → logic_and ( "||" logic_and )*
//! logic_and      → equality ( "&&" equality )*
//! equality       → comparison ( ( "==" | "!=" ) comparison )*
//! comparison     → addition ( ( "<" | ">" | "<=" | ">=" ) addition )*
//! addition       → multiplication ( ( "+" | "-" ) multiplication )*
//! multiplication → unary ( ( "*" | "/" | "%" ) unary )*
//! unary          → ( "!" | "-" ) unary | primary
//! primary        → INTEGER | FLOAT | STRING | "true" | "false" | "null"
//!                  | IDENTIFIER | "(" expression ")"
//!                  | primary ".." primary  (range, inside for)
//! ```

use crate::ast::{BinaryOperator, Block, Expression, Program, Statement, UnaryOperator};
use crate::diagnostics::CompilerError;
use crate::lexer::{Token, TokenKind};

/// Parses a token stream into an AST.
///
/// The token stream must end with a [`TokenKind::Eof`] token (as
/// produced by [`crate::lexer::tokenize`]).
///
/// # Errors
///
/// Returns a [`CompilerError`] on the first syntax error encountered.
pub fn parse(mut tokens: Vec<Token>) -> Result<Program, CompilerError> {
    // Be defensive: ensure the stream terminates with EOF to avoid panics when
    // parsing token streams not produced by `lexer::tokenize`.
    if tokens.last().map(|t| &t.kind) != Some(&TokenKind::Eof) {
        let (line, column) = tokens.last().map(|t| (t.line, t.column)).unwrap_or((1, 1));
        tokens.push(Token::new(TokenKind::Eof, "", line, column));
    }

    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ── Internal Parser ───────────────────────────────────────────────────────

/// Internal parser state. Not exposed publicly — callers use [`parse`].
struct Parser {
    /// The token stream from the lexer.
    tokens: Vec<Token>,
    /// Current read position in `tokens`.
    position: usize,
}

impl Parser {
    /// Creates a new parser for the given token stream.
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    // ── Top-Level ─────────────────────────────────────────────────────

    /// Parses a complete program (sequence of statements until Eof).
    fn parse_program(&mut self) -> Result<Program, CompilerError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    // ── Statements ────────────────────────────────────────────────────

    /// Parses a single statement, dispatching on the leading token.
    fn parse_statement(&mut self) -> Result<Statement, CompilerError> {
        match self.peek().kind {
            TokenKind::Let => self.parse_let_declaration(),
            TokenKind::Const => self.parse_const_declaration(),
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            // `identifier =` is an assignment; everything else is expr_stmt
            TokenKind::Identifier if self.peek_next_is(&TokenKind::Equal) => {
                self.parse_assign_statement()
            }
            _ => self.parse_expression_statement(),
        }
    }

    /// Parses `let name = expression`.
    fn parse_let_declaration(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "variable name")?;
        let name = name_token.lexeme;

        self.expect(TokenKind::Equal, "'='")?;

        let initializer = self.parse_expression()?;

        Ok(Statement::LetDeclaration {
            name,
            initializer,
            line,
            column,
        })
    }

    /// Parses `const name = expression`.
    fn parse_const_declaration(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "variable name")?;
        let name = name_token.lexeme;

        self.expect(TokenKind::Equal, "'='")?;

        let initializer = self.parse_expression()?;

        Ok(Statement::ConstDeclaration {
            name,
            initializer,
            line,
            column,
        })
    }

    /// Parses `print( expression )`.
    fn parse_print_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        self.expect(TokenKind::LeftParen, "'('")?;
        let argument = self.parse_expression()?;
        self.expect(TokenKind::RightParen, "')'")?;

        Ok(Statement::PrintStatement {
            argument,
            line,
            column,
        })
    }

    /// Parses `if expression block ( "else" ( if_stmt | block ) )?`.
    ///
    /// `else if` chains are handled naturally: after `else`, if the next token
    /// is `if` we parse another `if_stmt` and wrap it in a single-element block,
    /// otherwise we parse a plain block.
    fn parse_if_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance(); // consume `if`
        let line = keyword.line;
        let column = keyword.column;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.check(&TokenKind::Else) {
            self.advance(); // consume `else`

            if self.check(&TokenKind::If) {
                // `else if` — parse the nested if and wrap it in a synthetic block
                let else_if_line = self.peek().line;
                let else_if_col = self.peek().column;
                let nested = self.parse_if_statement()?;
                Some(Block {
                    statements: vec![nested],
                    line: else_if_line,
                    column: else_if_col,
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::IfStatement {
            condition,
            then_branch,
            else_branch,
            line,
            column,
        })
    }

    /// Parses a block: `"{" statement* "}"`.
    fn parse_block(&mut self) -> Result<Block, CompilerError> {
        let open = self.expect(TokenKind::LeftBrace, "'{'")?;
        let line = open.line;
        let column = open.column;

        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace, "'}'")?;

        Ok(Block {
            statements,
            line,
            column,
        })
    }

    /// Parses `while expression block`.
    fn parse_while_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance(); // consume `while`
        let line = keyword.line;
        let column = keyword.column;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::WhileStatement {
            condition,
            body,
            line,
            column,
        })
    }

    /// Parses `for IDENTIFIER in range_expr block`.
    fn parse_for_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance(); // consume `for`
        let line = keyword.line;
        let column = keyword.column;

        let var_token = self.expect(TokenKind::Identifier, "loop variable name")?;
        let variable = var_token.lexeme;

        self.expect(TokenKind::In, "'in'")?;

        // Parse the range expression: start (..|..=) end
        let iterable = self.parse_range_expression()?;

        let body = self.parse_block()?;

        Ok(Statement::ForInStatement {
            variable,
            iterable,
            body,
            line,
            column,
        })
    }

    /// Parses a range expression: `expression ( ".." | "..=" ) expression`.
    ///
    /// This is only called from `parse_for_statement`; it is not part of the
    /// general expression hierarchy to keep range syntax unambiguous.
    fn parse_range_expression(&mut self) -> Result<Expression, CompilerError> {
        let start = self.parse_addition()?; // parse start (no logic operators in ranges)

let op_token = if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEqual) {
    self.advance()
} else {
    return Err(CompilerError::UnexpectedToken {
        expected: "'..' or '..='".to_string(),
        found: describe_token(self.peek()),
        line: self.peek().line,
        column: self.peek().column,
    });
};

let inclusive = matches!(op_token.kind, TokenKind::DotDotEqual);
let op_line = op_token.line;
let op_column = op_token.column;
let end = self.parse_addition()?;

Ok(Expression::Range {
    start: Box::new(start),
    end: Box::new(end),
    inclusive,
    line: op_line,
    column: op_column,
})
    }

    /// Parses `break`.
    fn parse_break_statement(&mut self) -> Result<Statement, CompilerError> {
        let tok = self.advance();
        Ok(Statement::Break {
            line: tok.line,
            column: tok.column,
        })
    }

    /// Parses `continue`.
    fn parse_continue_statement(&mut self) -> Result<Statement, CompilerError> {
        let tok = self.advance();
        Ok(Statement::Continue {
            line: tok.line,
            column: tok.column,
        })
    }

    /// Parses `IDENTIFIER = expression` (variable reassignment).
    fn parse_assign_statement(&mut self) -> Result<Statement, CompilerError> {
        let id_token = self.advance(); // consume identifier
        let name = id_token.lexeme;
        let line = id_token.line;
        let column = id_token.column;

        self.expect(TokenKind::Equal, "'='")?; // consume `=`
        let value = self.parse_expression()?;

        Ok(Statement::AssignStatement {
            name,
            value,
            line,
            column,
        })
    }

    /// Parses a bare expression as a statement.
    fn parse_expression_statement(&mut self) -> Result<Statement, CompilerError> {
        let expression = self.parse_expression()?;
        Ok(Statement::ExpressionStatement { expression })
    }

    // ── Expressions (by precedence, lowest first) ─────────────────────

    /// Parses an expression (entry point — lowest precedence).
    fn parse_expression(&mut self) -> Result<Expression, CompilerError> {
        self.parse_logic_or()
    }

    /// Parses `logic_and ( "||" logic_and )*`.
    fn parse_logic_or(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_logic_and()?;

        while self.check(&TokenKind::PipePipe) {
            let op_token = self.advance();
            let right = self.parse_logic_and()?;
            left = Expression::BinaryOp {
                operator: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `equality ( "&&" equality )*`.
    fn parse_logic_and(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::AmpersandAmpersand) {
            let op_token = self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                operator: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `comparison ( ( "==" | "!=" ) comparison )*`.
    fn parse_equality(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_comparison()?;

        while self.check(&TokenKind::EqualEqual) || self.check(&TokenKind::BangEqual) {
            let op_token = self.advance();
            let operator = match op_token.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `addition ( ( "<" | ">" | "<=" | ">=" ) addition )*`.
    fn parse_comparison(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_addition()?;

        while self.check(&TokenKind::Less)
            || self.check(&TokenKind::Greater)
            || self.check(&TokenKind::LessEqual)
            || self.check(&TokenKind::GreaterEqual)
        {
            let op_token = self.advance();
            let operator = match op_token.kind {
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_addition()?;
            left = Expression::BinaryOp {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `multiplication ( ( "+" | "-" ) multiplication )*`.
    fn parse_addition(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_multiplication()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op_token = self.advance();
            let operator = match op_token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_multiplication()?;
            left = Expression::BinaryOp {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `unary ( ( "*" | "/" | "%" ) unary )*`.
    fn parse_multiplication(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_unary()?;

        while self.check(&TokenKind::Star)
            || self.check(&TokenKind::Slash)
            || self.check(&TokenKind::Percent)
        {
            let op_token = self.advance();
            let operator = match op_token.kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                line: op_token.line,
                column: op_token.column,
            };
        }

        Ok(left)
    }

    /// Parses `( "!" | "-" ) unary | primary`.
    fn parse_unary(&mut self) -> Result<Expression, CompilerError> {
        if self.check(&TokenKind::Bang) || self.check(&TokenKind::Minus) {
            let op_token = self.advance();
            let operator = match op_token.kind {
                TokenKind::Bang => UnaryOperator::LogicalNot,
                TokenKind::Minus => UnaryOperator::Negate,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                operator,
                operand: Box::new(operand),
                line: op_token.line,
                column: op_token.column,
            });
        }

        self.parse_primary()
    }

    /// Parses a primary expression (literals, identifiers, grouping).
    fn parse_primary(&mut self) -> Result<Expression, CompilerError> {
        let token = self.peek().clone();

        match token.kind {
            TokenKind::IntegerLiteral => {
                self.advance();
                let value: i64 =
                    token
                        .lexeme
                        .parse()
                        .map_err(|_| CompilerError::UnexpectedToken {
                            expected: "valid integer".to_string(),
                            found: describe_token(&token),
                            line: token.line,
                            column: token.column,
                        })?;
                Ok(Expression::IntegerLiteral {
                    value,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::FloatLiteral => {
                self.advance();
                let value: f64 =
                    token
                        .lexeme
                        .parse()
                        .map_err(|_| CompilerError::UnexpectedToken {
                            expected: "valid float".to_string(),
                            found: describe_token(&token),
                            line: token.line,
                            column: token.column,
                        })?;
                Ok(Expression::FloatLiteral {
                    value,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::StringLiteral => {
                self.advance();
                Ok(Expression::StringLiteral {
                    value: token.lexeme,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::True => {
                self.advance();
                Ok(Expression::BooleanLiteral {
                    value: true,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::False => {
                self.advance();
                Ok(Expression::BooleanLiteral {
                    value: false,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::Null => {
                self.advance();
                Ok(Expression::NullLiteral {
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::Identifier => {
                self.advance();
                Ok(Expression::Identifier {
                    name: token.lexeme,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen, "')'")?;
                Ok(Expression::Grouping {
                    expression: Box::new(expr),
                    line: token.line,
                    column: token.column,
                })
            }

            _ => Err(CompilerError::ExpectedExpression {
                found: describe_token(&token),
                line: token.line,
                column: token.column,
            }),
        }
    }

    // ── Token Navigation ──────────────────────────────────────────────

    /// Returns a reference to the current token without consuming it.
    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    /// Consumes and returns a clone of the current token.
    ///
    /// Does not advance past `Eof`.
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        if !self.is_at_end() {
            self.position += 1;
        }
        token
    }

    /// Returns `true` if the current token is `Eof`.
    fn is_at_end(&self) -> bool {
        self.tokens[self.position].kind == TokenKind::Eof
    }

    /// Returns `true` if the current token's kind matches `kind`.
    fn check(&self, kind: &TokenKind) -> bool {
        self.peek().kind == *kind
    }

    /// Returns `true` if the **next** token (position + 1) matches `kind`.
    ///
    /// Used to distinguish `identifier = value` (assignment) from an identifier
    /// used as an expression, without consuming any tokens.
    fn peek_next_is(&self, kind: &TokenKind) -> bool {
        let next_pos = self.position + 1;
        if next_pos < self.tokens.len() {
            self.tokens[next_pos].kind == *kind
        } else {
            false
        }
    }

    /// Consumes the current token if it matches `kind`, otherwise
    /// returns a [`CompilerError::UnexpectedToken`].
    fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token, CompilerError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            Err(CompilerError::UnexpectedToken {
                expected: expected.to_string(),
                found: describe_token(token),
                line: token.line,
                column: token.column,
            })
        }
    }
}

// ── Helper Functions ──────────────────────────────────────────────────────

/// Returns a human-readable description of a token for error messages.
fn describe_token(token: &Token) -> String {
    match token.kind {
        TokenKind::Eof => "end of file".to_string(),
        _ => format!("'{}'", token.lexeme),
    }
}
