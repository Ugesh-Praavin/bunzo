//! Parsing of token streams into an abstract syntax tree.
//!
//! This module implements a recursive descent parser that converts
//! the token stream produced by the lexer into a structured AST.

pub mod parser;
#[cfg(test)]
pub mod tests;

pub use parser::parse;

