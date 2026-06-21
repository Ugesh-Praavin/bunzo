# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
