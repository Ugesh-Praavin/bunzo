//! Bunzo compiler entry point.
//!
//! This binary serves as the main driver for the Bunzo compiler (`bzc`).
//! It delegates immediately to [`cli::run`] and handles the process exit
//! code based on the result.

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    // Check executable name for BusyBox-style multi-call routing
    if let Ok(exe_path) = std::env::current_exe() {
        if exe_path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|name| *name == "bzfmt")
            .is_some()
        {
            // If run as `bzfmt <args>`, route to `bzc fmt <args>`
            args.insert(1, "fmt".to_string());
        }
    }

    if let Err(err) = bzc::cli::run(&args) {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}
