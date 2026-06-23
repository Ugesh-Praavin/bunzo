//! Bunzo IR Optimizer pass.
//!
//! Provides optimizations on the platform-independent Bunzo IR, such as
//! constant folding, constant propagation, CFG simplification, and dead
//! code elimination.

use std::collections::{HashMap, HashSet};

use super::module::IrModule;
use super::function::{IrFunction, BasicBlock};
use super::instructions::{Instruction, Operand, Constant, VirtualRegister, BinOpKind, UnaryOpKind};
use super::types::IrType;

/// Apply optimization passes on the given IR module.
///
/// Modifies the module in-place.
pub fn optimize(module: &mut IrModule) {
    for func in &mut module.functions {
        optimize_function(func);
    }
}

/// Optimizes a single function until a fixed point is reached.
fn optimize_function(func: &mut IrFunction) {
    loop {
        let mut changed = false;

        changed |= fold_constants(func);
        changed |= simplify_cfg(func);
        changed |= eliminate_dead_code(func);

        if !changed {
            break;
        }
    }
}

// ─── Constant Folding & Propagation ────────────────────────────────────────

/// Propagates constants and folds operations on constants.
fn fold_constants(func: &mut IrFunction) -> bool {
    let mut changed = false;
    let mut consts = HashMap::new();

    // 1. Gather all existing constants
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Instruction::Const { dest, value, .. } = inst {
                consts.insert(*dest, value.clone());
            }
        }
    }

    // 2. Propagate and Fold
    for block in &mut func.blocks {
        for inst in &mut block.instructions {
            // Propagate in operands
            match inst {
                Instruction::BinOp { left, right, .. } => {
                    changed |= propagate_operand(left, &consts);
                    changed |= propagate_operand(right, &consts);
                }
                Instruction::UnaryOp { operand, .. } => {
                    changed |= propagate_operand(operand, &consts);
                }
                Instruction::Call { callee, args, .. } => {
                    changed |= propagate_operand(callee, &consts);
                    for arg in args {
                        changed |= propagate_operand(arg, &consts);
                    }
                }
                Instruction::CallVoid { callee, args, .. } => {
                    changed |= propagate_operand(callee, &consts);
                    for arg in args {
                        changed |= propagate_operand(arg, &consts);
                    }
                }
                Instruction::GetField { object, .. } => {
                    changed |= propagate_operand(object, &consts);
                }
                Instruction::Store { value, .. } => {
                    changed |= propagate_operand(value, &consts);
                }
                Instruction::Print { value } => {
                    changed |= propagate_operand(value, &consts);
                }
                Instruction::SetField { object, value, .. } => {
                    changed |= propagate_operand(object, &consts);
                    changed |= propagate_operand(value, &consts);
                }
                Instruction::Branch { condition, .. } => {
                    changed |= propagate_operand(condition, &consts);
                }
                Instruction::Return { value: Some(value) } => {
                    changed |= propagate_operand(value, &consts);
                }
                _ => {}
            }

            // Now perform folding
            let folded = match inst {
                Instruction::BinOp { dest, op, left: Operand::Constant(l), right: Operand::Constant(r), .. } => {
                    eval_binop(*op, l, r).map(|(ty, val)| (*dest, ty, val))
                }
                Instruction::UnaryOp { dest, op, operand: Operand::Constant(o), .. } => {
                    eval_unary_op(*op, o).map(|(ty, val)| (*dest, ty, val))
                }
                _ => None,
            };

            if let Some((dest, ty, val)) = folded {
                consts.insert(dest, val.clone());
                *inst = Instruction::Const { dest, ty, value: val };
                changed = true;
            }
        }
    }

    changed
}

/// Helper to propagate constants in operands.
fn propagate_operand(operand: &mut Operand, consts: &HashMap<VirtualRegister, Constant>) -> bool {
    if let Operand::Register(r) = operand {
        if let Some(c) = consts.get(r) {
            *operand = Operand::Constant(c.clone());
            return true;
        }
    }
    false
}

/// Evaluates a binary operation on two constant values.
fn eval_binop(op: BinOpKind, left: &Constant, right: &Constant) -> Option<(IrType, Constant)> {
    match (left, right) {
        (Constant::Int(a), Constant::Int(b)) => {
            match op {
                BinOpKind::Add => Some((IrType::Int, Constant::Int(a.wrapping_add(*b)))),
                BinOpKind::Subtract => Some((IrType::Int, Constant::Int(a.wrapping_sub(*b)))),
                BinOpKind::Multiply => Some((IrType::Int, Constant::Int(a.wrapping_mul(*b)))),
                BinOpKind::Divide => {
                    if *b == 0 { None } else { Some((IrType::Int, Constant::Int(a / b))) }
                }
                BinOpKind::Modulo => {
                    if *b == 0 { None } else { Some((IrType::Int, Constant::Int(a % b))) }
                }
                BinOpKind::Equal => Some((IrType::Bool, Constant::Bool(a == b))),
                BinOpKind::NotEqual => Some((IrType::Bool, Constant::Bool(a != b))),
                BinOpKind::Less => Some((IrType::Bool, Constant::Bool(a < b))),
                BinOpKind::LessEqual => Some((IrType::Bool, Constant::Bool(a <= b))),
                BinOpKind::Greater => Some((IrType::Bool, Constant::Bool(a > b))),
                BinOpKind::GreaterEqual => Some((IrType::Bool, Constant::Bool(a >= b))),
                _ => None,
            }
        }
        (Constant::Float(a), Constant::Float(b)) => {
            match op {
                BinOpKind::Add => Some((IrType::Float, Constant::Float(a + b))),
                BinOpKind::Subtract => Some((IrType::Float, Constant::Float(a - b))),
                BinOpKind::Multiply => Some((IrType::Float, Constant::Float(a * b))),
                BinOpKind::Divide => {
                    if *b == 0.0 { None } else { Some((IrType::Float, Constant::Float(a / b))) }
                }
                BinOpKind::Equal => Some((IrType::Bool, Constant::Bool(a == b))),
                BinOpKind::NotEqual => Some((IrType::Bool, Constant::Bool(a != b))),
                BinOpKind::Less => Some((IrType::Bool, Constant::Bool(a < b))),
                BinOpKind::LessEqual => Some((IrType::Bool, Constant::Bool(a <= b))),
                BinOpKind::Greater => Some((IrType::Bool, Constant::Bool(a > b))),
                BinOpKind::GreaterEqual => Some((IrType::Bool, Constant::Bool(a >= b))),
                _ => None,
            }
        }
        (Constant::Bool(a), Constant::Bool(b)) => {
            match op {
                BinOpKind::And => Some((IrType::Bool, Constant::Bool(*a && *b))),
                BinOpKind::Or => Some((IrType::Bool, Constant::Bool(*a || *b))),
                BinOpKind::Equal => Some((IrType::Bool, Constant::Bool(a == b))),
                BinOpKind::NotEqual => Some((IrType::Bool, Constant::Bool(a != b))),
                _ => None,
            }
        }
        (Constant::String(a), Constant::String(b)) => {
            match op {
                BinOpKind::Add => Some((IrType::String, Constant::String(format!("{}{}", a, b)))),
                BinOpKind::Equal => Some((IrType::Bool, Constant::Bool(a == b))),
                BinOpKind::NotEqual => Some((IrType::Bool, Constant::Bool(a != b))),
                _ => None,
            }
        }
        (Constant::Null, Constant::Null) => {
            match op {
                BinOpKind::Equal => Some((IrType::Bool, Constant::Bool(true))),
                BinOpKind::NotEqual => Some((IrType::Bool, Constant::Bool(false))),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Evaluates a unary operation on a constant value.
fn eval_unary_op(op: UnaryOpKind, operand: &Constant) -> Option<(IrType, Constant)> {
    match operand {
        Constant::Int(v) => match op {
            UnaryOpKind::Negate => Some((IrType::Int, Constant::Int(v.wrapping_neg()))),
            _ => None,
        },
        Constant::Float(v) => match op {
            UnaryOpKind::Negate => Some((IrType::Float, Constant::Float(-v))),
            _ => None,
        },
        Constant::Bool(v) => match op {
            UnaryOpKind::Not => Some((IrType::Bool, Constant::Bool(!v))),
            _ => None,
        },
        _ => None,
    }
}

// ─── Control Flow Graph (CFG) Simplification ────────────────────────────────

fn simplify_cfg(func: &mut IrFunction) -> bool {
    let mut changed = false;
    changed |= fold_branches(func);
    changed |= simplify_jumps(func);
    changed |= eliminate_unreachable_blocks(func);
    changed
}

/// Folds conditional branches with constant conditions into unconditional jumps.
fn fold_branches(func: &mut IrFunction) -> bool {
    let mut changed = false;
    for block in &mut func.blocks {
        if let Some(term) = block.instructions.last_mut() {
            if let Instruction::Branch { condition: Operand::Constant(Constant::Bool(b)), then_label, else_label } = term {
                let target = if *b { then_label.clone() } else { else_label.clone() };
                *term = Instruction::Jump { target };
                changed = true;
            }
        }
    }
    changed
}

/// Simplifies forwarding jumps (blocks containing only a jump to another block).
fn simplify_jumps(func: &mut IrFunction) -> bool {
    let mut changed = false;
    let mut forwarding = HashMap::new();

    for block in &func.blocks {
        if block.instructions.len() == 1 {
            if let Some(Instruction::Jump { target }) = block.instructions.first() {
                if target != &block.label {
                    forwarding.insert(block.label.clone(), target.clone());
                }
            }
        }
    }

    // Resolve chains
    for (from, to) in forwarding.clone() {
        let mut target = to;
        let mut visited = HashSet::new();
        visited.insert(from.clone());

        while let Some(next_target) = forwarding.get(&target) {
            if !visited.insert(target.clone()) {
                break;
            }
            target = next_target.clone();
        }
        forwarding.insert(from, target);
    }

    if forwarding.is_empty() {
        return false;
    }

    for block in &mut func.blocks {
        for inst in &mut block.instructions {
            match inst {
                Instruction::Jump { target } => {
                    if let Some(new_target) = forwarding.get(target) {
                        *target = new_target.clone();
                        changed = true;
                    }
                }
                Instruction::Branch { then_label, else_label, .. } => {
                    if let Some(new_target) = forwarding.get(then_label) {
                        *then_label = new_target.clone();
                        changed = true;
                    }
                    if let Some(new_target) = forwarding.get(else_label) {
                        *else_label = new_target.clone();
                        changed = true;
                    }
                }
                _ => {}
            }
        }
    }

    changed
}

/// Eliminates blocks that are unreachable from the entry block.
fn eliminate_unreachable_blocks(func: &mut IrFunction) -> bool {
    if func.blocks.is_empty() {
        return false;
    }

    let mut reachable = HashSet::new();
    let mut worklist = Vec::new();

    let entry_label = func.blocks[0].label.clone();
    reachable.insert(entry_label.clone());
    worklist.push(entry_label);

    let label_to_block: HashMap<String, &BasicBlock> = func.blocks.iter()
        .map(|b| (b.label.clone(), b))
        .collect();

    while let Some(current_label) = worklist.pop() {
        if let Some(block) = label_to_block.get(&current_label) {
            if let Some(term) = block.instructions.last() {
                match term {
                    Instruction::Jump { target } => {
                        if reachable.insert(target.clone()) {
                            worklist.push(target.clone());
                        }
                    }
                    Instruction::Branch { then_label, else_label, .. } => {
                        if reachable.insert(then_label.clone()) {
                            worklist.push(then_label.clone());
                        }
                        if reachable.insert(else_label.clone()) {
                            worklist.push(else_label.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let old_len = func.blocks.len();
    func.blocks.retain(|b| reachable.contains(&b.label));
    func.blocks.len() != old_len
}

// ─── Dead Code Elimination ──────────────────────────────────────────────────

fn eliminate_dead_code(func: &mut IrFunction) -> bool {
    let mut changed = false;
    let uses = count_register_uses(func);

    for block in &mut func.blocks {
        let mut new_instructions = Vec::with_capacity(block.instructions.len());
        for inst in block.instructions.drain(..) {
            if let Some(dest) = inst.dest_register() {
                let use_count = uses.get(&dest).copied().unwrap_or(0);
                if use_count == 0 {
                    match inst {
                        Instruction::Const { .. }
                        | Instruction::Load { .. }
                        | Instruction::BinOp { .. }
                        | Instruction::UnaryOp { .. }
                        | Instruction::GetField { .. } => {
                            changed = true;
                            continue;
                        }
                        Instruction::Call { callee, args, .. } => {
                            new_instructions.push(Instruction::CallVoid { callee, args });
                            changed = true;
                            continue;
                        }
                        _ => unreachable!("Instruction has dest_register but is not matched"),
                    }
                }
            }
            new_instructions.push(inst);
        }
        block.instructions = new_instructions;
    }

    changed
}

/// Counts the number of times each virtual register is read in the function.
fn count_register_uses(func: &IrFunction) -> HashMap<VirtualRegister, usize> {
    let mut uses = HashMap::new();

    let mut add_use = |op: &Operand| {
        if let Operand::Register(r) = op {
            *uses.entry(*r).or_insert(0) += 1;
        }
    };

    for block in &func.blocks {
        for inst in &block.instructions {
            match inst {
                Instruction::Const { .. } => {}
                Instruction::Load { .. } => {}
                Instruction::BinOp { left, right, .. } => {
                    add_use(left);
                    add_use(right);
                }
                Instruction::UnaryOp { operand, .. } => {
                    add_use(operand);
                }
                Instruction::Call { callee, args, .. } => {
                    add_use(callee);
                    for arg in args {
                        add_use(arg);
                    }
                }
                Instruction::CallVoid { callee, args, .. } => {
                    add_use(callee);
                    for arg in args {
                        add_use(arg);
                    }
                }
                Instruction::GetField { object, .. } => {
                    add_use(object);
                }
                Instruction::Store { value, .. } => {
                    add_use(value);
                }
                Instruction::Print { value } => {
                    add_use(value);
                }
                Instruction::SetField { object, value, .. } => {
                    add_use(object);
                    add_use(value);
                }
                Instruction::Branch { condition, .. } => {
                    add_use(condition);
                }
                Instruction::Return { value: Some(value) } => {
                    add_use(value);
                }
                _ => {}
            }
        }
    }

    uses
}
