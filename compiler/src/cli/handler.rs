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
    println!("Reading {file_path}...\n");

    let source = source::read_source(path).map_err(|e| format!("{e}"))?;

    // Phase 2: Tokenize source.
    let tokens = lexer::tokenize(&source).map_err(|e| format!("{e}"))?;

    // Print token stream for verification.
    for token in &tokens {
        println!(
            "[{}:{}]\t{:<20}\t{:?}",
            token.line, token.column, format!("{:?}", token.kind), token.lexeme,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build an args slice matching real CLI invocation.
    fn make_args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn missing_arguments_shows_usage() {
        let args = make_args(&["bzc"]);
        let result = run(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn too_many_arguments_shows_usage() {
        let args = make_args(&["bzc", "run", "a.bz", "extra"]);
        let result = run(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn unknown_command_shows_usage() {
        let args = make_args(&["bzc", "build", "hello.bz"]);
        let result = run(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn run_missing_file_returns_error() {
        let args = make_args(&["bzc", "run", "nonexistent_file.bz"]);
        let result = run(&args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BZ0001"));
    }

    #[test]
    fn run_valid_source_file() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("bzc_test_cli_run.bz");

        std::fs::write(&file_path, "let x = 42").expect("failed to write temp file");

        let args = make_args(&["bzc", "run", file_path.to_str().unwrap()]);
        let result = run(&args);

        assert!(result.is_ok(), "should succeed: {result:?}");

        // Clean up.
        let _ = std::fs::remove_file(&file_path);
    }
}
