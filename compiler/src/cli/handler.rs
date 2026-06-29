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

/// Help screen layout with categorized commands.
const HELP_SCREEN: &str = "\
Bunzo Compiler

Build software, not boilerplate.

Usage:
    bzc <command> [options]

Project Commands
    run         Execute a Bunzo source file
    build       Compile a Bunzo project
    emit-c      Generate equivalent C code

Development Commands
    fmt         Format Bunzo source files
    lint        Analyze code for issues
    benchmark   Benchmark compilation and execution

Package Manager Commands
    install     Install Bunzo components
    update      Update Bunzo
    add         Add a package
    remove      Remove a package

Language Server Commands
    lsp         Launch the Language Server

Options
    --help      Show this help message
    --version   Show version information
    -V          Show version information

Repository
    https://github.com/Ugesh-Praavin/bunzo";

/// Minimal/fallback usage string for generic argument validation errors.
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

const HELP_USAGE: &str = "bzc --help";
const VERSION_USAGE: &str = "bzc --version";
const RUN_USAGE: &str = "bzc run <file.bz>";
const EMIT_C_USAGE: &str = "bzc emit-c <file.bz> [-o <output>]";
const BUILD_USAGE: &str = "bzc build <file.bz> [-o <output>]";
const BENCHMARK_USAGE: &str = "bzc benchmark <file.bz> [--repeat <N>] [--emit-c] [--no-run]";
const FMT_USAGE: &str = "bzc fmt <file_or_dir> [--check]";
const LINT_USAGE: &str = "bzc lint <file_or_dir>";
const INSTALL_USAGE: &str = "bzc install";
const UPDATE_USAGE: &str = "bzc update";
const ADD_USAGE: &str = "bzc add <package_name> <git_url>";
const REMOVE_USAGE: &str = "bzc remove <package_name>";
const LSP_USAGE: &str = "bzc lsp";

/// Represents a CLI error that maps to appropriate exit codes and formatted output.
#[derive(Debug)]
pub enum CliError {
    /// Exit Code 1: Compilation failed
    Compilation(String),
    /// Exit Code 1: Runtime error
    Runtime(String),
    /// Exit Code 1: Validation / linting / formatting failure
    Validation(String),
    /// Exit Code 2: Invalid CLI arguments
    InvalidArguments { message: String, usage: String },
    /// Exit Code 2: Unknown command
    UnknownCommand { command: String },
    /// Exit Code 2: Missing required parameters
    MissingParameters { message: String, usage: String },
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Compilation(msg) => write!(f, "{}", msg),
            CliError::Runtime(msg) => write!(f, "{}", msg),
            CliError::Validation(msg) => write!(f, "{}", msg),
            CliError::InvalidArguments { message, usage } => {
                write!(f, "error[E0003]: {}\n\nUsage:\n    {}", message, usage)
            }
            CliError::UnknownCommand { command } => {
                write!(
                    f,
                    "error[E0001]: unknown command '{}'\n\nRun 'bzc --help' for available commands.",
                    command
                )
            }
            CliError::MissingParameters { message, usage } => {
                write!(f, "error[E0002]: {}\n\nUsage:\n    {}", message, usage)
            }
        }
    }
}

impl std::error::Error for CliError {}

impl CliError {
    /// Return the exit code for the CLI invocation.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Compilation(_) | CliError::Runtime(_) | CliError::Validation(_) => 1,
            CliError::InvalidArguments { .. }
            | CliError::UnknownCommand { .. }
            | CliError::MissingParameters { .. } => 2,
        }
    }
}

/// Represents the parsed command to execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Run {
        file_path: String,
    },
    EmitC {
        file_path: String,
        output_path: Option<String>,
    },
    Build {
        file_path: String,
        output_path: Option<String>,
    },
    Benchmark {
        file_path: String,
        repeat: usize,
        emit_c: bool,
        no_run: bool,
    },
    Fmt {
        path: String,
        check: bool,
    },
    Lint {
        path: String,
    },
    Install,
    Update,
    Add {
        package_name: String,
        git_url: String,
    },
    Remove {
        package_name: String,
    },
    New {
        project_name: String,
    },
    Lsp,
}

/// Parse command-line arguments into a structured `Command` or returns `CliError`.
pub fn parse_args(args: &[String]) -> Result<Command, CliError> {
    if args.len() < 2 {
        return Err(CliError::MissingParameters {
            message: "missing command".to_string(),
            usage: USAGE.to_string(),
        });
    }

    let command = args[1].as_str();

    match command {
        "--help" | "-h" | "help" => {
            if args.len() > 2 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[2]),
                    usage: HELP_USAGE.to_string(),
                });
            }
            Ok(Command::Help)
        }
        "--version" | "-V" | "version" => {
            if args.len() > 2 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[2]),
                    usage: VERSION_USAGE.to_string(),
                });
            }
            Ok(Command::Version)
        }
        "run" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing source file".to_string(),
                    usage: RUN_USAGE.to_string(),
                });
            }
            let file_path = args[2].clone();
            if args.len() > 3 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[3]),
                    usage: RUN_USAGE.to_string(),
                });
            }
            Ok(Command::Run { file_path })
        }
        "emit-c" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing source file".to_string(),
                    usage: EMIT_C_USAGE.to_string(),
                });
            }
            let file_path = args[2].clone();
            let mut output_path = None;
            let mut i = 3;
            while i < args.len() {
                if args[i] == "-o" {
                    if i + 1 < args.len() {
                        output_path = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(CliError::InvalidArguments {
                            message: "missing value for option '-o'".to_string(),
                            usage: EMIT_C_USAGE.to_string(),
                        });
                    }
                } else {
                    return Err(CliError::InvalidArguments {
                        message: format!("unexpected argument '{}'", args[i]),
                        usage: EMIT_C_USAGE.to_string(),
                    });
                }
            }
            Ok(Command::EmitC {
                file_path,
                output_path,
            })
        }
        "build" => {
            let mut file_path = None;
            let mut output_path = None;

            if args.len() < 3 {
                if Path::new("bunzo.toml").exists() {
                    if let Ok(manifest) =
                        crate::packagemanager::manifest::Manifest::load_or_create("bunzo.toml")
                    {
                        let proj_bz = format!("{}.bz", manifest.name);
                        if Path::new(&proj_bz).exists() {
                            file_path = Some(proj_bz);
                        } else if Path::new("main.bz").exists() {
                            file_path = Some("main.bz".to_string());
                        }
                    }
                }

                if file_path.is_none() {
                    return Err(CliError::MissingParameters {
                        message: "missing source file".to_string(),
                        usage: BUILD_USAGE.to_string(),
                    });
                }
            } else {
                file_path = Some(args[2].clone());
            }

            let file_path = file_path.unwrap();
            let mut i = 3;
            if args.len() < 3 {
                i = 2;
            }
            while i < args.len() {
                if args[i] == "-o" {
                    if i + 1 < args.len() {
                        output_path = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(CliError::InvalidArguments {
                            message: "missing value for option '-o'".to_string(),
                            usage: BUILD_USAGE.to_string(),
                        });
                    }
                } else {
                    return Err(CliError::InvalidArguments {
                        message: format!("unexpected argument '{}'", args[i]),
                        usage: BUILD_USAGE.to_string(),
                    });
                }
            }
            Ok(Command::Build {
                file_path,
                output_path,
            })
        }
        "new" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing project name".to_string(),
                    usage: "bzc new <project_name>".to_string(),
                });
            }
            let project_name = args[2].clone();
            if args.len() > 3 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[3]),
                    usage: "bzc new <project_name>".to_string(),
                });
            }
            Ok(Command::New { project_name })
        }
        "benchmark" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing source file".to_string(),
                    usage: BENCHMARK_USAGE.to_string(),
                });
            }
            let file_path = args[2].clone();
            let mut repeat = 10;
            let mut emit_c = false;
            let mut no_run = false;
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--repeat" {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<usize>() {
                            Ok(r) => {
                                repeat = r;
                                i += 2;
                            }
                            Err(_) => {
                                return Err(CliError::InvalidArguments {
                                    message: format!("invalid repeat count '{}'", args[i + 1]),
                                    usage: BENCHMARK_USAGE.to_string(),
                                });
                            }
                        }
                    } else {
                        return Err(CliError::InvalidArguments {
                            message: "missing value for option '--repeat'".to_string(),
                            usage: BENCHMARK_USAGE.to_string(),
                        });
                    }
                } else if args[i] == "--emit-c" {
                    emit_c = true;
                    i += 1;
                } else if args[i] == "--no-run" {
                    no_run = true;
                    i += 1;
                } else {
                    return Err(CliError::InvalidArguments {
                        message: format!("unexpected argument '{}'", args[i]),
                        usage: BENCHMARK_USAGE.to_string(),
                    });
                }
            }
            Ok(Command::Benchmark {
                file_path,
                repeat,
                emit_c,
                no_run,
            })
        }
        "fmt" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing source file or directory".to_string(),
                    usage: FMT_USAGE.to_string(),
                });
            }
            let path = args[2].clone();
            let mut check = false;
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--check" {
                    check = true;
                    i += 1;
                } else {
                    return Err(CliError::InvalidArguments {
                        message: format!("unexpected argument '{}'", args[i]),
                        usage: FMT_USAGE.to_string(),
                    });
                }
            }
            Ok(Command::Fmt { path, check })
        }
        "lint" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing source file or directory".to_string(),
                    usage: LINT_USAGE.to_string(),
                });
            }
            let path = args[2].clone();
            if args.len() > 3 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[3]),
                    usage: LINT_USAGE.to_string(),
                });
            }
            Ok(Command::Lint { path })
        }
        "install" => {
            if args.len() > 2 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[2]),
                    usage: INSTALL_USAGE.to_string(),
                });
            }
            Ok(Command::Install)
        }
        "update" => {
            if args.len() > 2 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[2]),
                    usage: UPDATE_USAGE.to_string(),
                });
            }
            Ok(Command::Update)
        }
        "add" => {
            if args.len() < 4 {
                let msg = if args.len() == 2 {
                    "missing package name and git URL"
                } else {
                    "missing git URL"
                };
                return Err(CliError::MissingParameters {
                    message: msg.to_string(),
                    usage: ADD_USAGE.to_string(),
                });
            }
            let package_name = args[2].clone();
            let git_url = args[3].clone();
            if args.len() > 4 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[4]),
                    usage: ADD_USAGE.to_string(),
                });
            }
            Ok(Command::Add {
                package_name,
                git_url,
            })
        }
        "remove" => {
            if args.len() < 3 {
                return Err(CliError::MissingParameters {
                    message: "missing package name".to_string(),
                    usage: REMOVE_USAGE.to_string(),
                });
            }
            let package_name = args[2].clone();
            if args.len() > 3 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[3]),
                    usage: REMOVE_USAGE.to_string(),
                });
            }
            Ok(Command::Remove { package_name })
        }
        "lsp" => {
            if args.len() > 2 {
                return Err(CliError::InvalidArguments {
                    message: format!("unexpected argument '{}'", args[2]),
                    usage: LSP_USAGE.to_string(),
                });
            }
            Ok(Command::Lsp)
        }
        _ => Err(CliError::UnknownCommand {
            command: command.to_string(),
        }),
    }
}

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
/// Returns `Ok(())` on success, or a `CliError` on failure.
pub fn run(args: &[String]) -> Result<(), CliError> {
    if args.len() < 2 {
        let is_test = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .map(|name| name.contains("test") || name.contains("deps"))
            .unwrap_or(false);
        if is_test || !std::io::stdin().is_terminal() {
            return Err(CliError::MissingParameters {
                message: "missing command".to_string(),
                usage: USAGE.to_string(),
            });
        }
        return crate::repl::run().map_err(CliError::Runtime);
    }

    let cmd = parse_args(args)?;

    match cmd {
        Command::Help => {
            println!("{}", HELP_SCREEN);
            Ok(())
        }
        Command::Version => {
            println!(
                "Bunzo Compiler\n\
                 Version    : {}\n\
                 Target     : {}\n\
                 Language   : Bunzo\n\
                 Built With : Rust\n\
                 Repository : https://github.com/Ugesh-Praavin/bunzo\n\
                 Copyright (c) 2026 Ugesh Praavin",
                env!("CARGO_PKG_VERSION"),
                env!("TARGET")
            );
            Ok(())
        }
        Command::Run { file_path } => {
            let path = Path::new(&file_path);

            // Phase 1: Read source file.
            let source =
                source::read_source(path).map_err(|e| CliError::Compilation(e.to_string()))?;

            // Phase 2: Tokenize source.
            let tokens =
                lexer::tokenize(&source).map_err(|e| CliError::Compilation(format!("{e}")))?;

            // Phase 3: Parse tokens into an AST.
            let program =
                crate::parser::parse(tokens).map_err(|e| CliError::Compilation(format!("{e}")))?;

            // Phase 5: Semantic Analysis.
            crate::semantic::analyze(&program)
                .map_err(|e| CliError::Compilation(format!("{e}")))?;

            // Phase 10: Type Checking.
            crate::typechecker::check(&program)
                .map_err(|e| CliError::Compilation(format!("{e}")))?;

            // Phase 4: Interpret the AST.
            crate::runtime::execute(program).map_err(|e| CliError::Runtime(format!("{e}")))?;

            Ok(())
        }
        Command::EmitC {
            file_path,
            output_path,
        } => run_codegen(file_path, output_path, false),
        Command::Build {
            file_path,
            output_path,
        } => run_codegen(file_path, output_path, true),
        Command::Benchmark {
            file_path,
            repeat,
            emit_c,
            no_run,
        } => {
            crate::benchmark::run_benchmark(&file_path, repeat, emit_c, no_run)
                .map_err(CliError::Runtime)?;
            Ok(())
        }
        Command::Fmt { path, check } => match run_fmt(&path, check) {
            Ok(needs_formatting) => {
                if needs_formatting && check {
                    return Err(CliError::Validation("Formatting required".to_string()));
                }
                Ok(())
            }
            Err(e) => Err(CliError::Validation(e)),
        },
        Command::Lint { path } => {
            run_lint(&path).map_err(CliError::Validation)?;
            Ok(())
        }
        Command::Install => {
            crate::packagemanager::install().map_err(CliError::Runtime)?;
            Ok(())
        }
        Command::Update => {
            crate::packagemanager::update().map_err(CliError::Runtime)?;
            Ok(())
        }
        Command::Add {
            package_name,
            git_url,
        } => {
            crate::packagemanager::add(&package_name, &git_url).map_err(CliError::Runtime)?;
            Ok(())
        }
        Command::Remove { package_name } => {
            crate::packagemanager::remove(&package_name).map_err(CliError::Runtime)?;
            Ok(())
        }
        Command::New { project_name } => {
            run_new(&project_name)?;
            Ok(())
        }
        Command::Lsp => {
            crate::lsp::run().map_err(CliError::Runtime)?;
            Ok(())
        }
    }
}

fn run_codegen(
    file_path: String,
    output_path: Option<String>,
    is_build: bool,
) -> Result<(), CliError> {
    let path = Path::new(&file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");

    // Phase 1: Read source file.
    let source = source::read_source(path).map_err(|e| CliError::Compilation(e.to_string()))?;

    // Phase 2: Tokenize source.
    let tokens = lexer::tokenize(&source).map_err(|e| CliError::Compilation(format!("{e}")))?;

    // Phase 3: Parse tokens into an AST.
    let program =
        crate::parser::parse(tokens).map_err(|e| CliError::Compilation(format!("{e}")))?;

    // Phase 5: Semantic Analysis.
    crate::semantic::analyze(&program).map_err(|e| CliError::Compilation(format!("{e}")))?;

    // Phase 10: Type Checking.
    crate::typechecker::check(&program).map_err(|e| CliError::Compilation(format!("{e}")))?;

    // Lower to IR.
    let mut ir_module =
        crate::ir::lower(&program).map_err(|e| CliError::Compilation(format!("{e}")))?;

    // Optimize the IR.
    crate::ir::optimize(&mut ir_module);

    // Generate C code.
    let c_code =
        crate::codegen::generate(&ir_module).map_err(|e| CliError::Compilation(format!("{e}")))?;

    let c_file_path = if !is_build {
        output_path
            .clone()
            .unwrap_or_else(|| format!("{}.c", file_stem))
    } else {
        format!("{}.c", file_stem)
    };

    // Write C file.
    std::fs::write(&c_file_path, c_code).map_err(|e| {
        CliError::Runtime(format!("Error writing C output file {c_file_path}: {e}"))
    })?;

    if is_build {
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
            CliError::Runtime(
                "Error: Could not locate Bunzo runtime directory (runtime/runtime.c)".to_string(),
            )
        })?;
        let runtime_c_path = runtime_dir.join("runtime.c");
        let runtime_include = runtime_dir
            .to_str()
            .ok_or_else(|| CliError::Runtime("Invalid runtime path".to_string()))?;

        // Try compilers.
        let mut compilers = Vec::new();

        let mut bundled_clang_path = None;
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                if let Some(bunzo_dir) = exe_dir.parent() {
                    let bundled_bin = bunzo_dir.join("toolchain").join("bin").join("clang.exe");
                    if bundled_bin.exists() {
                        bundled_clang_path = Some(bundled_bin);
                    } else {
                        let bundled_bin_alt = bunzo_dir.join("toolchain").join("clang.exe");
                        if bundled_bin_alt.exists() {
                            bundled_clang_path = Some(bundled_bin_alt);
                        }
                    }
                }
            }
        }

        let mut bundled_clang_str = String::new();
        if let Some(path) = bundled_clang_path {
            if let Some(p_str) = path.to_str() {
                bundled_clang_str = p_str.to_string();
            }
        }

        if !bundled_clang_str.is_empty() {
            compilers.push((
                bundled_clang_str.as_str(),
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
            ));
        }

        compilers.push((
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
        ));

        compilers.push((
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
        ));

        compilers.push((
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
        ));

        let mut compiled = false;
        for (cc, cc_args) in &compilers {
            match std::process::Command::new(cc).args(cc_args).output() {
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

fn run_new(project_name: &str) -> Result<(), CliError> {
    let path = Path::new(project_name);
    if path.exists() {
        return Err(CliError::Runtime(format!(
            "Error: Directory '{}' already exists",
            project_name
        )));
    }

    std::fs::create_dir_all(path)
        .map_err(|e| CliError::Runtime(format!("Failed to create project directory: {}", e)))?;

    let toml_content = format!(
        "\
[package]
name = \"{}\"
version = \"0.1.0\"

[dependencies]
",
        project_name
    );
    std::fs::write(path.join("bunzo.toml"), toml_content)
        .map_err(|e| CliError::Runtime(format!("Failed to create bunzo.toml: {}", e)))?;

    let bz_content = "\
// Hello world project created by Bunzo
print(\"Hello, World!\")
";
    std::fs::write(path.join(format!("{}.bz", project_name)), bz_content)
        .map_err(|e| CliError::Runtime(format!("Failed to create source file: {}", e)))?;

    println!("Created new Bunzo project '{}'", project_name);
    Ok(())
}
