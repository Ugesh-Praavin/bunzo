//! Runtime support for Bunzo program execution.
//!
//! This module will provide the runtime environment, including variable
//! storage, scope management, and built-in function execution.

pub mod db_state;
pub mod environment;
pub mod eval;
pub mod value;

pub use eval::execute;
