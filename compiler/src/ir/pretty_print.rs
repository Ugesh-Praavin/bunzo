//! Human-readable IR text format.
//!
//! [`PrettyPrinter`] renders an [`IrModule`] as a compact, readable
//! text format suitable for debugging, testing, and developer inspection.
//!
//! # Example output
//!
//! Given the Bunzo source:
//!
//! ```bunzo
//! func main() {
//!     let x = 10
//!     let y = x + 5
//! }
//! ```
//!
//! The pretty-printer produces:
//!
//! ```text
//! ; IR Module: source.bz
//!
//! function main() -> void:
//! entry:
//!     store x 10
//!     %0 = load x
//!     %1 = add %0 5
//!     store y %1
//!     return
//! ```
//!
//! # Design
//!
//! The format is intentionally readable rather than machine-parseable.
//! Future stages may introduce a canonical binary encoding; this module
//! is only for human consumption and test assertions.

use std::fmt::{self, Write};

use super::function::{BasicBlock, IrFunction};
use super::instructions::{BinOpKind, Instruction, Operand, UnaryOpKind};
use super::module::IrModule;

// ─── Public API ────────────────────────────────────────────────────────────

/// Renders an `IrModule` as a human-readable string.
///
/// This is the primary API used by tests and the CLI.
pub fn print_module(module: &IrModule) -> String {
    let mut buf = String::new();
    write_module(&mut buf, module).expect("string write cannot fail");
    buf
}

// ─── Module rendering ──────────────────────────────────────────────────────

fn write_module(buf: &mut String, module: &IrModule) -> fmt::Result {
    writeln!(buf, "; IR Module: {}", module.source_name)?;

    for function in &module.functions {
        writeln!(buf)?;
        write_function(buf, function)?;
    }

    Ok(())
}

// ─── Function rendering ────────────────────────────────────────────────────

fn write_function(buf: &mut String, func: &IrFunction) -> fmt::Result {
    // Function header: `function name(param: type, …) -> return_type:`
    write!(buf, "function {}(", func.name)?;
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 {
            write!(buf, ", ")?;
        }
        write!(buf, "{}: {}", param.name, param.ty)?;
    }
    writeln!(buf, ") -> {}:", func.return_type)?;

    for block in &func.blocks {
        write_block(buf, block)?;
    }

    Ok(())
}

// ─── Basic block rendering ─────────────────────────────────────────────────

fn write_block(buf: &mut String, block: &BasicBlock) -> fmt::Result {
    // Block label (unindented).
    writeln!(buf, "{}:", block.label)?;

    for instr in &block.instructions {
        write!(buf, "    ")?;
        write_instruction(buf, instr)?;
        writeln!(buf)?;
    }

    Ok(())
}

// ─── Instruction rendering ─────────────────────────────────────────────────

fn write_instruction(buf: &mut String, instr: &Instruction) -> fmt::Result {
    match instr {
        Instruction::Const { dest, ty, value } => {
            write!(buf, "{dest} = const.{ty} {value}")
        }

        Instruction::Load { dest, name } => {
            write!(buf, "{dest} = load {name}")
        }

        Instruction::BinOp {
            dest,
            op,
            left,
            right,
        } => {
            write!(buf, "{dest} = {} {} {}", write_binop(op), write_operand(left), write_operand(right))
        }

        Instruction::UnaryOp { dest, op, operand } => {
            write!(buf, "{dest} = {} {}", write_unaryop(op), write_operand(operand))
        }

        Instruction::Call { dest, callee, args } => {
            write!(buf, "{dest} = call {}(", write_operand(callee))?;
            write_args(buf, args)?;
            write!(buf, ")")
        }

        Instruction::CallVoid { callee, args } => {
            write!(buf, "call_void {}(", write_operand(callee))?;
            write_args(buf, args)?;
            write!(buf, ")")
        }

        Instruction::GetField {
            dest,
            object,
            field,
        } => write!(buf, "{dest} = get_field {} .{field}", write_operand(object)),

        Instruction::SetField {
            object,
            field,
            value,
        } => write!(
            buf,
            "set_field {} .{field} {}",
            write_operand(object),
            write_operand(value)
        ),

        Instruction::Store { name, value } => {
            write!(buf, "store {name} {}", write_operand(value))
        }

        Instruction::Print { value } => {
            write!(buf, "print {}", write_operand(value))
        }

        Instruction::Jump { target } => {
            write!(buf, "jump {target}")
        }

        Instruction::Branch {
            condition,
            then_label,
            else_label,
        } => write!(
            buf,
            "branch {} {then_label} {else_label}",
            write_operand(condition)
        ),

        Instruction::Return { value: None } => write!(buf, "return"),

        Instruction::Return { value: Some(v) } => {
            write!(buf, "return {}", write_operand(v))
        }
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn write_operand(operand: &Operand) -> String {
    operand.to_string()
}

fn write_binop(op: &BinOpKind) -> &'static str {
    match op {
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
    }
}

fn write_unaryop(op: &UnaryOpKind) -> &'static str {
    match op {
        UnaryOpKind::Negate => "neg",
        UnaryOpKind::Not => "not",
    }
}

fn write_args(buf: &mut String, args: &[Operand]) -> fmt::Result {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            write!(buf, ", ")?;
        }
        write!(buf, "{}", write_operand(arg))?;
    }
    Ok(())
}
