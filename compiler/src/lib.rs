//! Bunzo compiler library.
//!
//! This crate provides the library interface for the Bunzo compiler (`bzc`).
//! The binary entry point is in [`main.rs`], which delegates to [`cli::run`].

pub mod ast;
pub mod benchmark;
pub mod cli;
pub mod codegen;
pub mod diagnostics;
pub mod formatter;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod semantic;
pub mod source;
pub mod stdlib;
pub mod typechecker;
pub mod utils;
