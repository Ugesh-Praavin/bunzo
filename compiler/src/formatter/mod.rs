//! Formatting entry points for Bunzo.

pub mod style;
#[cfg(test)]
pub mod tests;
pub mod writer;

use crate::diagnostics::CompilerError;

/// Formats the given Bunzo source code using the default configuration.
pub fn format(source: &str) -> Result<String, CompilerError> {
    let tokens = crate::lexer::tokenize(source)?;
    let program = crate::parser::parse(tokens.clone())?;
    let comments = writer::extract_comments(source);
    let brace_pairs = writer::map_braces(&tokens);
    let config = style::Config::default();

    let formatter = writer::Formatter::new(&tokens, comments, brace_pairs, config);
    Ok(formatter.format(&program))
}
