//! Token types and definitions for the Bunzo lexer.
//!
//! This module defines the [`TokenKind`] enum and [`Token`] struct that
//! represent the output of lexical analysis. All token types are derived
//! from the Bunzo language specification and architecture documentation.

use std::fmt;

/// The kind of a token produced by the lexer.
///
/// Token kinds are organized into categories matching the architecture
/// documentation: keywords, literals, identifiers, operators, delimiters,
/// punctuation, and special tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Current Keywords ──────────────────────────────────────────────
    /// `var` — mutable variable declaration (alias for `let`).
    Var,
    /// `let` — mutable variable declaration.
    Let,
    /// `const` — immutable variable declaration.
    Const,
    /// `if` — conditional branch.
    If,
    /// `else` — alternate conditional branch.
    Else,
    /// `while` — loop with condition.
    While,
    /// `for` — iteration loop.
    For,
    /// `in` — used in `for x in range` expressions.
    In,
    /// `break` — exit a loop.
    Break,
    /// `continue` — skip to next loop iteration.
    Continue,
    /// `return` — return a value from a function.
    Return,
    /// `func` — function declaration.
    Func,
    /// `true` — boolean literal.
    True,
    /// `false` — boolean literal.
    False,
    /// `null` — null literal.
    Null,
    /// `print` — built-in print function.
    Print,

    // ── OOP / Phase 4-6 Keywords ──────────────────────────────────────
    /// `class` — class declaration.
    Class,
    /// `extends` — class inheritance.
    Extends,
    /// `implements` — interface implementation.
    Implements,
    /// `interface` — interface declaration.
    Interface,
    /// `super` — parent class reference.
    Super,
    /// `self` — current instance reference (alias for `this`).
    SelfKw,
    /// `enum` — enum declaration.
    Enum,
    /// `struct` — struct declaration.
    Struct,
    /// `match` — pattern matching.
    Match,
    /// `switch` — reserved.
    Switch,
    /// `move` — ownership transfer.
    Move,

    // ── OOP access / abstraction ─────────────────────────────────────
    /// `abstract` — abstract class or method.
    Abstract,
    /// `public` — public field or method.
    Public,
    /// `private` — private field or method.
    Private,
    /// `trait` — alias for `interface`.
    Trait,

    // ── Concurrency / Phase 10 Keywords ──────────────────────────────
    /// `spawn` — spawn a concurrent task.
    Spawn,
    /// `async` — async function modifier.
    Async,
    /// `await` — await a future.
    Await,
    /// `channel` — create a channel.
    Channel,

    // ── Error Handling Keywords ───────────────────────────────────────
    /// `try` — try block.
    Try,
    /// `catch` — catch block.
    Catch,
    /// `throw` — throw an error.
    Throw,

    // ── Module Keywords ───────────────────────────────────────────────
    /// `import` — module import.
    Import,
    /// `export` — module export.
    Export,
    /// `from` — import source path.
    From,

    // ── Literals ──────────────────────────────────────────────────────
    /// An integer literal, e.g. `42`.
    IntegerLiteral,
    /// A floating-point literal, e.g. `3.14`.
    FloatLiteral,
    /// A string literal (contents without quotes), e.g. `"hello"`.
    StringLiteral,

    // ── Identifiers ───────────────────────────────────────────────────
    /// A user-defined identifier, e.g. `myVariable`.
    Identifier,

    // ── Arithmetic Operators ──────────────────────────────────────────
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,

    // ── Comparison Operators ──────────────────────────────────────────
    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `<`
    Less,
    /// `>`
    Greater,
    /// `<=`
    LessEqual,
    /// `>=`
    GreaterEqual,

    // ── Logical Operators ─────────────────────────────────────────────
    /// `&&`
    AmpersandAmpersand,
    /// `||`
    PipePipe,
    /// `!`
    Bang,

    // ── Assignment Operators ──────────────────────────────────────────
    /// `=`
    Equal,
    /// `+=`
    PlusEqual,
    /// `-=`
    MinusEqual,
    /// `*=`
    StarEqual,
    /// `/=`
    SlashEqual,

    // ── Delimiters ────────────────────────────────────────────────────
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,

    // ── Punctuation ───────────────────────────────────────────────────
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `..` — range operator.
    DotDot,
    /// `..=` — inclusive range operator.
    DotDotEqual,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `::`
    DoubleColon,
    /// `->` — function return type arrow.
    Arrow,
    /// `=>` — match arm fat arrow.
    FatArrow,
    /// `?` — error propagation.
    QuestionMark,
    /// `&` — borrow / reference.
    Ampersand,
    /// `|` — enum variant separator / pipe.
    Pipe,

    // ── Special ───────────────────────────────────────────────────────
    /// End of file.
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A source position in Bunzo source code.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The 1-based line number.
    pub line: usize,
    /// The 1-based column number.
    pub column: usize,
}

/// A single token produced by the lexer.
///
/// Each token carries its kind, the original source text (lexeme),
/// and the 1-based line and column where it begins.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of this token.
    pub kind: TokenKind,
    /// The original source text for this token.
    pub lexeme: String,
    /// The 1-based line number where the token starts.
    pub line: usize,
    /// The 1-based column number where the token starts.
    pub column: usize,
}

impl Token {
    /// Creates a new token with the given kind, lexeme, and position.
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}] {:?} {:?}",
            self.line, self.column, self.kind, self.lexeme,
        )
    }
}

/// Looks up a keyword by its string representation.
///
/// Returns `Some(TokenKind)` for recognized keywords (current and
/// reserved-future), or `None` for identifiers.
pub fn lookup_keyword(word: &str) -> Option<TokenKind> {
    match word {
        // Current keywords
        "var" => Some(TokenKind::Var),
        "let" => Some(TokenKind::Let),
        "const" => Some(TokenKind::Const),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "while" => Some(TokenKind::While),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "return" => Some(TokenKind::Return),
        "func" => Some(TokenKind::Func),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "null" => Some(TokenKind::Null),
        "print" => Some(TokenKind::Print),
        // OOP / type system
        "class" => Some(TokenKind::Class),
        "extends" => Some(TokenKind::Extends),
        "implements" => Some(TokenKind::Implements),
        "interface" => Some(TokenKind::Interface),
        "super" => Some(TokenKind::Super),
        "self" => Some(TokenKind::SelfKw),
        "enum" => Some(TokenKind::Enum),
        "struct" => Some(TokenKind::Struct),
        "match" => Some(TokenKind::Match),
        "switch" => Some(TokenKind::Switch),
        "move" => Some(TokenKind::Move),
        "abstract" => Some(TokenKind::Abstract),
        "public" => Some(TokenKind::Public),
        "private" => Some(TokenKind::Private),
        "trait" => Some(TokenKind::Trait),
        // Concurrency
        "spawn" => Some(TokenKind::Spawn),
        "async" => Some(TokenKind::Async),
        "await" => Some(TokenKind::Await),
        "channel" => Some(TokenKind::Channel),
        // Error handling
        "try" => Some(TokenKind::Try),
        "catch" => Some(TokenKind::Catch),
        "throw" => Some(TokenKind::Throw),
        // Modules
        "import" => Some(TokenKind::Import),
        "export" => Some(TokenKind::Export),
        "from" => Some(TokenKind::From),
        _ => None,
    }
}
