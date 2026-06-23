//! IR instructions and operands for the Bunzo compiler.
//!
//! An [`Instruction`] is the atomic unit of computation in the IR.
//! Each instruction optionally produces a result stored in a
//! [`VirtualRegister`].  Operands ([`Operand`]) reference either a
//! virtual register or an inline constant.
//!
//! Design goals
//! ─────────────
//! • **Explicit** — every value use is named; no implicit stack.
//! • **Typed** — every operand carries its [`IrType`].
//! • **Flat** — no nested expressions; all sub-expressions are
//!   already lowered to their own registers before being used here.
//! • **Extensible** — new instructions can be added as variants
//!   without affecting existing match arms that use wildcards.

use std::fmt;

use super::types::IrType;

// ─── Virtual Register ──────────────────────────────────────────────────────

/// A numbered virtual register produced by an instruction.
///
/// Virtual registers are local to a single [`crate::ir::function::IrFunction`].
/// They are written exactly once and may be read zero or more times.
/// The numbering starts at `0` and increases monotonically within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualRegister(pub u32);

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

// ─── Constant ──────────────────────────────────────────────────────────────

/// An inline constant value embedded directly in an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// A 64-bit signed integer constant.
    Int(i64),
    /// A 64-bit floating-point constant.
    Float(f64),
    /// A string constant.
    String(std::string::String),
    /// A boolean constant.
    Bool(bool),
    /// The `null` constant.
    Null,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(v) => write!(f, "{v}"),
            Constant::Float(v) => write!(f, "{v}"),
            Constant::String(v) => write!(f, "\"{v}\""),
            Constant::Bool(v) => write!(f, "{v}"),
            Constant::Null => write!(f, "null"),
        }
    }
}

// ─── Operand ───────────────────────────────────────────────────────────────

/// An instruction operand — either a virtual register or an inline constant.
///
/// Every use of a value in an instruction is represented as an `Operand`.
/// This is the primary reference mechanism in the IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// A reference to a previously computed virtual register.
    Register(VirtualRegister),
    /// An inline constant value.
    Constant(Constant),
    /// A reference to a named function in the current module.
    FunctionRef(std::string::String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(r) => write!(f, "{r}"),
            Operand::Constant(c) => write!(f, "{c}"),
            Operand::FunctionRef(name) => write!(f, "@{name}"),
        }
    }
}

// ─── Binary Operation Kind ────────────────────────────────────────────────

/// The arithmetic, comparison, or logical operation for a [`Instruction::BinOp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    // ── Arithmetic ──────────────────────────────────────────────────
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // ── Comparison ──────────────────────────────────────────────────
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // ── Logical ─────────────────────────────────────────────────────
    And,
    Or,
}

impl fmt::Display for BinOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOpKind::Add => "add",
            BinOpKind::Subtract => "sub",
            BinOpKind::Multiply => "mul",
            BinOpKind::Divide => "div",
            BinOpKind::Modulo => "mod",
            BinOpKind::Equal => "eq",
            BinOpKind::NotEqual => "neq",
            BinOpKind::Less => "lt",
            BinOpKind::Greater => "gt",
            BinOpKind::LessEqual => "lte",
            BinOpKind::GreaterEqual => "gte",
            BinOpKind::And => "and",
            BinOpKind::Or => "or",
        };
        write!(f, "{s}")
    }
}

// ─── Unary Operation Kind ─────────────────────────────────────────────────

/// The operation for a [`Instruction::UnaryOp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpKind {
    /// Arithmetic negation (`-x`).
    Negate,
    /// Logical NOT (`!x`).
    Not,
}

impl fmt::Display for UnaryOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOpKind::Negate => write!(f, "neg"),
            UnaryOpKind::Not => write!(f, "not"),
        }
    }
}

// ─── Instruction ──────────────────────────────────────────────────────────

/// A single IR instruction.
///
/// Instructions are the leaves of the IR — they do the actual work.
/// Every instruction belongs to exactly one [`crate::ir::function::BasicBlock`].
///
/// Instructions that produce a value record the result register and
/// its type.  Terminator instructions (`Return`, `Jump`, `Branch`)
/// must be the last instruction in a basic block.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // ── Value-producing instructions ────────────────────────────────
    /// Load a constant value into a virtual register.
    ///
    /// `%dest = const.<type> <value>`
    Const {
        dest: VirtualRegister,
        ty: IrType,
        value: Constant,
    },

    /// Load a named variable from the current scope into a register.
    ///
    /// `%dest = load <name>`
    Load {
        dest: VirtualRegister,
        name: std::string::String,
    },

    /// Binary arithmetic, comparison, or logical operation.
    ///
    /// `%dest = <op> <left> <right>`
    BinOp {
        dest: VirtualRegister,
        op: BinOpKind,
        left: Operand,
        right: Operand,
    },

    /// Unary negation or logical NOT.
    ///
    /// `%dest = <op> <operand>`
    UnaryOp {
        dest: VirtualRegister,
        op: UnaryOpKind,
        operand: Operand,
    },

    /// Call a function and store its result.
    ///
    /// `%dest = call <callee>(<args…>)`
    Call {
        dest: VirtualRegister,
        callee: Operand,
        args: Vec<Operand>,
    },

    /// Access a field on a struct or object.
    ///
    /// `%dest = get_field <object> <field>`
    GetField {
        dest: VirtualRegister,
        object: Operand,
        field: std::string::String,
    },

    // ── Side-effect instructions ────────────────────────────────────
    /// Store a value into a named variable in the current scope.
    ///
    /// `store <name> <value>`
    Store {
        name: std::string::String,
        value: Operand,
    },

    /// Print a value to standard output.
    ///
    /// `print <value>`
    Print { value: Operand },

    /// Set a field on a struct or object.
    ///
    /// `set_field <object> <field> <value>`
    SetField {
        object: Operand,
        field: std::string::String,
        value: Operand,
    },

    /// Call a function for its side effects (no return value captured).
    ///
    /// `call_void <callee>(<args…>)`
    CallVoid { callee: Operand, args: Vec<Operand> },

    // ── Terminator instructions ─────────────────────────────────────
    // Every basic block must end with exactly one terminator.
    /// Unconditionally jump to a basic block.
    ///
    /// `jump <target_label>`
    Jump { target: std::string::String },

    /// Conditionally branch to one of two basic blocks.
    ///
    /// `branch <condition> <then_label> <else_label>`
    Branch {
        condition: Operand,
        then_label: std::string::String,
        else_label: std::string::String,
    },

    /// Return from the current function, optionally with a value.
    ///
    /// `return` or `return <value>`
    Return { value: Option<Operand> },
}

impl Instruction {
    /// Returns `true` if this instruction is a block terminator.
    ///
    /// A basic block must end with exactly one terminator instruction.
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Instruction::Jump { .. } | Instruction::Branch { .. } | Instruction::Return { .. }
        )
    }

    /// Returns the virtual register written by this instruction, if any.
    pub fn dest_register(&self) -> Option<VirtualRegister> {
        match self {
            Instruction::Const { dest, .. }
            | Instruction::Load { dest, .. }
            | Instruction::BinOp { dest, .. }
            | Instruction::UnaryOp { dest, .. }
            | Instruction::Call { dest, .. }
            | Instruction::GetField { dest, .. } => Some(*dest),

            Instruction::Store { .. }
            | Instruction::Print { .. }
            | Instruction::SetField { .. }
            | Instruction::CallVoid { .. }
            | Instruction::Jump { .. }
            | Instruction::Branch { .. }
            | Instruction::Return { .. } => None,
        }
    }
}
