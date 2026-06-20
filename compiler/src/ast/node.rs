//! Abstract syntax tree node definitions for Bunzo.
//!
//! This module defines the data structures that represent the syntactic
//! structure of a Bunzo program after parsing. AST nodes are produced by
//! the parser and consumed by downstream stages (semantic analysis,
//! interpreter, code generation).
//!
//! Phase 3 scope: variables (`let`, `const`), `print` statements, and
//! expressions (arithmetic, comparison, logical, literals, identifiers).

// ── Program ───────────────────────────────────────────────────────────────

/// A complete Bunzo program — a sequence of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The top-level statements in source order.
    pub statements: Vec<Statement>,
}

// ── Statements ────────────────────────────────────────────────────────────

/// A single statement in a Bunzo program.
///
/// Declarations and `print` statements carry source location (line/column) for
/// downstream error reporting. For expression statements, use the inner
/// expression's location.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// `let name = initializer` — mutable variable declaration.
    LetDeclaration {
        /// The variable name.
        name: String,
        /// The initializer expression.
        initializer: Expression,
        /// Line where the `let` keyword appears.
        line: usize,
        /// Column where the `let` keyword appears.
        column: usize,
    },

    /// `const name = initializer` — immutable variable declaration.
    ConstDeclaration {
        /// The variable name.
        name: String,
        /// The initializer expression.
        initializer: Expression,
        /// Line where the `const` keyword appears.
        line: usize,
        /// Column where the `const` keyword appears.
        column: usize,
    },

    /// `print(argument)` — built-in print statement.
    PrintStatement {
        /// The expression to print.
        argument: Expression,
        /// Line where the `print` keyword appears.
        line: usize,
        /// Column where the `print` keyword appears.
        column: usize,
    },

    /// A bare expression used as a statement.
    ExpressionStatement {
        /// The expression.
        expression: Expression,
    },
}

// ── Expressions ───────────────────────────────────────────────────────────

/// An expression that evaluates to a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// An integer literal, e.g. `42`.
    IntegerLiteral {
        value: i64,
        line: usize,
        column: usize,
    },

    /// A floating-point literal, e.g. `3.14`.
    FloatLiteral {
        value: f64,
        line: usize,
        column: usize,
    },

    /// A string literal, e.g. `"hello"`.
    StringLiteral {
        value: String,
        line: usize,
        column: usize,
    },

    /// A boolean literal (`true` or `false`).
    BooleanLiteral {
        value: bool,
        line: usize,
        column: usize,
    },

    /// The `null` literal.
    NullLiteral {
        line: usize,
        column: usize,
    },

    /// A variable reference, e.g. `x`.
    Identifier {
        name: String,
        line: usize,
        column: usize,
    },

    /// A binary operation, e.g. `a + b`.
    BinaryOp {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
        /// Line of the operator token.
        line: usize,
        /// Column of the operator token.
        column: usize,
    },

    /// A unary operation, e.g. `-x` or `!flag`.
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
        /// Line of the operator token.
        line: usize,
        /// Column of the operator token.
        column: usize,
    },

    /// A parenthesized expression, e.g. `(a + b)`.
    Grouping {
        expression: Box<Expression>,
        /// Line of the opening `(`.
        line: usize,
        /// Column of the opening `(`.
        column: usize,
    },
}

// ── Operators ─────────────────────────────────────────────────────────────

/// A binary operator connecting two expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Logical
    And,
    Or,
}

/// A unary prefix operator applied to one expression.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// Arithmetic negation (`-`).
    Negate,
    /// Logical negation (`!`).
    LogicalNot,
}
