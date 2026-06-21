//! Recursive descent parser for Bunzo source code.
//!
//! This module converts a token stream (produced by the lexer) into an
//! Abstract Syntax Tree. It implements a recursive descent parser with
//! one function per grammar precedence level.
//!
//! # Grammar (Phase 3 + Phase 1 functions + control flow)
//!
//! ```text
//! program        → statement* EOF
//! statement      → let_decl | const_decl | print_stmt | func_decl
//!                  | return_stmt | if_stmt | while_stmt | for_stmt
//!                  | break_stmt | continue_stmt | assign_stmt | expr_stmt
//! let_decl       → "let" IDENTIFIER "=" expression
//! const_decl     → "const" IDENTIFIER "=" expression
//! print_stmt     → "print" "(" expression ")"
//! func_decl      → "func" IDENTIFIER "(" params? ")" ( "->" IDENTIFIER )? block
//! params         → param ( "," param )*
//! param          → IDENTIFIER ":" IDENTIFIER
//! return_stmt    → "return" expression?
//! if_stmt        → "if" expression block ( "else" ( if_stmt | block ) )?
//! while_stmt     → "while" expression block
//! for_stmt       → "for" IDENTIFIER "in" expression ".." expression block
//! break_stmt     → "break"
//! continue_stmt  → "continue"
//! assign_stmt    → IDENTIFIER "=" expression
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
//! unary          → ( "!" | "-" ) unary | call
//! call           → primary ( "(" arguments? ")" )*
//! arguments      → expression ( "," expression )*
//! primary        → INTEGER | FLOAT | STRING | "true" | "false" | "null"
//!                  | IDENTIFIER | "(" expression ")"
//! ```

use crate::ast::{
    BinaryOperator, Expression, Parameter, Program, Statement, UnaryOperator,
};
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
    /// `true` while parsing an expression context where a `{` must NOT
    /// be interpreted as the start of a struct literal (e.g. an `if` or
    /// `while` condition, where `{` instead opens the statement block).
    ///
    /// This mirrors the standard "no struct literals in condition
    /// position" rule used by languages with the same ambiguity.
    no_struct_literal: bool,
}

impl Parser {
    /// Creates a new parser for the given token stream.
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            no_struct_literal: false,
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
            TokenKind::Let | TokenKind::Var => self.parse_let_declaration(),
            TokenKind::Const => self.parse_const_declaration(),
            TokenKind::Print => self.parse_print_statement(),
            TokenKind::Func => self.parse_function_declaration(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Struct => self.parse_struct_declaration(),
            TokenKind::Class => self.parse_class_declaration(),
            TokenKind::Try => self.parse_try_catch_statement(),
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Identifier if self.check_next(&TokenKind::Equal) => {
                self.parse_assignment_statement()
            }
            _ => self.parse_expression_statement(),
        }
    }

    /// Parses `name = expression`.
    fn parse_assignment_statement(&mut self) -> Result<Statement, CompilerError> {
        let name_token = self.advance();
        let name = name_token.lexeme;
        let line = name_token.line;
        let column = name_token.column;

        self.expect(TokenKind::Equal, "'='")?;
        let value = self.parse_expression()?;

        Ok(Statement::Assignment { name, value, line, column })
    }

    /// Parses `if condition { ... } ( else ( if ... | { ... } ) )?`.
    fn parse_if_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let condition = self.with_no_struct_literal(|p| p.parse_expression())?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                // `else if` — represented as a single nested if statement.
                Some(vec![self.parse_if_statement()?])
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

    /// Parses `while condition { ... }`.
    fn parse_while_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let condition = self.with_no_struct_literal(|p| p.parse_expression())?;
        let body = self.parse_block()?;

        Ok(Statement::WhileStatement { condition, body, line, column })
    }

    /// Parses `for name in start..end { ... }`.
    fn parse_for_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "loop variable name")?;
        let variable = name_token.lexeme;

        self.expect(TokenKind::In, "'in'")?;

        let start = self.with_no_struct_literal(|p| p.parse_addition())?;
        self.expect(TokenKind::DotDot, "'..'")?;
        let end = self.with_no_struct_literal(|p| p.parse_addition())?;

        let body = self.parse_block()?;

        Ok(Statement::ForStatement { variable, start, end, body, line, column })
    }

    /// Parses `break`.
    fn parse_break_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        Ok(Statement::BreakStatement { line: keyword.line, column: keyword.column })
    }

    /// Parses `continue`.
    fn parse_continue_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        Ok(Statement::ContinueStatement { line: keyword.line, column: keyword.column })
    }

    /// Parses `struct Name { field: type ... }`.
    ///
    /// Fields are newline-separated (no comma), matching the Bunzo
    /// example programs; a trailing field list ending right before `}`
    /// is also accepted without a separator requirement.
    fn parse_struct_declaration(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "struct name")?;
        let name = name_token.lexeme;

        self.expect(TokenKind::LeftBrace, "'{'")?;
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let field_name_token = self.expect(TokenKind::Identifier, "field name")?;
            self.expect(TokenKind::Colon, "':'")?;
            let type_token = self.expect(TokenKind::Identifier, "field type")?;

            fields.push(Parameter {
                name: field_name_token.lexeme,
                type_name: type_token.lexeme,
                line: field_name_token.line,
                column: field_name_token.column,
            });

            // Allow an optional comma between fields, in addition to the
            // newline-separated style shown in the language examples.
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(TokenKind::RightBrace, "'}'")?;

        Ok(Statement::StructDeclaration { name, fields, line, column })
    }

    /// Parses `class Name [extends Parent] [implements A, B] { ... }`.
    fn parse_class_declaration(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance(); // consume `class`
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "class name")?;
        let name = name_token.lexeme;

        // Optional `extends Parent`
        let extends = if self.check(&TokenKind::Extends) {
            self.advance();
            let parent = self.expect(TokenKind::Identifier, "parent class name")?;
            Some(parent.lexeme)
        } else {
            None
        };

        // Optional `implements A, B, ...`
        let mut implements = Vec::new();
        if self.check(&TokenKind::Implements) {
            self.advance();
            loop {
                let iface = self.expect(TokenKind::Identifier, "interface name")?;
                implements.push(iface.lexeme);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::LeftBrace, "'{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Func) {
                let method = self.parse_function_declaration()?;
                methods.push(method);
            } else {
                let field_name_token = self.expect(TokenKind::Identifier, "field name")?;
                self.expect(TokenKind::Colon, "':'")?;
                let type_token = self.expect(TokenKind::Identifier, "field type")?;

                fields.push(Parameter {
                    name: field_name_token.lexeme,
                    type_name: type_token.lexeme,
                    line: field_name_token.line,
                    column: field_name_token.column,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
        }

        self.expect(TokenKind::RightBrace, "'}'")?;

        Ok(Statement::ClassDeclaration {
            name,
            extends,
            implements,
            fields,
            methods,
            line,
            column,
        })
    }

    /// Parses `try { try_block } catch catch_var { catch_block }`.
    fn parse_try_catch_statement(&mut self) -> Result<Statement, CompilerError> {
        let try_kw = self.advance(); // consume `try`
        let try_block = self.parse_block()?;

        self.expect(TokenKind::Catch, "'catch'")?;
        let catch_var_token = self.expect(TokenKind::Identifier, "catch variable name")?;
        let catch_var = catch_var_token.lexeme;

        let catch_block = self.parse_block()?;

        Ok(Statement::TryCatch {
            try_block,
            catch_var,
            catch_block,
            line: try_kw.line,
            column: try_kw.column,
        })
    }

    /// Parses `throw expression`.
    fn parse_throw_statement(&mut self) -> Result<Statement, CompilerError> {
        let throw_kw = self.advance(); // consume `throw`
        let value = self.parse_expression()?;
        Ok(Statement::Throw {
            value,
            line: throw_kw.line,
            column: throw_kw.column,
        })
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

    /// Parses a bare expression as a statement.
    fn parse_expression_statement(&mut self) -> Result<Statement, CompilerError> {
        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Equal) {
            let eq = self.advance(); // consume '='
            let value = self.parse_expression()?;
            match expression {
                Expression::FieldAccess { object, field, .. } => {
                    Ok(Statement::FieldAssignment {
                        object: *object,
                        field,
                        value,
                        line: eq.line,
                        column: eq.column,
                    })
                }
                _ => Err(CompilerError::UnexpectedToken {
                    expected: "assignable field expression".to_string(),
                    found: describe_token(&eq),
                    line: eq.line,
                    column: eq.column,
                }),
            }
        } else {
            Ok(Statement::ExpressionStatement { expression })
        }
    }

    /// Parses `func name(param: type, ...) -> returnType { statements }`.
    ///
    /// The return type and parameter list are both optional in shape
    /// (zero parameters, no `->` clause), matching the Bunzo example
    /// programs where a function may declare no return value.
    fn parse_function_declaration(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let name_token = self.expect(TokenKind::Identifier, "function name")?;
        let name = name_token.lexeme;

        self.expect(TokenKind::LeftParen, "'('")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name_token = self.expect(TokenKind::Identifier, "parameter name")?;
                self.expect(TokenKind::Colon, "':'")?;
                let type_token = self.expect(TokenKind::Identifier, "parameter type")?;

                params.push(Parameter {
                    name: param_name_token.lexeme,
                    type_name: type_token.lexeme,
                    line: param_name_token.line,
                    column: param_name_token.column,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen, "')'")?;

        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            let type_token = self.expect(TokenKind::Identifier, "return type")?;
            Some(type_token.lexeme)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Statement::FunctionDeclaration {
            name,
            params,
            return_type,
            body,
            line,
            column,
        })
    }

    /// Parses `return` or `return expression`.
    ///
    /// A bare `return` (no value) is detected by checking whether the
    /// next token can begin an expression; if not, the function implicitly
    /// returns `null`.
    fn parse_return_statement(&mut self) -> Result<Statement, CompilerError> {
        let keyword = self.advance();
        let line = keyword.line;
        let column = keyword.column;

        let value = if self.can_start_expression() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::ReturnStatement { value, line, column })
    }

    /// Parses a `{ statement* }` block, used for function bodies.
    fn parse_block(&mut self) -> Result<Vec<Statement>, CompilerError> {
        self.expect(TokenKind::LeftBrace, "'{'")?;

        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace, "'}'")?;
        Ok(statements)
    }

    /// Returns `true` if the current token can begin an expression.
    ///
    /// Used to disambiguate a bare `return` from `return <expression>`,
    /// since Bunzo statements have no required terminator.
    fn can_start_expression(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::IntegerLiteral
                | TokenKind::FloatLiteral
                | TokenKind::StringLiteral
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Identifier
                | TokenKind::LeftParen
                | TokenKind::Bang
                | TokenKind::Minus
        )
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

    /// Parses `( "!" | "-" ) unary | call`.
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

        self.parse_call()
    }

    /// Parses `primary ( "(" arguments? ")" | "." IDENTIFIER )*` — zero or
    /// more call applications and/or field accesses chained onto a
    /// primary expression, e.g. `add(1, 2)` or `user.name`.
    fn parse_call(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&TokenKind::Dot) {
                let dot = self.advance();
                let field_token = self.expect(TokenKind::Identifier, "field name")?;
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field: field_token.lexeme,
                    line: dot.line,
                    column: dot.column,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parses the argument list and closing `)` of a call, given the
    /// already-parsed callee expression.
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, CompilerError> {
        let paren = self.advance(); // consume '('

        let mut arguments = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.with_struct_literal_allowed(|p| p.parse_expression())?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen, "')'")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            line: paren.line,
            column: paren.column,
        })
    }

    /// Parses the field list and closing `}` of a struct literal, given
    /// the already-consumed struct type name and its source location.
    ///
    /// `name { field: expr, field: expr }` — fields are comma-separated.
    fn parse_struct_literal(
        &mut self,
        name: String,
        line: usize,
        column: usize,
    ) -> Result<Expression, CompilerError> {
        self.expect(TokenKind::LeftBrace, "'{'")?;

        let mut fields = Vec::new();
        if !self.check(&TokenKind::RightBrace) {
            loop {
                let field_name_token = self.expect(TokenKind::Identifier, "field name")?;
                self.expect(TokenKind::Colon, "':'")?;
                let value = self.with_struct_literal_allowed(|p| p.parse_expression())?;

                fields.push((field_name_token.lexeme, value));

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightBrace, "'}'")?;

        Ok(Expression::StructLiteral { name, fields, line, column })
    }

    /// Parses a primary expression (literals, identifiers, grouping).
    fn parse_primary(&mut self) -> Result<Expression, CompilerError> {
        let token = self.peek().clone();

        match token.kind {
            TokenKind::IntegerLiteral => {
                self.advance();
                let value: i64 = token.lexeme.parse().map_err(|_| {
                    CompilerError::UnexpectedToken {
                        expected: "valid integer".to_string(),
                        found: describe_token(&token),
                        line: token.line,
                        column: token.column,
                    }
                })?;
                Ok(Expression::IntegerLiteral {
                    value,
                    line: token.line,
                    column: token.column,
                })
            }

            TokenKind::FloatLiteral => {
                self.advance();
                let value: f64 = token.lexeme.parse().map_err(|_| {
                    CompilerError::UnexpectedToken {
                        expected: "valid float".to_string(),
                        found: describe_token(&token),
                        line: token.line,
                        column: token.column,
                    }
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
                if !self.no_struct_literal && self.check(&TokenKind::LeftBrace) {
                    self.parse_struct_literal(token.lexeme, token.line, token.column)
                } else {
                    Ok(Expression::Identifier {
                        name: token.lexeme,
                        line: token.line,
                        column: token.column,
                    })
                }
            }

            TokenKind::LeftParen => {
                self.advance();
                let expr = self.with_struct_literal_allowed(|p| p.parse_expression())?;
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

    /// Returns `true` if the *next* token (one past current) matches `kind`.
    ///
    /// Used to disambiguate `name = expr` (assignment) from a bare
    /// expression statement starting with an identifier, without
    /// committing to a parse path.
    fn check_next(&self, kind: &TokenKind) -> bool {
        self.tokens
            .get(self.position + 1)
            .map(|t| t.kind == *kind)
            .unwrap_or(false)
    }

    /// Runs `f` with struct-literal parsing suppressed, restoring the
    /// previous setting afterward (even if `f` errors).
    ///
    /// Used for `if`/`while` conditions and `for` range bounds, where a
    /// bare `{` must open the statement block rather than a struct
    /// literal's field list.
    fn with_no_struct_literal<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, CompilerError>,
    ) -> Result<T, CompilerError> {
        let previous = self.no_struct_literal;
        self.no_struct_literal = true;
        let result = f(self);
        self.no_struct_literal = previous;
        result
    }

    /// Runs `f` with struct-literal parsing re-enabled, restoring the
    /// previous setting afterward (even if `f` errors).
    ///
    /// Used inside parentheses and call argument lists, which are
    /// unambiguous even within an `if`/`while` condition.
    fn with_struct_literal_allowed<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, CompilerError>,
    ) -> Result<T, CompilerError> {
        let previous = self.no_struct_literal;
        self.no_struct_literal = false;
        let result = f(self);
        self.no_struct_literal = previous;
        result
    }

    /// Consumes the current token if it matches `kind`, otherwise
    /// returns a [`CompilerError::UnexpectedToken`].
    fn expect(
        &mut self,
        kind: TokenKind,
        expected: &str,
    ) -> Result<Token, CompilerError> {
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
