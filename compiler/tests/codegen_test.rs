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

#[test]
#[ignore = "requires C compiler"]
fn test_c_garbage_collector_direct() {
    if !has_c_compiler() {
        return;
    }

    let temp_dir = std::env::temp_dir();
    let temp_file_c = temp_dir.join("test_gc_direct.c");
    let temp_file_exe = temp_dir.join("test_gc_direct_exe");

    let c_program = r#"
#include "runtime.h"
#include <stdio.h>
#include <assert.h>
#include <stdbool.h>

typedef struct GCAllocation {
    void* ptr;
    size_t size;
    bool marked;
    struct GCAllocation* next;
} GCAllocation;

extern GCAllocation* bunzo_gc_allocations;

int main() {
    int dummy = 0;
    bunzo_gc_init(&dummy);
    
    void* p1 = bunzo_gc_malloc(128);
    void* p2 = bunzo_gc_malloc(256);
    void* p3 = bunzo_gc_malloc(512);
    
    bunzo_gc_register_root(&p2);
    
    // We clear p3. It is no longer referenced.
    p3 = NULL;
    
    // Keep p1 referenced on stack by printing or passing it to volatile
    // to prevent optimizer from removing it.
    volatile void* p1_vol = p1;
    (void)p1_vol;
    
    bunzo_gc_collect();
    
    // p1 (128 bytes) and p2 (256 bytes) should remain.
    // p3 (512 bytes) should be swept.
    int found_512 = 0;
    int found_256 = 0;
    int found_128 = 0;
    GCAllocation* curr = bunzo_gc_allocations;
    while (curr) {
        if (curr->size == 512) found_512 = 1;
        if (curr->size == 256) found_256 = 1;
        if (curr->size == 128) found_128 = 1;
        curr = curr->next;
    }
    
    assert(found_256 == 1);
    assert(found_512 == 0);
    
    printf("GC test passed!\n");
    bunzo_gc_cleanup();
    return 0;
}
"#;

    fs::write(&temp_file_c, c_program).unwrap();

    let mut runtime_dir = std::path::PathBuf::from("runtime");
    if !runtime_dir.exists() {
        runtime_dir = std::path::PathBuf::from("../runtime");
    }
    let runtime_c_path = runtime_dir.join("runtime.c");
    let runtime_include = runtime_dir.to_str().unwrap().to_string();

    let cc = "gcc";
    let args = vec![
        "-O2", // optimize to clean up stale stack temporaries and registers
        "-o",
        temp_file_exe.to_str().unwrap(),
        temp_file_c.to_str().unwrap(),
        runtime_c_path.to_str().unwrap(),
        "-I",
        &runtime_include,
        "-lm",
    ];

    let output = Command::new(cc).args(&args).output().unwrap();
    if !output.status.success() {
        let _ = fs::remove_file(&temp_file_c);
        panic!(
            "Compilation of GC test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let _ = fs::remove_file(&temp_file_c);

    let output_run = Command::new(temp_file_exe.to_str().unwrap())
        .output()
        .unwrap();
    let _ = fs::remove_file(&temp_file_exe);

    if !output_run.status.success() {
        panic!(
            "GC test run failed!\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output_run.stdout),
            String::from_utf8_lossy(&output_run.stderr)
        );
    }
    let stdout = String::from_utf8(output_run.stdout).unwrap();
    assert!(stdout.contains("GC test passed!"));
}
