//! IR builder — stateful helper for constructing IR incrementally.
//!
//! [`IrBuilder`] provides a high-level API for creating functions, basic
//! blocks, and instructions.  The lowering pass ([`crate::ir::lowering`])
//! uses it exclusively — no IR data structure is constructed directly.
//!
//! # Design
//!
//! The builder maintains:
//! - The [`IrModule`] being built.
//! - The index of the **current function** (the function being lowered).
//! - A monotonically increasing virtual-register counter, reset per function.
//! - A monotonically increasing label counter for generating unique block names.
//!
//! These invariants are maintained internally, so the lowering pass never
//! has to reason about register numbering or block labelling.

use super::function::{BasicBlock, IrFunction, IrParameter};
use super::instructions::{BinOpKind, Constant, Instruction, Operand, UnaryOpKind, VirtualRegister};
use super::module::IrModule;
use super::types::IrType;

/// Stateful IR construction helper.
///
/// Always use the builder to create IR — never construct [`IrModule`],
/// [`IrFunction`], or [`BasicBlock`] directly in lowering code.
pub struct IrBuilder {
    /// The module under construction.
    pub module: IrModule,

    /// Index into `module.functions` for the function currently being built.
    /// `None` when no function is active.
    current_function_index: Option<usize>,

    /// Next virtual register number within the current function.
    /// Reset to `0` at the start of every new function.
    next_register: u32,

    /// Global counter for generating unique block label suffixes.
    /// Never reset — guarantees globally unique labels across the module.
    next_label_id: u32,
}

impl IrBuilder {
    /// Creates a new builder for a module representing `source_name`.
    pub fn new(source_name: impl Into<String>) -> Self {
        Self {
            module: IrModule::new(source_name),
            current_function_index: None,
            next_register: 0,
            next_label_id: 0,
        }
    }

    // ── Register allocation ───────────────────────────────────────────────

    /// Allocates the next virtual register in the current function.
    ///
    /// Panics if called outside a function context (should never happen in
    /// well-formed lowering code).
    pub fn alloc_register(&mut self) -> VirtualRegister {
        let reg = VirtualRegister(self.next_register);
        self.next_register += 1;
        reg
    }

    // ── Label generation ─────────────────────────────────────────────────

    /// Generates a unique block label with the given prefix.
    ///
    /// Examples: `then.0`, `else.1`, `loop.header.2`.
    pub fn fresh_label(&mut self, prefix: &str) -> String {
        let id = self.next_label_id;
        self.next_label_id += 1;
        format!("{prefix}.{id}")
    }

    // ── Function management ───────────────────────────────────────────────

    /// Begins a new function and makes it the current function.
    ///
    /// Resets the virtual-register counter for the new function.
    /// Must be paired with [`finish_function`].
    pub fn begin_function(
        &mut self,
        name: impl Into<String>,
        params: Vec<IrParameter>,
        return_type: IrType,
    ) {
        let function = IrFunction::new(name, params, return_type);
        self.module.functions.push(function);
        self.current_function_index = Some(self.module.functions.len() - 1);
        // Reset per-function register numbering.
        self.next_register = 0;
    }

    /// Returns a mutable reference to the function currently being built.
    ///
    /// # Panics
    ///
    /// Panics when called outside a `begin_function`/`finish_function` pair.
    fn current_function_mut(&mut self) -> &mut IrFunction {
        let idx = self
            .current_function_index
            .expect("IrBuilder: no active function");
        &mut self.module.functions[idx]
    }

    /// Signals that the current function is complete.
    ///
    /// Clears the active-function context.  After this call, callers must
    /// invoke `begin_function` again before emitting any more instructions.
    pub fn finish_function(&mut self) {
        self.current_function_index = None;
    }

    // ── Block management ──────────────────────────────────────────────────

    /// Appends a new basic block to the current function and makes it active.
    ///
    /// All subsequent `emit_*` calls will append instructions to this block.
    pub fn begin_block(&mut self, label: impl Into<String>) {
        let block = BasicBlock::new(label);
        self.current_function_mut().push_block(block);
    }

    /// Returns a mutable reference to the last (active) block.
    ///
    /// # Panics
    ///
    /// Panics when there is no active function or the function has no blocks.
    fn current_block_mut(&mut self) -> &mut BasicBlock {
        self.current_function_mut()
            .current_block_mut()
            .expect("IrBuilder: no active basic block")
    }

    // ── Instruction emission ──────────────────────────────────────────────

    /// Appends an arbitrary instruction to the current block.
    pub fn emit(&mut self, instruction: Instruction) {
        self.current_block_mut().push(instruction);
    }

    /// Emits a `const` instruction and returns the result register.
    ///
    /// `%N = const.<ty> <value>`
    pub fn emit_const(&mut self, ty: IrType, value: Constant) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::Const {
            dest,
            ty,
            value,
        });
        dest
    }

    /// Emits a `load` instruction and returns the result register.
    ///
    /// `%N = load <name>`
    pub fn emit_load(&mut self, name: impl Into<String>) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::Load {
            dest,
            name: name.into(),
        });
        dest
    }

    /// Emits a binary operation instruction and returns the result register.
    ///
    /// `%N = <op> <left> <right>`
    pub fn emit_binop(
        &mut self,
        op: BinOpKind,
        left: Operand,
        right: Operand,
    ) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::BinOp {
            dest,
            op,
            left,
            right,
        });
        dest
    }

    /// Emits a unary operation instruction and returns the result register.
    ///
    /// `%N = <op> <operand>`
    pub fn emit_unary_op(&mut self, op: UnaryOpKind, operand: Operand) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::UnaryOp { dest, op, operand });
        dest
    }

    /// Emits a `store` instruction (no result register).
    ///
    /// `store <name> <value>`
    pub fn emit_store(&mut self, name: impl Into<String>, value: Operand) {
        self.emit(Instruction::Store {
            name: name.into(),
            value,
        });
    }

    /// Emits a `print` instruction (no result register).
    ///
    /// `print <value>`
    pub fn emit_print(&mut self, value: Operand) {
        self.emit(Instruction::Print { value });
    }

    /// Emits a `call` instruction (with a result register) and returns it.
    ///
    /// `%N = call <callee>(<args…>)`
    pub fn emit_call(&mut self, callee: Operand, args: Vec<Operand>) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::Call { dest, callee, args });
        dest
    }

    /// Emits a `call_void` instruction (no result captured).
    ///
    /// `call_void <callee>(<args…>)`
    pub fn emit_call_void(&mut self, callee: Operand, args: Vec<Operand>) {
        self.emit(Instruction::CallVoid { callee, args });
    }

    /// Emits a `get_field` instruction and returns the result register.
    ///
    /// `%N = get_field <object> <field>`
    pub fn emit_get_field(&mut self, object: Operand, field: impl Into<String>) -> VirtualRegister {
        let dest = self.alloc_register();
        self.emit(Instruction::GetField {
            dest,
            object,
            field: field.into(),
        });
        dest
    }

    /// Emits a `set_field` instruction (no result register).
    ///
    /// `set_field <object> <field> <value>`
    pub fn emit_set_field(&mut self, object: Operand, field: impl Into<String>, value: Operand) {
        self.emit(Instruction::SetField {
            object,
            field: field.into(),
            value,
        });
    }

    // ── Terminator emission ───────────────────────────────────────────────

    /// Emits an unconditional `jump` terminator.
    ///
    /// `jump <target>`
    pub fn emit_jump(&mut self, target: impl Into<String>) {
        self.emit(Instruction::Jump {
            target: target.into(),
        });
    }

    /// Emits a conditional `branch` terminator.
    ///
    /// `branch <condition> <then_label> <else_label>`
    pub fn emit_branch(
        &mut self,
        condition: Operand,
        then_label: impl Into<String>,
        else_label: impl Into<String>,
    ) {
        self.emit(Instruction::Branch {
            condition,
            then_label: then_label.into(),
            else_label: else_label.into(),
        });
    }

    /// Emits a `return` terminator without a value.
    ///
    /// `return`
    pub fn emit_return_void(&mut self) {
        self.emit(Instruction::Return { value: None });
    }

    /// Emits a `return` terminator with a value.
    ///
    /// `return <value>`
    pub fn emit_return(&mut self, value: Operand) {
        self.emit(Instruction::Return {
            value: Some(value),
        });
    }

    // ── Module finalisation ───────────────────────────────────────────────

    /// Consumes the builder and returns the completed [`IrModule`].
    pub fn finish(self) -> IrModule {
        self.module
    }
}
