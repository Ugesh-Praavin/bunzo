//! Command-line interface for the Bunzo compiler.
//!
//! This module handles argument parsing, subcommand dispatch, and
//! user-facing output for the `bzc` binary. All file-reading logic
//! is delegated to the [`crate::source`] module, and tokenization
//! is delegated to the [`crate::lexer`] module.

use std::path::Path;

use crate::lexer;
use crate::source;

/// Usage message printed when the user provides invalid arguments.
const USAGE: &str = "\
Usage:
    bzc run <file.bz>";

/// Runs the compiler CLI with the given command-line arguments.
///
/// `args` should be the full argument list including the program name
/// (i.e. the result of `std::env::args().collect()`).
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error message string on failure.
/// The caller is responsible for writing errors to stderr and setting
/// the exit code.
pub fn run(args: &[String]) -> Result<(), String> {
    // args[0] = program name, args[1] = subcommand, args[2] = file
    if args.len() != 3 {
        return Err(USAGE.to_string());
    }

    let command = &args[1];
    let file_path = &args[2];

    if command != "run" {
        return Err(USAGE.to_string());
    }

    let path = Path::new(file_path);

    // Phase 1: Read source file.
    eprintln!("Reading {file_path}...\n");

    let source = source::read_source(path).map_err(|e| format!("{e}"))?;

    // Phase 2: Tokenize source.
    let tokens = lexer::tokenize(&source).map_err(|e| format!("{e}"))?;

    // Phase 3: Parse tokens into an AST.
    let program = crate::parser::parse(tokens).map_err(|e| format!("{e}"))?;

    // Phase 5: Semantic Analysis.
    crate::semantic::analyze(&program).map_err(|e| format!("{e}"))?;

    // Phase 10: Type Checking.
    crate::typechecker::check(&program).map_err(|e| format!("{e}"))?;

    // Phase 4: Interpret the AST.
    crate::runtime::execute(program).map_err(|e| format!("{e}"))?;

    Ok(())
}
