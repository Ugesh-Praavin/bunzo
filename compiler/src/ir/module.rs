//! Top-level IR module container.
//!
//! An [`IrModule`] is the complete IR representation of one Bunzo source
//! file.  It is the single artefact produced by the lowering pass and
//! the single input consumed by any backend (interpreter, native compiler,
//! LLVM, WASM, …).
//!
//! Structure:
//! ```text
//! IrModule
//!   └── functions: Vec<IrFunction>
//!         └── blocks: Vec<BasicBlock>
//!               └── instructions: Vec<Instruction>
//! ```

use super::function::IrFunction;

/// The IR representation of a Bunzo source module.
///
/// Every Bunzo source file is treated as a module (per Language_spec §Modules).
/// The top-level executable statements of a file are collected into a
/// synthesised function named `__main__`.  User-declared `func` definitions
/// become additional functions in this module.
///
/// # Extension points
///
/// Future work may add:
/// - `global_constants` — module-level `const` declarations.
/// - `struct_layouts` — field order and size information for native codegen.
/// - `imported_modules` — resolved import statements.
/// - `debug_info` — source-location mapping for debugger support.
#[derive(Debug, Clone)]
pub struct IrModule {
    /// The source file name, used in diagnostics and debug output.
    pub source_name: std::string::String,

    /// All functions in this module, including the synthesised `__main__`.
    ///
    /// Functions appear in the order they are discovered during lowering:
    /// `__main__` first, then user-declared functions in source order.
    pub functions: Vec<IrFunction>,
}

impl IrModule {
    /// Creates a new, empty module for the given source file.
    pub fn new(source_name: impl Into<std::string::String>) -> Self {
        Self {
            source_name: source_name.into(),
            functions: Vec::new(),
        }
    }

    /// Appends a function to this module.
    pub fn add_function(&mut self, function: IrFunction) {
        self.functions.push(function);
    }

    /// Looks up a function by name.
    ///
    /// Returns `None` if no function with that name exists in the module.
    pub fn get_function(&self, name: &str) -> Option<&IrFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Returns the total number of instructions across all functions.
    ///
    /// Primarily useful for testing and diagnostic output.
    pub fn instruction_count(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|f| &f.blocks)
            .map(|b| b.instructions.len())
            .sum()
    }
}
