//! Runtime support for Bunzo program execution.
//!
//! This module will provide the runtime environment, including variable
//! storage, scope management, and built-in function execution.

pub mod environment;
pub mod eval;
pub mod value;

#[cfg(test)]
pub mod tests;

pub use eval::execute;

