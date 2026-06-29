<p align="center">
  <!-- TODO: Replace with actual Bunzo logo -->
  <img src="docs/assets/logo-placeholder.png" alt="Bunzo Logo" width="200" />
</p>

<h1 align="center">Bunzo Programming Language</h1>

<p align="center">
  <strong>Build software. Not boilerplate.</strong>
</p>

<p align="center">
  <a href="#getting-started">Getting Started</a> •
  <a href="#features">Features</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#contributing">Contributing</a> •
  <a href="#license">License</a>
</p>

<p align="center">
  <a href="https://github.com/ugeshpraavin/bunzo/actions/workflows/ci.yml">
    <img src="https://github.com/ugeshpraavin/bunzo/actions/workflows/ci.yml/badge.svg" alt="CI Status" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" />
  </a>
</p>

---

## About

Bunzo is an open-source programming language designed to reduce software development complexity. It eliminates repetitive boilerplate, minimizes configuration, and provides an exceptional developer experience — so you can focus on solving real problems instead of fighting your tools.

Bunzo is written in [Rust](https://www.rust-lang.org/) for reliability and performance.

> **Note:** Bunzo is in early development. The compiler is under active construction and is not yet suitable for production use.

## Vision

Bunzo aims to become a complete software development platform — not just another programming language. The long-term goal includes:

- A modern, readable language with sensible defaults
- A fast compiler with excellent error messages
- An integrated standard library, package manager, and tooling ecosystem
- A welcoming, community-driven open-source project

## Features

Bunzo is being built with the following planned capabilities:

| Feature                  | Status       |
| ------------------------ | ------------ |
| Lexer                    | ✅ Complete   |
| Parser                   | ✅ Complete   |
| Abstract Syntax Tree     | ✅ Complete   |
| Interpreter              | ✅ Complete   |
| Semantic Analysis        | ✅ Complete   |
| Type Checker             | ✅ Complete   |
| IR Generation            | ✅ Complete   |
| IR Optimization          | ✅ Complete   |
| Standard Library         | ✅ Complete   |
| Package Manager          | ✅ Complete   |
| Formatter                | ✅ Complete   |
| Language Server (LSP)    | ✅ Complete   |
| VS Code Extension        | ✅ Complete   |
| Native Compiler Backend  | 🔮 Future    |
| LLVM Backend             | 🔮 Future    |
| WebAssembly Target       | 🔮 Future    |

## Repository Structure

```text
bunzo/
├── compiler/           # The Bunzo compiler (bzc)
│   └── src/
│       ├── ast/        # Abstract Syntax Tree definitions
│       ├── cli/        # Command-line interface
│       ├── diagnostics/# Error reporting and diagnostics
│       ├── ir/         # Intermediate representation
│       ├── lexer/      # Lexical analysis (tokenizer)
│       ├── parser/     # Syntax analysis (parser)
│       ├── runtime/    # Runtime support
│       ├── semantic/   # Semantic analysis
│       ├── source/     # Source code reader
│       ├── utils/      # Shared utilities
│       └── main.rs     # Compiler entry point
├── benchmarks/         # Performance benchmarks
├── book/               # The Bunzo language book
├── docs/               # Project documentation
├── examples/           # Example Bunzo programs
├── project_context/    # Internal project context
├── rfcs/               # Request for Comments
├── runtime/            # Standalone runtime components
├── scripts/            # Build and development scripts
├── stdlib/             # Standard library
├── tests/              # Integration and end-to-end tests
├── website/            # Project website source
├── Cargo.toml          # Rust workspace configuration
└── Cargo.lock          # Dependency lock file
```

## Compiler Architecture

The Bunzo compiler processes source code through a multi-stage pipeline:

```text
    Bunzo Source Code (.bz)
             │
             ▼
        Source Reader         ← Read raw source text
             │
             ▼
           Lexer              ← Tokenize into a stream of tokens
             │
             ▼
          Parser              ← Build an Abstract Syntax Tree
             │
             ▼
     Semantic Analyzer        ← Validate correctness and types
             │
             ▼
   Intermediate Representation
             │
    ┌────────┴────────┐
    ▼                 ▼
Interpreter      Compiler     ← Execute or compile to native code
    │                 │
    ▼                 ▼
 Output          Executable
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Git](https://git-scm.com/)

### Clone the Repository

```bash
git clone https://github.com/ugeshpraavin/bunzo.git
cd bunzo
```

### Building

Verify the project compiles successfully:

```bash
cargo check
```

Run the test suite:

```bash
cargo test
```

Build the compiler:

```bash
cargo build
```

### Running the Compiler

```bash
cargo run -p bzc
```

### Code Quality

Check formatting:

```bash
cargo fmt --check
```

Run the linter:

```bash
cargo clippy -- -D warnings
```

## Roadmap

### Phase 1 — Core Compiler
- [x] Project foundation and architecture
- [x] Lexer
- [x] Parser
- [x] AST
- [x] Interpreter

### Phase 2 — Language Features
- [x] Semantic analysis
- [x] Type checker
- [x] IR Generation
- [x] IR Optimization
- [x] Standard library
- [x] Module system

### Phase 3 — Developer Tooling
- [x] Package manager
- [x] Code formatter
- [x] REPL
- [x] Language Server (LSP)
- [x] VS Code Extension

### Phase 4 — Backend Development Features
- [x] Built-in HTTP server
- [x] Database connectivity
- [x] REST API support

### Phase 5 — Advanced
- [ ] Native compiler backend
- [ ] LLVM integration
- [ ] WebAssembly target
- [ ] AI-assisted development features

## Contributing

We welcome contributions from everyone! Whether you're fixing a typo, improving documentation, or implementing a new compiler feature, your help is appreciated.

Please read our [Contributing Guide](CONTRIBUTING.md) before submitting a pull request.

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Community

- 📖 [Documentation](docs/)
- 🛠️ [Release Engineering](docs/release_engineering.md)
- 🐛 [Issue Tracker](https://github.com/ugeshpraavin/bunzo/issues)
- 💬 [Discussions](https://github.com/ugeshpraavin/bunzo/discussions)

## License

Bunzo is licensed under the [MIT License](LICENSE).

```
Copyright (c) 2026 Ugesh Praavin D
```

---

<p align="center">
  Made with ❤️ by the Bunzo community
</p>
