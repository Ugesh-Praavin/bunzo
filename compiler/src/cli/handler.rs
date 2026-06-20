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
