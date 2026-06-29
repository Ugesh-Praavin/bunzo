//! Bunzo compiler entry point.
//!
//! This binary serves as the main driver for the Bunzo compiler (`bzc`).
//! It delegates immediately to [`cli::run`] and handles the process exit
//! code based on the result.

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Err(err) = bzc::cli::run(&args) {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}
