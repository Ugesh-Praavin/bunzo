# Contributing to Bunzo

Thank you for your interest in contributing to Bunzo! Every contribution — from fixing a typo to implementing a compiler feature — helps make this project better.

This guide will walk you through how to contribute effectively.

## Table of Contents

- [Project Philosophy](#project-philosophy)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Documentation-First Development](#documentation-first-development)
- [Coding Standards](#coding-standards)
- [Commit Message Convention](#commit-message-convention)
- [Pull Request Process](#pull-request-process)
- [Testing Requirements](#testing-requirements)
- [Review Expectations](#review-expectations)
- [Getting Help](#getting-help)

---

## Project Philosophy

Before contributing, please understand the principles that guide Bunzo:

1. **Solve real problems.** Every feature must address a genuine developer pain point.
2. **Simplicity over cleverness.** Readable, maintainable code is always preferred.
3. **Documentation first.** Design decisions should be documented before implementation begins.
4. **Quality over speed.** We would rather ship fewer, well-tested features than rush incomplete work.

Please read the [Bunzo Principles](context/bunzo_principles.md) for the full design philosophy.

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Git](https://git-scm.com/)

### Setup

1. **Fork** the repository on GitHub.

2. **Clone** your fork:

   ```bash
   git clone https://github.com/<your-username>/bunzo.git
   cd bunzo
   ```

3. **Verify** the project builds:

   ```bash
   cargo check
   cargo test
   ```

4. **Create a branch** for your work:

   ```bash
   git checkout -b feat/your-feature-name
   ```

---

## Development Workflow

1. **Check for existing issues.** Look at the [issue tracker](https://github.com/ugeshpraavin/bunzo/issues) before starting work.
2. **Open an issue first** if you're proposing a significant change. This allows discussion before implementation.
3. **Keep changes focused.** Each pull request should address a single concern.
4. **Write tests** for any new functionality.
5. **Update documentation** if your change affects public behavior.
6. **Ensure CI passes** before requesting review.

---

## Documentation-First Development

Bunzo follows a **documentation-first** approach:

- Before implementing a new feature, write or update the relevant design documentation.
- Document the *what* and *why* before the *how*.
- Architecture decisions should be recorded in the `rfcs/` directory for significant changes.
- Code comments should explain intent, not restate what the code does.

This practice ensures that contributors understand the reasoning behind decisions and that the project remains approachable for newcomers.

---

## Coding Standards

### Rust Style

- Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `cargo fmt` to format all code. The CI pipeline enforces this.
- Use `cargo clippy` and resolve all warnings. The CI pipeline treats warnings as errors.
- Prefer expressive variable and function names over abbreviations.
- Keep functions short and focused on a single responsibility.

### Code Organization

- Each compiler component lives in its own module under `compiler/src/`.
- Avoid circular dependencies between modules.
- Public APIs should have documentation comments (`///` doc comments).

### Error Handling

- Use `Result` types for operations that can fail.
- Provide meaningful error messages that help users understand what went wrong.
- Avoid `unwrap()` and `expect()` in library code; prefer proper error propagation.

---

## Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

### Types

| Type       | Description                                      |
| ---------- | ------------------------------------------------ |
| `feat`     | A new feature                                    |
| `fix`      | A bug fix                                        |
| `docs`     | Documentation changes only                       |
| `style`    | Code style changes (formatting, no logic change) |
| `refactor` | Code restructuring without behavior change       |
| `test`     | Adding or updating tests                         |
| `chore`    | Build process, CI, or tooling changes            |
| `perf`     | Performance improvements                         |
| `ci`       | Changes to CI configuration                      |

### Scope

Use the relevant compiler component or area:

- `lexer`, `parser`, `ast`, `semantic`, `ir`, `runtime`, `cli`, `diagnostics`
- `docs`, `ci`, `deps`, `repo`

### Examples

```
feat(lexer): add support for string escape sequences

fix(parser): handle trailing commas in function arguments

docs(readme): update build instructions for Windows

test(lexer): add edge case tests for numeric literals

chore(ci): add cargo clippy step to CI pipeline
```

---

## Pull Request Process

### Before Submitting

1. **Rebase** your branch on the latest `main`:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks locally:**

   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cargo check
   ```

3. **Write a clear PR description** explaining what your change does and why.

### Pull Request Checklist

- [ ] Code compiles without errors (`cargo check`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Commit messages follow Conventional Commits
- [ ] Documentation is updated if applicable
- [ ] New code includes tests where appropriate
- [ ] PR description clearly explains the change

---

## Testing Requirements

- **All new features** must include unit tests.
- **All bug fixes** should include a regression test.
- **Integration tests** belong in the `tests/` directory.
- **Unit tests** should be placed in the same file as the code they test, using Rust's `#[cfg(test)]` module convention.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p bzc

# Run a specific test
cargo test test_name
```

---

## Review Expectations

- **Be patient.** Maintainers review PRs as time permits. We aim to provide initial feedback within a few days.
- **Be open to feedback.** Reviews are collaborative. Suggested changes are meant to improve the project, not criticize your work.
- **Keep discussions constructive.** Disagree respectfully and provide reasoning.
- **Small PRs are reviewed faster.** Large PRs take longer to review and are more likely to require changes.

### What Reviewers Look For

- Correctness and test coverage
- Code clarity and readability
- Adherence to project conventions
- Documentation completeness
- No unrelated changes mixed in

---

## Getting Help

- **Questions?** Open a [Discussion](https://github.com/ugeshpraavin/bunzo/discussions).
- **Found a bug?** Open an [Issue](https://github.com/ugeshpraavin/bunzo/issues/new?template=bug_report.md).
- **Have an idea?** Open a [Feature Request](https://github.com/ugeshpraavin/bunzo/issues/new?template=feature_request.md).

---

Thank you for helping build Bunzo! 🎉
