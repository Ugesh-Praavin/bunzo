//! Abstract syntax tree node definitions for Bunzo.
//!
//! This module defines the data structures that represent the syntactic
//! structure of a Bunzo program after parsing. AST nodes are produced by
//! the parser and consumed by downstream stages (semantic analysis,
//! interpreter, code generation).
//!
//! Phase 3 scope: variables (`let`, `const`), `print` statements, and
//! expressions (arithmetic, comparison, logical, literals, identifiers).
//! Phase 1 (functions) scope: function declarations, `return`, and calls.

// ── Program ───────────────────────────────────────────────────────────────

/// A complete Bunzo program — a sequence of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The top-level statements in source order.
    pub statements: Vec<Statement>,
}

// ── Function Parameters ──────────────────────────────────────────────────

/// Field/method visibility for classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Public
    }
}

/// A single parameter in a function signature, e.g. `name: type`.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter name.
    pub name: String,
    /// The parameter's declared type annotation (e.g. `int`, `string`).
    pub type_name: String,
    /// Optional visibility (`public` / `private`); defaults to public.
    pub visibility: Visibility,
    /// Line where the parameter name appears.
    pub line: usize,
    /// Column where the parameter name appears.
    pub column: usize,
}

/// A method signature inside an `interface` / `trait` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<String>,
    pub line: usize,
    pub column: usize,
}

// ── Match Arm ─────────────────────────────────────────────────────────────

/// One arm of a `match` statement: `pattern => { body }`.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// The pattern to match against. Can be a literal, identifier, or `_`.
    pub pattern: MatchPattern,
    /// The statements to execute when this arm matches.
    pub body: Vec<Statement>,
}

/// A pattern in a match arm.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    /// Wildcard — matches everything.
    Wildcard,
    /// An integer literal pattern.
    Integer(i64),
    /// A float literal pattern.
    Float(f64),
    /// A string literal pattern.
    StringLit(String),
    /// A boolean pattern.
    Boolean(bool),
    /// `null` pattern.
    Null,
    /// A named pattern (enum variant or binding).
    Identifier(String),
    /// An enum variant with payload, e.g. `Some(x)`.
    EnumVariant(String, Option<Box<MatchPattern>>),
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

    /// `func name(params) -> returnType { body }` — function declaration.
    FunctionDeclaration {
        /// The function name.
        name: String,
        /// The function's parameters, in declaration order.
        params: Vec<Parameter>,
        /// The declared return type name, if any.
        return_type: Option<String>,
        /// The statements making up the function body (empty for abstract methods).
        body: Vec<Statement>,
        /// Method/function visibility.
        visibility: Visibility,
        /// `true` for abstract methods (no body).
        is_abstract: bool,
        /// Line where the `func` keyword appears.
        line: usize,
        /// Column where the `func` keyword appears.
        column: usize,
    },

    /// `return` or `return expression` — returns from the enclosing function.
    ReturnStatement {
        /// The returned value, or `None` for a bare `return`.
        value: Option<Expression>,
        /// Line where the `return` keyword appears.
        line: usize,
        /// Column where the `return` keyword appears.
        column: usize,
    },

    /// `name = expression` — reassigns an existing variable.
    Assignment {
        /// The variable being assigned to.
        name: String,
        /// The new value expression.
        value: Expression,
        /// Line where the target identifier appears.
        line: usize,
        /// Column where the target identifier appears.
        column: usize,
    },

    /// `if condition { ... } else { ... }` — conditional branch.
    IfStatement {
        /// The branch condition.
        condition: Expression,
        /// Statements executed when `condition` is true.
        then_branch: Vec<Statement>,
        /// Statements executed when `condition` is false, if an `else`
        /// clause is present. `else if` is represented as a single
        /// nested `IfStatement` inside this vector.
        else_branch: Option<Vec<Statement>>,
        /// Line where the `if` keyword appears.
        line: usize,
        /// Column where the `if` keyword appears.
        column: usize,
    },

    /// `while condition { ... }` — conditional loop.
    WhileStatement {
        /// The loop condition, checked before each iteration.
        condition: Expression,
        /// The loop body.
        body: Vec<Statement>,
        /// Line where the `while` keyword appears.
        line: usize,
        /// Column where the `while` keyword appears.
        column: usize,
    },

    /// `for name in start..end { ... }` — range-based iteration loop.
    ForStatement {
        /// The loop variable name, bound fresh each iteration.
        variable: String,
        /// The inclusive-start bound of the range.
        start: Expression,
        /// The exclusive end bound of the range.
        end: Expression,
        /// The loop body.
        body: Vec<Statement>,
        /// Line where the `for` keyword appears.
        line: usize,
        /// Column where the `for` keyword appears.
        column: usize,
    },

    /// `break` — exits the nearest enclosing loop.
    BreakStatement {
        /// Line where the `break` keyword appears.
        line: usize,
        /// Column where the `break` keyword appears.
        column: usize,
    },

    /// `continue` — skips to the next iteration of the nearest enclosing loop.
    ContinueStatement {
        /// Line where the `continue` keyword appears.
        line: usize,
        /// Column where the `continue` keyword appears.
        column: usize,
    },

    /// `struct Name { field: type ... }` — struct type declaration.
    StructDeclaration {
        /// The struct's name.
        name: String,
        /// The struct's fields, in declaration order.
        fields: Vec<Parameter>,
        /// Line where the `struct` keyword appears.
        line: usize,
        /// Column where the `struct` keyword appears.
        column: usize,
    },

    /// `class Name { field: type ... func method() { ... } }` — class type declaration.
    ClassDeclaration {
        /// The class's name.
        name: String,
        /// Optional parent class name (`extends ParentClass`).
        extends: Option<String>,
        /// Implemented interface names (`implements A, B`).
        implements: Vec<String>,
        /// Whether this class cannot be instantiated directly.
        is_abstract: bool,
        /// The class's fields, in declaration order.
        fields: Vec<Parameter>,
        /// The class's methods.
        methods: Vec<Statement>,
        /// Line where the `class` keyword appears.
        line: usize,
        /// Column where the `class` keyword appears.
        column: usize,
    },

    /// `object.field = value` — field assignment statement.
    FieldAssignment {
        /// The object whose field is being assigned.
        object: Expression,
        /// The field name.
        field: String,
        /// The new value.
        value: Expression,
        /// Line where the target identifier or dot appears.
        line: usize,
        /// Column where the target identifier or dot appears.
        column: usize,
    },

    /// `try { try_block } catch catch_var { catch_block }`
    TryCatch {
        try_block: Vec<Statement>,
        catch_var: String,
        catch_block: Vec<Statement>,
        line: usize,
        column: usize,
    },

    /// `throw expression`
    Throw {
        value: Expression,
        line: usize,
        column: usize,
    },

    // ─────────────────── Phase 4+ additions ─────────────────────────
    /// `import moduleName` or `import name from "path"` — module import.
    ImportDeclaration {
        /// The local binding name (e.g. `json` in `import json`).
        name: String,
        /// Optional path string (e.g. `"./utils"` in `import utils from "./utils"`).
        path: Option<String>,
        line: usize,
        column: usize,
    },

    /// `export name` or `export func ...` — module export marker.
    ExportDeclaration {
        /// The name being exported.
        name: String,
        /// Optional inner declaration statement.
        declaration: Option<Box<Statement>>,
        line: usize,
        column: usize,
    },

    /// `enum Name { Variant1 Variant2(type) ... }` — enum type declaration.
    EnumDeclaration {
        name: String,
        /// Each variant: (variant_name, optional_payload_type)
        variants: Vec<(String, Option<String>)>,
        line: usize,
        column: usize,
    },

    /// `match expr { pattern => body ... }` — pattern matching.
    MatchStatement {
        subject: Expression,
        arms: Vec<MatchArm>,
        line: usize,
        column: usize,
    },

    /// `interface Name { func method(params) -> type ... }`
    InterfaceDeclaration {
        name: String,
        methods: Vec<MethodSignature>,
        line: usize,
        column: usize,
    },

    /// `spawn expr` — fire-and-forget concurrent execution.
    SpawnStatement {
        expression: Expression,
        line: usize,
        column: usize,
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
    NullLiteral { line: usize, column: usize },

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

    /// A function call, e.g. `add(1, 2)`.
    Call {
        /// The expression evaluating to the function being called.
        callee: Box<Expression>,
        /// The argument expressions, in call order.
        arguments: Vec<Expression>,
        /// Line of the opening `(`.
        line: usize,
        /// Column of the opening `(`.
        column: usize,
    },

    /// A struct literal, e.g. `User { id: 1, name: "Bart" }`.
    StructLiteral {
        /// The name of the struct type being constructed.
        name: String,
        /// The field initializers, in source order.
        fields: Vec<(String, Expression)>,
        /// Line where the struct type name appears.
        line: usize,
        /// Column where the struct type name appears.
        column: usize,
    },

    /// A field access, e.g. `user.name`.
    FieldAccess {
        /// The expression evaluating to the struct instance.
        object: Box<Expression>,
        /// The field being accessed.
        field: String,
        /// Line of the `.` token.
        line: usize,
        /// Column of the `.` token.
        column: usize,
    },

    // ─────────────────── Phase 4+ additions ─────────────────────────
    /// An array literal, e.g. `[1, 2, 3]`.
    ArrayLiteral {
        elements: Vec<Expression>,
        line: usize,
        column: usize,
    },

    /// An index expression, e.g. `arr[0]` or `map["key"]`.
    IndexExpression {
        object: Box<Expression>,
        index: Box<Expression>,
        line: usize,
        column: usize,
    },

    /// An enum variant construction, e.g. `Color::Red` or `Option::Some(x)`.
    EnumVariantExpr {
        enum_name: String,
        variant: String,
        payload: Option<Box<Expression>>,
        line: usize,
        column: usize,
    },

    /// Error propagation `expr?` — re-throws on error, unwraps on ok.
    PropagateError {
        expression: Box<Expression>,
        line: usize,
        column: usize,
    },

    /// `move x` — transfer ownership.
    MoveExpr {
        name: String,
        line: usize,
        column: usize,
    },

    /// `await expr` — await a future/channel.
    AwaitExpr {
        expression: Box<Expression>,
        line: usize,
        column: usize,
    },

    /// `super` — reference to the parent class (inside methods only).
    SuperExpr { line: usize, column: usize },
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
