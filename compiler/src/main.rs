//! Bunzo compiler entry point.
//!
//! This binary serves as the main driver for the Bunzo compiler (`bzc`).
//! It delegates immediately to [`cli::run`] and handles the process exit
//! code based on the result.

mod ast;
mod cli;
mod diagnostics;
mod ir;
mod lexer;
mod parser;
mod runtime;
mod semantic;
mod source;
mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Err(message) = cli::run(&args) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
