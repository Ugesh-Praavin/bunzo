//! Integration tests for the Bunzo IR optimizer (Milestone 13).

use bzc::ir::{
    self, Constant, IrModule, Instruction, Operand,
};
use bzc::lexer;
use bzc::parser;
use bzc::semantic;
use bzc::typechecker;

/// Parse, analyze, type check, and lower a Bunzo source snippet into an [`IrModule`].
fn lower_source(source: &str) -> IrModule {
    let tokens = lexer::tokenize(source).expect("tokenize failed");
    let program = parser::parse(tokens).expect("parse failed");
    semantic::analyze(&program).expect("semantic analysis failed");
    typechecker::check(&program).expect("type check failed");
    ir::lower(&program).expect("IR lowering failed")
}

/// Lower a source snippet and optimize it.
fn optimize_source(source: &str) -> IrModule {
    let mut module = lower_source(source);
    ir::optimize(&mut module);
    module
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Constant Folding & Propagation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_constant_folding_int_arithmetic() {
    let source = "let x = 3 + 4 * 2";
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    // Check that we have a store instruction with the folded constant 11.
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, value: Operand::Constant(Constant::Int(11)) } if name == "x"
    )));

    // Verify there are no BinOp instructions left in the block.
    assert!(!entry.instructions.iter().any(|i| matches!(i, Instruction::BinOp { .. })));
}

#[test]
fn test_constant_folding_float_arithmetic() {
    let source = "let x = 1.5 + 2.5";
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, value: Operand::Constant(Constant::Float(val)) } if name == "x" && (val - 4.0).abs() < 1e-9
    )));
    assert!(!entry.instructions.iter().any(|i| matches!(i, Instruction::BinOp { .. })));
}

#[test]
fn test_constant_folding_boolean_logic() {
    let source = "let x = true && false";
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, value: Operand::Constant(Constant::Bool(false)) } if name == "x"
    )));
    assert!(!entry.instructions.iter().any(|i| matches!(i, Instruction::BinOp { .. })));
}

#[test]
fn test_constant_folding_string_concatenation() {
    let source = r#"let greeting = "hello " + "world""#;
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, value: Operand::Constant(Constant::String(s)) } if name == "greeting" && s == "hello world"
    )));
    assert!(!entry.instructions.iter().any(|i| matches!(i, Instruction::BinOp { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Control Flow Graph (CFG) Simplification Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_cfg_branch_folding_and_unreachable_block_elimination() {
    // If condition is true, else block should be unreachable.
    let source = r#"
        if true {
            print("then")
        } else {
            print("else")
        }
    "#;
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    // Verify that the "else" block is completely eliminated.
    // The names of remaining blocks should only be entry, then.N, merge.N.
    for block in &main_fn.blocks {
        assert!(!block.label.starts_with("else"));
    }

    // Verify that print("then") is present but print("else") is absent.
    let prints_then = main_fn.blocks.iter().any(|b| {
        b.instructions.iter().any(|i| matches!(
            i,
            Instruction::Print { value: Operand::Constant(Constant::String(s)) } if s == "then"
        ))
    });
    let prints_else = main_fn.blocks.iter().any(|b| {
        b.instructions.iter().any(|i| matches!(
            i,
            Instruction::Print { value: Operand::Constant(Constant::String(s)) } if s == "else"
        ))
    });

    assert!(prints_then);
    assert!(!prints_else);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Dead Code Elimination (DCE) Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dead_code_elimination_unused_registers() {
    // The expression statement `2 + 3` should be completely optimized away since its result is unused.
    let source = "let x = 10\n2 + 3";
    let unoptimized = lower_source(source);
    let optimized = optimize_source(source);

    let unoptimized_insts = unoptimized.instruction_count();
    let optimized_insts = optimized.instruction_count();

    // Optimized IR should contain fewer instructions because the dead BinOp is removed.
    assert!(optimized_insts < unoptimized_insts);
}

#[test]
fn test_dce_unused_call_to_void() {
    // If we call a function but don't use its return value, the destination register is unused.
    // DCE should convert Instruction::Call to Instruction::CallVoid.
    let source = r#"
        func add(a: int, b: int) -> int {
            return a + b
        }
        add(1, 2)
    "#;
    let module = optimize_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    // Check that we have a CallVoid instead of Call.
    let has_call = entry.instructions.iter().any(|i| matches!(i, Instruction::Call { .. }));
    let has_call_void = entry.instructions.iter().any(|i| matches!(i, Instruction::CallVoid { .. }));

    assert!(!has_call);
    assert!(has_call_void);
}
