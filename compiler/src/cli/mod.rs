//! Command-line interface for the Bunzo compiler.
//!
//! This module handles argument parsing, subcommand dispatch, and
//! user-facing output for the `bzc` binary. All file-reading logic
//! is delegated to the [`crate::source`] module.

use std::path::Path;

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

    println!("Reading {file_path}...\n");

    match source::read_source(path) {
        Ok(contents) => {
            println!("{contents}");
            Ok(())
        }
        Err(err) => Err(format!("{err}")),
    }
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
}
