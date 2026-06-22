//! Intermediate Representation (IR) for the Bunzo compiler.
//!
//! This module provides a platform-independent IR that sits between the
//! type-checked AST and any compilation backend.  It is the canonical
//! internal representation of a validated Bunzo program.
//!
//! # Architecture
//!
//! ```text
//! ast::Program  (validated + type-checked)
//!      │
//!      ▼  ir::lower()
//! ir::IrModule
//!      │
//!      ├──▶ Interpreter
//!      └──▶ Native / LLVM / WASM backend (future)
//! ```
//!
//! # Module organisation
//!
//! | Sub-module | Purpose |
//! |---|---|
//! | [`types`] | [`IrType`] — Bunzo-level type of an IR value |
//! | [`instructions`] | [`Instruction`], [`Operand`], [`VirtualRegister`], [`Constant`] |
//! | [`function`] | [`IrFunction`] and [`BasicBlock`] |
//! | [`module`] | [`IrModule`] — top-level container |
//! | [`builder`] | [`IrBuilder`] — stateful IR construction helper |
//! | [`lowering`] | [`lower()`] — AST → IR translation |
//! | [`pretty_print`] | [`print_module()`] — human-readable text output |
//!
//! # Usage
//!
//! ```rust,ignore
//! use bzc::ir;
//!
//! let ir_module = ir::lower(&typed_program)?;
//! let text = ir::print_module(&ir_module);
//! println!("{text}");
//! ```

pub mod builder;
pub mod function;
pub mod instructions;
pub mod lowering;
pub mod module;
pub mod pretty_print;
pub mod types;

// Re-export the primary public API.
pub use builder::IrBuilder;
pub use function::{BasicBlock, IrFunction, IrParameter};
pub use instructions::{BinOpKind, Constant, Instruction, Operand, UnaryOpKind, VirtualRegister};
pub use lowering::lower;
pub use module::IrModule;
pub use pretty_print::print_module;
pub use types::IrType;
