//! Semantic analysis for Bunzo programs.
//!
//! This module validates program correctness after parsing, including
//! scope resolution and detection of undefined variables or duplicate declarations.

pub mod analyzer;

#[cfg(test)]
pub mod tests;

pub use analyzer::analyze;

