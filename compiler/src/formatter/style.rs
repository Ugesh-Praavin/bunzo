//! Formatting style configuration for Bunzo.

#[derive(Debug, Clone)]
pub struct Config {
    /// Number of spaces per indentation level.
    pub indent: usize,
    /// Line ending style, e.g. "\n" (LF).
    pub newline_style: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indent: 4,
            newline_style: "\n",
        }
    }
}
