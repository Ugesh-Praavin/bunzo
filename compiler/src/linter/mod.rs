pub mod rules;
#[cfg(test)]
pub mod tests;

pub use rules::{LintWarning, Linter};

/// Run the linter on a Program AST and return the warning diagnostics.
pub fn lint(program: &crate::ast::node::Program) -> Vec<LintWarning> {
    let linter = Linter::new();
    linter.lint(program)
}
