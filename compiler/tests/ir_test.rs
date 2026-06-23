//! Integration tests for the Bunzo IR subsystem (Stage 5).
//!
//! Tests are grouped into four categories:
//!
//! 1. **IR construction** — building IR data structures directly via
//!    [`IrBuilder`] without going through the lowering pass.
//! 2. **AST → IR lowering** — verifying that full Bunzo source programs
//!    produce the expected IR structure.
//! 3. **Instruction generation** — spot-checking individual instruction
//!    types and their field values.
//! 4. **Pretty-print output** — asserting that the text representation
//!    contains the expected tokens and structure.
//!
//! All Bunzo source snippets use valid syntax per Language_spec.md:
//! function declarations use `func`, variables use `let` / `const`, etc.

use bzc::ir::{
    self, BasicBlock, BinOpKind, Constant, Instruction, IrBuilder, IrModule, IrParameter, IrType,
    Operand, VirtualRegister,
};
use bzc::lexer;
use bzc::parser;
use bzc::semantic;
use bzc::typechecker;

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Parse, analyse, and lower a Bunzo source snippet into an [`IrModule`].
fn lower_source(source: &str) -> IrModule {
    let tokens = lexer::tokenize(source).expect("tokenize failed");
    let program = parser::parse(tokens).expect("parse failed");
    semantic::analyze(&program).expect("semantic analysis failed");
    typechecker::check(&program).expect("type check failed");
    ir::lower(&program).expect("IR lowering failed")
}

/// Lower a source snippet and render it as a pretty-printed string.
fn ir_text(source: &str) -> String {
    let module = lower_source(source);
    ir::print_module(&module)
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. IR Construction Tests (IrBuilder API)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_builder_creates_empty_module() {
    let builder = IrBuilder::new("test.bz");
    let module = builder.finish();
    assert_eq!(module.source_name, "test.bz");
    assert!(module.functions.is_empty());
}

#[test]
fn test_builder_creates_function() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("greet", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "greet");
    assert_eq!(module.functions[0].return_type, IrType::Void);
}

#[test]
fn test_builder_function_with_params() {
    let mut builder = IrBuilder::new("test.bz");
    let params = vec![
        IrParameter {
            name: "a".to_string(),
            ty: IrType::Int,
        },
        IrParameter {
            name: "b".to_string(),
            ty: IrType::Int,
        },
    ];
    builder.begin_function("add", params, IrType::Int);
    builder.begin_block("entry");
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    let func = &module.functions[0];
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.params[0].name, "a");
    assert_eq!(func.params[1].ty, IrType::Int);
}

#[test]
fn test_builder_allocates_registers_sequentially() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");

    let r0 = builder.alloc_register();
    let r1 = builder.alloc_register();
    let r2 = builder.alloc_register();

    assert_eq!(r0, VirtualRegister(0));
    assert_eq!(r1, VirtualRegister(1));
    assert_eq!(r2, VirtualRegister(2));

    builder.emit_return_void();
    builder.finish_function();
}

#[test]
fn test_builder_resets_registers_per_function() {
    let mut builder = IrBuilder::new("test.bz");

    builder.begin_function("first", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    let r_in_first = builder.alloc_register();
    builder.emit_return_void();
    builder.finish_function();

    builder.begin_function("second", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    let r_in_second = builder.alloc_register();
    builder.emit_return_void();
    builder.finish_function();

    // Register numbering resets for each function.
    assert_eq!(r_in_first, VirtualRegister(0));
    assert_eq!(r_in_second, VirtualRegister(0));
}

#[test]
fn test_builder_fresh_labels_are_unique() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");

    let label_a = builder.fresh_label("then");
    let label_b = builder.fresh_label("then");
    let label_c = builder.fresh_label("else");

    assert_ne!(label_a, label_b);
    assert_ne!(label_a, label_c);
    assert_ne!(label_b, label_c);

    builder.emit_return_void();
    builder.finish_function();
}

#[test]
fn test_builder_emit_const_instruction() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");

    let reg = builder.emit_const(IrType::Int, Constant::Int(42));
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    let block = &module.functions[0].blocks[0];
    assert!(matches!(
        &block.instructions[0],
        Instruction::Const {
            dest,
            ty: IrType::Int,
            value: Constant::Int(42)
        } if dest == &reg
    ));
}

#[test]
fn test_builder_emit_binop() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");

    let left = Operand::Constant(Constant::Int(10));
    let right = Operand::Constant(Constant::Int(5));
    let reg = builder.emit_binop(BinOpKind::Add, left, right);
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    let block = &module.functions[0].blocks[0];
    assert!(matches!(
        &block.instructions[0],
        Instruction::BinOp { dest, op: BinOpKind::Add, .. }
        if dest == &reg
    ));
}

#[test]
fn test_builder_basic_block_is_terminated_after_return() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    assert!(module.functions[0].blocks[0].is_terminated());
}

#[test]
fn test_ir_module_instruction_count() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("f", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    builder.emit_const(IrType::Int, Constant::Int(1));
    builder.emit_const(IrType::Int, Constant::Int(2));
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    // 2 const + 1 return = 3 instructions
    assert_eq!(module.instruction_count(), 3);
}

#[test]
fn test_ir_module_get_function() {
    let mut builder = IrBuilder::new("test.bz");
    builder.begin_function("alpha", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    builder.emit_return_void();
    builder.finish_function();

    builder.begin_function("beta", Vec::new(), IrType::Void);
    builder.begin_block("entry");
    builder.emit_return_void();
    builder.finish_function();

    let module = builder.finish();
    assert!(module.get_function("alpha").is_some());
    assert!(module.get_function("beta").is_some());
    assert!(module.get_function("gamma").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. AST → IR Lowering Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_empty_program_produces_main() {
    let module = lower_source("");
    // An empty program produces a __main__ function.
    assert!(module.get_function("__main__").is_some());
}

#[test]
fn test_lower_let_declaration() {
    // `let x = 10` should produce a store instruction.
    let module = lower_source("let x = 10");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    // Expect: store x 10
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, value: Operand::Constant(Constant::Int(10)) }
        if name == "x"
    )));
}

#[test]
fn test_lower_const_declaration() {
    let module = lower_source("const PI = 3");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    // const is stored like let at the IR level.
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, .. }
        if name == "PI"
    )));
}

#[test]
fn test_lower_string_literal() {
    let module = lower_source(r#"let greeting = "hello""#);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store {
            name,
            value: Operand::Constant(Constant::String(s))
        }
        if name == "greeting" && s == "hello"
    )));
}

#[test]
fn test_lower_boolean_literal() {
    let module = lower_source("let flag = true");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store {
            name,
            value: Operand::Constant(Constant::Bool(true))
        }
        if name == "flag"
    )));
}

#[test]
fn test_lower_null_literal() {
    let module = lower_source("let nothing = null");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store {
            name,
            value: Operand::Constant(Constant::Null)
        }
        if name == "nothing"
    )));
}

#[test]
fn test_lower_print_statement() {
    let module = lower_source(r#"print("hello")"#);
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Print {
            value: Operand::Constant(Constant::String(s))
        }
        if s == "hello"
    )));
}

#[test]
fn test_lower_assignment() {
    let module = lower_source("let x = 1\nx = 2");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    // There should be two store instructions to x.
    let stores_to_x: Vec<_> = entry
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::Store { name, .. } if name == "x"))
        .collect();
    assert_eq!(stores_to_x.len(), 2);
}

#[test]
fn test_lower_binary_expression() {
    let module = lower_source("let result = 3 + 4");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];

    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Add,
            ..
        }
    )));
}

#[test]
fn test_lower_binary_subtraction() {
    let module = lower_source("let x = 10 - 3");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Subtract,
            ..
        }
    )));
}

#[test]
fn test_lower_binary_multiply_divide() {
    let module = lower_source("let a = 2 * 3\nlet b = 10 / 2");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Multiply,
            ..
        }
    )));
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Divide,
            ..
        }
    )));
}

#[test]
fn test_lower_comparison_operators() {
    let module = lower_source("let a = 5 > 3\nlet b = 5 == 5");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Greater,
            ..
        }
    )));
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::BinOp {
            op: BinOpKind::Equal,
            ..
        }
    )));
}

#[test]
fn test_lower_identifier_load() {
    let module = lower_source("let x = 5\nlet y = x");
    let main_fn = module.get_function("__main__").unwrap();
    let entry = &main_fn.blocks[0];
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Load { name, .. } if name == "x"
    )));
}

#[test]
fn test_lower_function_declaration() {
    let source = r#"
func greet() {
    print("hi")
}
"#;
    let module = lower_source(source);

    // There should be a __main__ and a greet function.
    assert!(module.get_function("__main__").is_some());
    assert!(module.get_function("greet").is_some());
}

#[test]
fn test_lower_function_with_params() {
    let source = r#"
func add(a: int, b: int) -> int {
    return a + b
}
"#;
    let module = lower_source(source);
    let add_fn = module.get_function("add").unwrap();
    assert_eq!(add_fn.params.len(), 2);
    assert_eq!(add_fn.params[0].name, "a");
    assert_eq!(add_fn.params[1].name, "b");
    assert_eq!(add_fn.return_type, IrType::Int);
}

#[test]
fn test_lower_function_return_statement() {
    let source = r#"
func getValue() -> int {
    return 42
}
"#;
    let module = lower_source(source);
    let func = module.get_function("getValue").unwrap();
    let entry = &func.blocks[0];
    assert!(
        entry
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Return { value: Some(_) }))
    );
}

#[test]
fn test_lower_if_creates_three_blocks() {
    let source = r#"
let x = 5
if x > 3 {
    let a = 1
} else {
    let b = 2
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    // Should have: entry, then.N, else.N, merge.N = 4 blocks minimum
    assert!(
        main_fn.blocks.len() >= 4,
        "Expected at least 4 blocks for if/else"
    );
}

#[test]
fn test_lower_if_entry_contains_branch() {
    let source = r#"
let flag = true
if flag {
    let x = 1
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    let entry = &main_fn.blocks[0];
    assert!(
        entry
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Branch { .. }))
    );
}

#[test]
fn test_lower_while_creates_loop_blocks() {
    let source = r#"
let x = 0
while x < 5 {
    x = x + 1
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    // Expect loop.header, loop.body, loop.exit blocks
    let labels: Vec<&str> = main_fn.blocks.iter().map(|b| b.label.as_str()).collect();
    assert!(labels.iter().any(|l| l.starts_with("loop.header")));
    assert!(labels.iter().any(|l| l.starts_with("loop.body")));
    assert!(labels.iter().any(|l| l.starts_with("loop.exit")));
}

#[test]
fn test_lower_while_header_has_branch() {
    let source = r#"
let i = 0
while i < 3 {
    i = i + 1
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    let header = main_fn
        .blocks
        .iter()
        .find(|b| b.label.starts_with("loop.header"))
        .expect("loop.header block missing");

    assert!(
        header
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Branch { .. }))
    );
}

#[test]
fn test_lower_for_creates_loop_blocks() {
    let source = r#"
for i in 1..5 {
    print(i)
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();
    let labels: Vec<&str> = main_fn.blocks.iter().map(|b| b.label.as_str()).collect();
    assert!(labels.iter().any(|l| l.starts_with("loop.header")));
    assert!(labels.iter().any(|l| l.starts_with("loop.body")));
    assert!(labels.iter().any(|l| l.starts_with("loop.exit")));
}

#[test]
fn test_lower_for_initialises_loop_variable() {
    let source = r#"
for i in 1..4 {
    print(i)
}
"#;
    let module = lower_source(source);
    let main_fn = module.get_function("__main__").unwrap();

    // The entry block should store the start value into the loop variable.
    let entry = &main_fn.blocks[0];
    assert!(entry.instructions.iter().any(|i| matches!(
        i,
        Instruction::Store { name, .. } if name == "i"
    )));
}

#[test]
fn test_lower_main_ends_with_return() {
    let module = lower_source("let x = 1");
    let main_fn = module.get_function("__main__").unwrap();

    // Some block in __main__ must end with Return.
    assert!(main_fn.blocks.iter().any(|b| {
        b.instructions
            .last()
            .map(|i| matches!(i, Instruction::Return { .. }))
            .unwrap_or(false)
    }));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Type Preservation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ir_type_display_int() {
    assert_eq!(IrType::Int.to_string(), "int");
}

#[test]
fn test_ir_type_display_float() {
    assert_eq!(IrType::Float.to_string(), "float");
}

#[test]
fn test_ir_type_display_string() {
    assert_eq!(IrType::String.to_string(), "string");
}

#[test]
fn test_ir_type_display_bool() {
    assert_eq!(IrType::Bool.to_string(), "bool");
}

#[test]
fn test_ir_type_display_void() {
    assert_eq!(IrType::Void.to_string(), "void");
}

#[test]
fn test_ir_type_display_array() {
    assert_eq!(
        IrType::Array(Box::new(IrType::Int)).to_string(),
        "Array<int>"
    );
}

#[test]
fn test_function_return_type_void_by_default() {
    let source = r#"
func doWork() {
    let x = 1
}
"#;
    let module = lower_source(source);
    let func = module.get_function("doWork").unwrap();
    assert_eq!(func.return_type, IrType::Void);
}

#[test]
fn test_function_return_type_int() {
    let source = r#"
func getValue() -> int {
    return 1
}
"#;
    let module = lower_source(source);
    let func = module.get_function("getValue").unwrap();
    assert_eq!(func.return_type, IrType::Int);
}

#[test]
fn test_function_return_type_string() {
    let source = r#"
func getName() -> string {
    return "bunzo"
}
"#;
    let module = lower_source(source);
    let func = module.get_function("getName").unwrap();
    assert_eq!(func.return_type, IrType::String);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Pretty-Print Output Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_pretty_print_contains_function_header() {
    let text = ir_text("let x = 1");
    assert!(
        text.contains("function __main__()"),
        "Expected function header in:\n{text}"
    );
}

#[test]
fn test_pretty_print_contains_entry_label() {
    let text = ir_text("let x = 1");
    assert!(text.contains("entry:"), "Expected 'entry:' in:\n{text}");
}

#[test]
fn test_pretty_print_contains_store() {
    let text = ir_text("let score = 99");
    assert!(
        text.contains("store score"),
        "Expected 'store score' in:\n{text}"
    );
}

#[test]
fn test_pretty_print_contains_print_instruction() {
    let text = ir_text(r#"print("bunzo")"#);
    assert!(text.contains("print"), "Expected 'print' in:\n{text}");
}

#[test]
fn test_pretty_print_contains_return() {
    let text = ir_text("let x = 1");
    assert!(text.contains("return"), "Expected 'return' in:\n{text}");
}

#[test]
fn test_pretty_print_contains_add_for_binary_expr() {
    let text = ir_text("let result = 3 + 4");
    assert!(text.contains("add"), "Expected 'add' in:\n{text}");
}

#[test]
fn test_pretty_print_contains_function_for_declaration() {
    let source = r#"
func greet() {
    print("hi")
}
"#;
    let text = ir_text(source);
    assert!(
        text.contains("function greet()"),
        "Expected 'function greet()' in:\n{text}"
    );
}

#[test]
fn test_pretty_print_while_contains_loop_labels() {
    let source = r#"
let i = 0
while i < 3 {
    i = i + 1
}
"#;
    let text = ir_text(source);
    assert!(
        text.contains("loop.header"),
        "Expected 'loop.header' in:\n{text}"
    );
    assert!(
        text.contains("loop.exit"),
        "Expected 'loop.exit' in:\n{text}"
    );
}

#[test]
fn test_pretty_print_if_contains_branch() {
    let source = r#"
let x = 5
if x > 3 {
    let a = 1
}
"#;
    let text = ir_text(source);
    assert!(text.contains("branch"), "Expected 'branch' in:\n{text}");
}

#[test]
fn test_pretty_print_for_contains_lt_comparison() {
    let source = r#"
for i in 1..5 {
    print(i)
}
"#;
    let text = ir_text(source);
    // The for loop header compares the loop var with the end bound using lt.
    assert!(text.contains("lt"), "Expected 'lt' instruction in:\n{text}");
}

#[test]
fn test_pretty_print_module_header() {
    let module = lower_source("let x = 1");
    let text = ir::print_module(&module);
    assert!(
        text.starts_with("; IR Module:"),
        "Expected module comment header in:\n{text}"
    );
}

#[test]
fn test_pretty_print_function_with_params() {
    let source = r#"
func add(a: int, b: int) -> int {
    return a + b
}
"#;
    let text = ir_text(source);
    assert!(
        text.contains("a: int") && text.contains("b: int"),
        "Expected param types in:\n{text}"
    );
    assert!(text.contains("-> int"), "Expected return type in:\n{text}");
}

#[test]
fn test_pretty_print_virtual_register_format() {
    // Binary ops produce registers like %0, %1, etc.
    let text = ir_text("let z = 2 + 3");
    assert!(
        text.contains('%'),
        "Expected '%N' register references in:\n{text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Virtual Register and Instruction Unit Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_virtual_register_display() {
    let reg = VirtualRegister(7);
    assert_eq!(reg.to_string(), "%7");
}

#[test]
fn test_constant_int_display() {
    let c = Constant::Int(42);
    assert_eq!(c.to_string(), "42");
}

#[test]
fn test_constant_string_display() {
    let c = Constant::String("hello".to_string());
    assert_eq!(c.to_string(), "\"hello\"");
}

#[test]
fn test_constant_bool_true_display() {
    let c = Constant::Bool(true);
    assert_eq!(c.to_string(), "true");
}

#[test]
fn test_constant_null_display() {
    let c = Constant::Null;
    assert_eq!(c.to_string(), "null");
}

#[test]
fn test_instruction_is_terminator_return() {
    let instr = Instruction::Return { value: None };
    assert!(instr.is_terminator());
}

#[test]
fn test_instruction_is_terminator_jump() {
    let instr = Instruction::Jump {
        target: "somewhere".to_string(),
    };
    assert!(instr.is_terminator());
}

#[test]
fn test_instruction_is_terminator_branch() {
    let instr = Instruction::Branch {
        condition: Operand::Constant(Constant::Bool(true)),
        then_label: "then".to_string(),
        else_label: "else".to_string(),
    };
    assert!(instr.is_terminator());
}

#[test]
fn test_instruction_store_is_not_terminator() {
    let instr = Instruction::Store {
        name: "x".to_string(),
        value: Operand::Constant(Constant::Int(1)),
    };
    assert!(!instr.is_terminator());
}

#[test]
fn test_instruction_const_has_dest_register() {
    let instr = Instruction::Const {
        dest: VirtualRegister(3),
        ty: IrType::Int,
        value: Constant::Int(0),
    };
    assert_eq!(instr.dest_register(), Some(VirtualRegister(3)));
}

#[test]
fn test_instruction_store_has_no_dest_register() {
    let instr = Instruction::Store {
        name: "x".to_string(),
        value: Operand::Constant(Constant::Int(1)),
    };
    assert_eq!(instr.dest_register(), None);
}

#[test]
fn test_basic_block_is_terminated_false_when_empty() {
    let block = BasicBlock::new("test");
    assert!(!block.is_terminated());
}

#[test]
fn test_basic_block_is_terminated_true_after_return() {
    let mut block = BasicBlock::new("test");
    block.push(Instruction::Return { value: None });
    assert!(block.is_terminated());
}
