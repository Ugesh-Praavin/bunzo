//! Parsing of token streams into an abstract syntax tree.
//!
//! This module implements a recursive descent parser that converts
//! the token stream produced by the lexer into a structured AST.

#![allow(clippy::module_inception)]

pub mod parser;

pub use parser::parse;
