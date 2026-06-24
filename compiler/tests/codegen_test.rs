//! Integration tests for the Bunzo C Code Generator (Stage 14).

use bzc::ir;
use bzc::lexer;
use bzc::parser;
use bzc::semantic;
use bzc::typechecker;
use std::fs;
use std::process::Command;

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Check if a C compiler is available in the current environment.
fn has_c_compiler() -> bool {
    Command::new("clang").arg("--version").output().is_ok()
        || Command::new("gcc").arg("--version").output().is_ok()
        || Command::new("cc").arg("--version").output().is_ok()
}

/// Helper to compile and run a Bunzo source program, capturing and returning its stdout.
/// Returns None if no C compiler is available.
fn run_codegen_program(source: &str, temp_name: &str) -> Option<String> {
    if !has_c_compiler() {
        return None;
    }

    let temp_dir = std::env::temp_dir();
    let temp_file_bz = temp_dir.join(format!("{}.bz", temp_name));
    let temp_file_exe = temp_dir.join(format!("{}_exe", temp_name));

    // Write source file.
    fs::write(&temp_file_bz, source).unwrap();

    // Call CLI run directly to build.
    let args = vec![
        "bzc".to_string(),
        "build".to_string(),
        temp_file_bz.to_str().unwrap().to_string(),
        "-o".to_string(),
        temp_file_exe.to_str().unwrap().to_string(),
    ];

    let result = bzc::cli::run(&args);
    if let Err(e) = result {
        // Cleanup.
        let _ = fs::remove_file(&temp_file_bz);
        panic!("CLI build failed: {}", e);
    }

    // Clean up .bz source file.
    let _ = fs::remove_file(&temp_file_bz);

    // Run the compiled executable.
    let exe_run_path = temp_file_exe.to_str().unwrap();
    let output = Command::new(exe_run_path)
        .output()
        .expect("Failed to execute compiled binary");

    // Clean up executable.
    let _ = fs::remove_file(&temp_file_exe);

    let stdout = String::from_utf8(output.stdout).unwrap();
    Some(stdout.replace("\r\n", "\n"))
}

/// Generates C code text directly for validation without executing it.
fn generate_c_code(source: &str) -> String {
    let tokens = lexer::tokenize(source).expect("tokenize failed");
    let program = parser::parse(tokens).expect("parse failed");
    semantic::analyze(&program).expect("semantic analysis failed");
    typechecker::check(&program).expect("type check failed");
    let mut ir_module = ir::lower(&program).expect("IR lowering failed");
    ir::optimize(&mut ir_module);
    bzc::codegen::generate(&ir_module).expect("C generation failed")
}

// ═══════════════════════════════════════════════════════════════════════════
// Codegen tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "requires C compiler"]
fn test_codegen_c_output_structure() {
    let source = "
        let x = 42
        print(x)
    ";
    let c_code = generate_c_code(source);
    assert!(c_code.contains("int main("));
    assert!(c_code.contains("bunzo_print_int("));
}

#[test]
#[ignore = "requires C compiler"]
fn test_arithmetic_execution() {
    let source = "
        let x = 10
        let y = 20
        print(x + y)
        print(y - x)
        print(x * y)
        print(y / x)
    ";
    if let Some(stdout) = run_codegen_program(source, "test_arith") {
        assert_eq!(stdout, "30\n10\n200\n2\n");
    }
}

#[test]
#[ignore = "requires C compiler"]
fn test_string_ops_execution() {
    let source = "
        let a = \"hello \"
        let b = \"world\"
        let c = a + b
        print(c)
        print(c == \"hello world\")
        print(c != \"hello world\")
    ";
    if let Some(stdout) = run_codegen_program(source, "test_string") {
        assert_eq!(stdout, "hello world\ntrue\nfalse\n");
    }
}

#[test]
#[ignore = "requires C compiler"]
fn test_conditionals_execution() {
    let source = "
        let x = 42
        if x > 40 {
            print(\"greater\")
        } else {
            print(\"less\")
        }
    ";
    if let Some(stdout) = run_codegen_program(source, "test_cond") {
        assert_eq!(stdout, "greater\n");
    }
}

#[test]
#[ignore = "requires C compiler"]
fn test_loops_execution() {
    let source = "
        let count = 0
        while count < 3 {
            print(count)
            count = count + 1
        }
    ";
    if let Some(stdout) = run_codegen_program(source, "test_loops") {
        assert_eq!(stdout, "0\n1\n2\n");
    }
}

#[test]
#[ignore = "requires C compiler"]
fn test_functions_and_recursion() {
    let source = "
        func fib(n: int) -> int {
            if n <= 1 {
                return n
            }
            return fib(n - 1) + fib(n - 2)
        }
        print(fib(7))
    ";
    if let Some(stdout) = run_codegen_program(source, "test_func") {
        assert_eq!(stdout, "13\n");
    }
}
