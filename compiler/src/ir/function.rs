//! IR function and basic block definitions.
//!
//! A Bunzo program is compiled to a collection of [`IrFunction`]s, each
//! representing a named callable unit (a top-level `func` declaration or
//! a synthesised `__main__` for the module's top-level statements).
//!
//! Within a function, control flow is expressed through [`BasicBlock`]s —
//! straight-line sequences of [`Instruction`]s terminated by a single
//! terminator (jump, branch, or return).

use super::instructions::Instruction;
use super::types::IrType;

// ─── Parameter ────────────────────────────────────────────────────────────

/// A single parameter of an IR function.
///
/// Parameters are like `Load` instructions that have already been
/// pre-resolved: the calling convention places the argument value
/// into the corresponding virtual register before the entry block runs.
#[derive(Debug, Clone, PartialEq)]
pub struct IrParameter {
    /// The Bunzo source name of the parameter.
    pub name: std::string::String,
    /// The IR type of the parameter.
    pub ty: IrType,
}

// ─── Basic Block ──────────────────────────────────────────────────────────

/// A basic block — a straight-line sequence of instructions.
///
/// A basic block:
/// - Has a unique label within its function.
/// - Contains zero or more non-terminator instructions followed by
///   exactly one terminator instruction (Jump, Branch, or Return).
/// - Is identified by its `label` which is the target of Jump / Branch.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique label for this block within the function.
    ///
    /// Labels follow the pattern `entry`, `then.N`, `else.N`,
    /// `merge.N`, `loop.header.N`, `loop.body.N`, `loop.exit.N`.
    pub label: std::string::String,

    /// The instructions making up this basic block.
    ///
    /// The last instruction must be a terminator.
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    /// Creates a new, empty basic block with the given label.
    pub fn new(label: impl Into<std::string::String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
        }
    }

    /// Appends an instruction to this block.
    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Returns `true` if the block ends with a terminator instruction.
    pub fn is_terminated(&self) -> bool {
        self.instructions
            .last()
            .map(|i| i.is_terminator())
            .unwrap_or(false)
    }
}

// ─── IR Function ──────────────────────────────────────────────────────────

/// A single Bunzo function in the IR.
///
/// An `IrFunction` corresponds to:
/// - A `func` declaration in the source.
/// - A synthesised `__main__` function that wraps top-level statements.
///
/// The function is organised as an ordered list of [`BasicBlock`]s.
/// The first block is always the **entry block** — execution begins there.
#[derive(Debug, Clone)]
pub struct IrFunction {
    /// The function's Bunzo name.
    ///
    /// Top-level statements are lowered into a function named `__main__`.
    pub name: std::string::String,

    /// Ordered list of parameter definitions.
    pub params: Vec<IrParameter>,

    /// The return type of this function.
    pub return_type: IrType,

    /// The basic blocks, in source order.
    ///
    /// `blocks[0]` is always the entry block.
    pub blocks: Vec<BasicBlock>,
}

impl IrFunction {
    /// Creates a new function with the given name, parameters, and return type.
    ///
    /// The function is initially empty — no blocks, no instructions.
    pub fn new(
        name: impl Into<std::string::String>,
        params: Vec<IrParameter>,
        return_type: IrType,
    ) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            blocks: Vec::new(),
        }
    }

    /// Returns a reference to the entry block, if the function has one.
    pub fn entry_block(&self) -> Option<&BasicBlock> {
        self.blocks.first()
    }

    /// Returns a mutable reference to the last block in the function.
    ///
    /// Used by the builder to append instructions to the active block.
    pub fn current_block_mut(&mut self) -> Option<&mut BasicBlock> {
        self.blocks.last_mut()
    }

    /// Appends a new basic block and makes it the current (active) block.
    pub fn push_block(&mut self, block: BasicBlock) {
        self.blocks.push(block);
    }
}
