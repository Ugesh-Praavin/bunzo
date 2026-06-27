//! Command-line interface for the Bunzo compiler.
//!
//! This module handles argument parsing, subcommand dispatch, and
//! user-facing output for the `bzc` binary. All file-reading logic
//! is delegated to the [`crate::source`] module, and tokenization
//! is delegated to the [`crate::lexer`] module.

use std::io::IsTerminal;
use std::path::Path;

use crate::lexer;
use crate::source;

/// Usage message printed when the user provides invalid arguments.
const USAGE: &str = "\
Usage:
    bzc run <file.bz>
    bzc emit-c <file.bz> [-o <output>]
    bzc build <file.bz> [-o <output>]
    bzc benchmark <file.bz> [--repeat <N>] [--emit-c] [--no-run]
    bzc fmt <file_or_dir> [--check]
    bzc lint <file_or_dir>
    bzc install
    bzc update
    bzc add <package_name> <git_url>
    bzc remove <package_name>
    bzc lsp";

/// Find the runtime directory containing runtime.c/runtime.h
fn find_runtime_dir() -> Option<std::path::PathBuf> {
    // 1. Try relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join("runtime");
        if path.join("runtime.c").exists() {
            return Some(path);
        }
    }

    // 2. Try relative to executable path
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent();
        while let Some(d) = dir {
            let path = d.join("runtime");
            if path.join("runtime.c").exists() {
                return Some(path);
            }
            // Sibling check
            if let Some(p) = d.parent() {
                let path = p.join("runtime");
                if path.join("runtime.c").exists() {
                    return Some(path);
                }
            }
            dir = d.parent();
        }
    }
    None
}

/// Runs the compiler CLI with the given command-line arguments.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error message string on failure.
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        let is_test = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .map(|name| name.contains("test") || name.contains("deps"))
            .unwrap_or(false);
        if is_test || !std::io::stdin().is_terminal() {
            return Err(USAGE.to_string());
        }
        return crate::repl::run();
    }

    let command = args[1].as_str();

    if command == "lsp" {
        if args.len() != 2 {
            return Err(USAGE.to_string());
        }
        return crate::lsp::run();
    }

    if command == "install" {
        if args.len() != 2 {
            return Err(USAGE.to_string());
        }
        return crate::packagemanager::install();
    }

    if command == "update" {
        if args.len() != 2 {
            return Err(USAGE.to_string());
        }
        return crate::packagemanager::update();
    }

    if command == "add" {
        if args.len() != 4 {
            return Err(USAGE.to_string());
        }
        return crate::packagemanager::add(&args[2], &args[3]);
    }

    if command == "remove" {
        if args.len() != 3 {
            return Err(USAGE.to_string());
        }
        return crate::packagemanager::remove(&args[2]);
    }

    if args.len() < 3 {
        return Err(USAGE.to_string());
    }

    let file_path = &args[2];

    if command != "run" && command != "emit-c" && command != "build" && command != "benchmark" && command != "fmt" && command != "lint" {
        return Err(USAGE.to_string());
    }

    let mut output_path = None;
    let mut repeat = 10;
    let mut emit_c = false;
    let mut no_run = false;

    if command == "benchmark" {
        let mut i = 3;
        while i < args.len() {
            if args[i] == "--repeat" && i + 1 < args.len() {
                repeat = args[i + 1]
                    .parse::<usize>()
                    .map_err(|_| "Error: Invalid repeat count".to_string())?;
                i += 2;
            } else if args[i] == "--emit-c" {
                emit_c = true;
                i += 1;
            } else if args[i] == "--no-run" {
                no_run = true;
                i += 1;
            } else {
                return Err(USAGE.to_string());
            }
        }
    } else if command == "fmt" {
        // fmt subcommand has its own option parsing
    } else {
        let mut i = 3;
        while i < args.len() {
            if args[i] == "-o" && i + 1 < args.len() {
                output_path = Some(args[i + 1].clone());
                i += 2;
            } else {
                return Err(USAGE.to_string());
            }
        }
    }

    if command == "run" && output_path.is_some() {
        return Err("Error: -o option is not supported for 'run' command".to_string());
    }

    if command == "fmt" {
        let mut check = false;
        let mut i = 3;
        while i < args.len() {
            if args[i] == "--check" {
                check = true;
                i += 1;
            } else {
                return Err(USAGE.to_string());
            }
        }
        match run_fmt(file_path, check) {
            Ok(needs_formatting) => {
                if needs_formatting && check {
                    return Err("Formatting required".to_string());
                } else {
                    return Ok(());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    if command == "lint" {
        if args.len() > 3 {
            return Err(USAGE.to_string());
        }
        return run_lint(file_path);
    }

    if command == "benchmark" {
        crate::benchmark::run_benchmark(file_path, repeat, emit_c, no_run)?;
        return Ok(());
    }

    let path = Path::new(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");

    // Phase 1: Read source file.
    let source = source::read_source(path).map_err(|e| format!("{e}"))?;

    // Phase 2: Tokenize source.
    let tokens = lexer::tokenize(&source).map_err(|e| format!("{e}"))?;

    // Phase 3: Parse tokens into an AST.
    let program = crate::parser::parse(tokens).map_err(|e| format!("{e}"))?;

    // Phase 5: Semantic Analysis.
    crate::semantic::analyze(&program).map_err(|e| format!("{e}"))?;

    // Phase 10: Type Checking.
    crate::typechecker::check(&program).map_err(|e| format!("{e}"))?;

    if command == "run" {
        // Phase 4: Interpret the AST.
        crate::runtime::execute(program).map_err(|e| format!("{e}"))?;
    } else {
        // Lower to IR.
        let mut ir_module = crate::ir::lower(&program).map_err(|e| format!("{e}"))?;

        // Optimize the IR.
        crate::ir::optimize(&mut ir_module);

        // Generate C code.
        let c_code = crate::codegen::generate(&ir_module).map_err(|e| format!("{e}"))?;

        let c_file_path = if command == "emit-c" {
            output_path
                .clone()
                .unwrap_or_else(|| format!("{}.c", file_stem))
        } else {
            format!("{}.c", file_stem)
        };

        // Write C file.
        std::fs::write(&c_file_path, c_code)
            .map_err(|e| format!("Error writing C output file {c_file_path}: {e}"))?;

        if command == "build" {
            let exe_path = output_path.clone().unwrap_or_else(|| {
                #[cfg(target_os = "windows")]
                {
                    format!("{}.exe", file_stem)
                }
                #[cfg(not(target_os = "windows"))]
                {
                    file_stem.to_string()
                }
            });

            // Find runtime directory.
            let runtime_dir = find_runtime_dir().ok_or_else(|| {
                "Error: Could not locate Bunzo runtime directory (runtime/runtime.c)".to_string()
            })?;
            let runtime_c_path = runtime_dir.join("runtime.c");
            let runtime_include = runtime_dir.to_str().ok_or("Invalid runtime path")?;

            // Try compilers.
            let compilers = [
                (
                    "clang",
                    vec![
                        "-O2",
                        "-o",
                        &exe_path,
                        &c_file_path,
                        runtime_c_path.to_str().unwrap(),
                        "-I",
                        runtime_include,
                        "-lm",
                    ],
                ),
                (
                    "gcc",
                    vec![
                        "-O2",
                        "-o",
                        &exe_path,
                        &c_file_path,
                        runtime_c_path.to_str().unwrap(),
                        "-I",
                        runtime_include,
                        "-lm",
                    ],
                ),
                (
                    "cc",
                    vec![
                        "-O2",
                        "-o",
                        &exe_path,
                        &c_file_path,
                        runtime_c_path.to_str().unwrap(),
                        "-I",
                        runtime_include,
                        "-lm",
                    ],
                ),
            ];

            let mut compiled = false;
            for (cc, args) in &compilers {
                match std::process::Command::new(cc).args(args).output() {
                    Ok(output) => {
                        eprintln!("Trying {}", cc);

                        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));

                        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

                        if output.status.success() {
                            compiled = true;
                            break;
                        }
                    }

                    Err(err) => {
                        eprintln!("Failed to launch {}: {}", cc, err);
                    }
                }
            }

            if compiled {
                eprintln!("Successfully compiled to {}", exe_path);
                // Clean up the temporary C file.
                let _ = std::fs::remove_file(&c_file_path);
            } else {
                eprintln!(
                    "Warning: No C compiler (clang/gcc/cc) succeeded. C source code written to {}",
                    c_file_path
                );
            }
        } else {
            eprintln!("C source code written to {}", c_file_path);
        }
    }

    Ok(())
}

fn run_fmt(path_str: &str, check: bool) -> Result<bool, String> {
    let path = Path::new(path_str);
    let mut files = Vec::new();
    collect_bz_files(path, &mut files).map_err(|e| format!("Error collecting files: {}", e))?;

    if files.is_empty() {
        return Err(format!("No .bz files found at path: {}", path_str));
    }

    let mut any_needs_formatting = false;
    let mut any_failed = false;

    for file in &files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                any_failed = true;
                continue;
            }
        };

        let formatted = match crate::formatter::format(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error parsing/formatting {}: {}", file.display(), e);
                any_failed = true;
                continue;
            }
        };

        if formatted != content {
            any_needs_formatting = true;
            if check {
                println!("File needs formatting: {}", file.display());
            } else {
                if let Err(e) = std::fs::write(file, &formatted) {
                    eprintln!("Error writing {}: {}", file.display(), e);
                    any_failed = true;
                } else {
                    println!("Formatted: {}", file.display());
                }
            }
        }
    }

    if any_failed {
        return Err("Some files failed to compile/format".to_string());
    }

    Ok(any_needs_formatting)
}

fn collect_bz_files(path: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        if path.extension().map_or(false, |ext| ext == "bz") {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" {
                    continue;
                }
            }
            collect_bz_files(&entry_path, files)?;
        }
    }
    Ok(())
}

fn run_lint(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    let mut files = Vec::new();
    collect_bz_files(path, &mut files).map_err(|e| format!("Error collecting files: {}", e))?;

    if files.is_empty() {
        return Err(format!("No .bz files found at path: {}", path_str));
    }

    let mut has_warnings = false;

    for file in &files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Error reading {}: {}", file.display(), e))?;

        let tokens = lexer::tokenize(&content)
            .map_err(|e| format!("Error tokenizing {}: {}", file.display(), e))?;

        let program = crate::parser::parse(tokens)
            .map_err(|e| format!("Error parsing {}: {}", file.display(), e))?;

        let warnings = crate::linter::lint(&program);
        if !warnings.is_empty() {
            has_warnings = true;
            for w in warnings {
                println!(
                    "[{}] Warning in {}:{}:{}: {}",
                    w.code,
                    file.display(),
                    w.line,
                    w.column,
                    w.message
                );
            }
        }
    }

    if has_warnings {
        Err("Lint checks failed with warnings".to_string())
    } else {
        println!("All checks passed!");
        Ok(())
    }
}
