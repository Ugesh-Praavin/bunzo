# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.8.0-alpha] - 2026-06-29

### Added
* Refactored command dispatch and argument parsing logic.
* Version information commands (`bzc --version`, `bzc -V`) showing target build triple, compiler language, and repository info.
* Categorized, aligned, and polished help screen (`bzc --help`).
* Exit status codes conforming to standard conventions (0 for success, 1 for compilation/runtime/validation errors, 2 for invalid arguments/unknown commands/missing parameters).
* Standardized CLI diagnostics using consistent compiler error codes (`E0001`, `E0002`, `E0003`).
* Built executable renamed to `bzc` (and `bzc.exe`) to prevent double-binary confusion.
* Automated Rust-only release packaging tool (`cargo xtask`).
* Portable release distribution staging for multiple platforms (`.zip` and `.tar.gz`).
* Installer staging preparation under a dedicated directory structure.
* Standardized release validation script verifying version consistency, checksum integrity, and archive contents.
* Unified manifest (`manifest.json`) and metadata (`release-metadata.json`) schema.
* Extracted release notes automatically generated from CHANGELOG.md.

## [v0.7.0] - 2026-06-23

### Added
* Intermediate Representation (IR) Generation and IR Optimization passes.
* C Code Generator backend compiling optimized IR to C99.
* Zero-dependency benchmarking framework (`bzc benchmark`).
* Official source code formatter (`bzc fmt`) with comment preservation.
* Linter subsystem (`bzc lint`) validating syntax style and quality.
* Lightweight package manager (`bzc install`, `bzc add`, etc.) for managing dependencies in `bunzo.toml`.
* Interactive REPL shell.
* Language Server Protocol (LSP) server (`bzc lsp`) and companion VS Code Extension.

## [v0.6.0] - 2026-06-21

### Added
* Support for User-defined Modules (multi-file projects)
* Inline export declarations (`export func`, `export let`, `export class`, etc.)
* Static compile-time recursive module analysis and type checking
* Validation of exported module member accesses at compile-time
* Cycle dependency checking for circular imports

## [v0.5.0] - 2026-06-21

### Added
* Functions with parameters and typed return values
* Object-Oriented Programming (Classes, Inheritance, Interfaces)
* Complete Static Type Checker validating types before execution
* Error Handling (`try`/`catch`, `throw`)
* Standard library modules (`math`, `json`, `http`, `db`, `os`)

## [v0.4.0] - 2026-06-21

### Added
* `while` loops (repeated execution based on condition)
* `for in` loops with `start..end` range evaluation
* `break` and `continue` control flow operations inside loop blocks
* `AssignStatement` support (`var = value` variable reassignments)
* Validation and runtime errors for range bounds and condition types

## [v0.3.0] - 2026-06-21

### Added
* `if` and `else` statements for conditional execution
* `else if` chaining
* Block scoping in runtime environments and static analyzer

## [v0.2.0] - 2026-06-21

### Added
* Static Semantic Analyzer framework
* Scope resolution for variables and constants
* Detection of undefined variables and duplicate variable declarations in the same scope

## v0.1.0-alpha - 2026-06-20

This is the first public alpha release of the Bunzo programming language.

### Added
* Cargo workspace
* CLI
* Source Reader
* Lexer
* Parser
* AST
* Interpreter
* Runtime
* Variable declarations
* Constants
* Print statements
* Expression evaluation
* Documentation
* Complete test suite

### Known Limitations
The following features are not yet implemented:
* Type Checker
* Functions
* Standard Library
