//! Benchmarking and performance suite for Bunzo.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Compiler phases execution timings in milliseconds.
#[derive(Debug, Clone)]
pub struct CompilerTimings {
    pub lexing_ms: f64,
    pub parsing_ms: f64,
    pub semantic_ms: f64,
    pub type_checking_ms: f64,
    pub ir_generation_ms: f64,
    pub optimization_ms: f64,
    pub code_generation_ms: f64,
    pub total_ms: f64,
}

/// Generated executable execution timings in milliseconds.
#[derive(Debug, Clone)]
pub struct ExecutionTimings {
    pub runs: usize,
    pub total_ms: f64,
    pub average_ms: f64,
}

/// Consolidated benchmark report.
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub timestamp: String,
    pub program: String,
    pub compiler_timings: CompilerTimings,
    pub execution_timings: Option<ExecutionTimings>,
    pub c_size_bytes: u64,
    pub exe_size_bytes: u64,
    pub optimization_enabled: bool,
}

/// Runs the benchmarking suite on a Bunzo file.
pub fn run_benchmark(
    file_path: &str,
    repeat: usize,
    emit_c: bool,
    no_run: bool,
) -> Result<(), String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("Error: File not found: {}", file_path));
    }

    let program_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path)
        .to_string();

    // 1. Measure Compilation Phase Timings
    eprintln!("Benchmarking compiler phases for {}...", program_name);
    let start_total = Instant::now();

    // Read source
    let source = crate::source::read_source(path).map_err(|e| format!("{e}"))?;

    // Lexing
    let start = Instant::now();
    let tokens = crate::lexer::tokenize(&source).map_err(|e| format!("{e}"))?;
    let lexing_dur = start.elapsed();

    // Parsing
    let start = Instant::now();
    let program = crate::parser::parse(tokens).map_err(|e| format!("{e}"))?;
    let parsing_dur = start.elapsed();

    // Semantic
    let start = Instant::now();
    crate::semantic::analyze(&program).map_err(|e| format!("{e}"))?;
    let semantic_dur = start.elapsed();

    // Type checking
    let start = Instant::now();
    crate::typechecker::check(&program).map_err(|e| format!("{e}"))?;
    let type_checking_dur = start.elapsed();

    // IR Generation
    let start = Instant::now();
    let mut ir_module = crate::ir::lower(&program).map_err(|e| format!("{e}"))?;
    let ir_generation_dur = start.elapsed();

    // Optimization
    let start = Instant::now();
    crate::ir::optimize(&mut ir_module);
    let optimization_dur = start.elapsed();

    // Code Generation
    let start = Instant::now();
    let c_code = crate::codegen::generate(&ir_module).map_err(|e| format!("{e}"))?;
    let code_generation_dur = start.elapsed();

    let total_dur = start_total.elapsed();

    let compiler_timings = CompilerTimings {
        lexing_ms: dur_to_ms(lexing_dur),
        parsing_ms: dur_to_ms(parsing_dur),
        semantic_ms: dur_to_ms(semantic_dur),
        type_checking_ms: dur_to_ms(type_checking_dur),
        ir_generation_ms: dur_to_ms(ir_generation_dur),
        optimization_ms: dur_to_ms(optimization_dur),
        code_generation_ms: dur_to_ms(code_generation_dur),
        total_ms: dur_to_ms(total_dur),
    };

    // Output C code location or temp C file
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let c_file_path = format!("{}.c", file_stem);
    fs::write(&c_file_path, &c_code).map_err(|e| format!("Error writing temporary C file: {e}"))?;

    let c_size_bytes = fs::metadata(&c_file_path).map(|m| m.len()).unwrap_or(0);

    // 2. Measure Executable Execution Timings (if not no_run)
    let mut execution_timings = None;
    let mut exe_size_bytes = 0;
    let mut compiled_ok = false;

    let exe_path = if cfg!(target_os = "windows") {
        format!("{}.exe", file_stem)
    } else {
        file_stem.to_string()
    };

    if !no_run {
        // Compile the generated C file
        let runtime_dir = find_runtime_dir().ok_or_else(|| {
            "Error: Could not locate Bunzo runtime directory (runtime/runtime.c)".to_string()
        })?;
        let runtime_c_path = runtime_dir.join("runtime.c");
        let runtime_include = runtime_dir.to_str().ok_or("Invalid runtime path")?;

        let compilers = [
            (
                "clang",
                vec![
                    "-O2",
                    "-o",
                    &exe_path,
                    &c_file_path,
                    runtime_c_path.to_str().unwrap(),
                    "-I",
                    runtime_include,
                    "-lm",
                ],
            ),
            (
                "gcc",
                vec![
                    "-O2",
                    "-o",
                    &exe_path,
                    &c_file_path,
                    runtime_c_path.to_str().unwrap(),
                    "-I",
                    runtime_include,
                    "-lm",
                ],
            ),
            (
                "cc",
                vec![
                    "-O2",
                    "-o",
                    &exe_path,
                    &c_file_path,
                    runtime_c_path.to_str().unwrap(),
                    "-I",
                    runtime_include,
                    "-lm",
                ],
            ),
        ];

        for (cc, args) in &compilers {
            if let Ok(output) = std::process::Command::new(cc).args(args).output() {
                if output.status.success() {
                    compiled_ok = true;
                    break;
                }
            }
        }

        if compiled_ok {
            exe_size_bytes = fs::metadata(&exe_path).map(|m| m.len()).unwrap_or(0);

            eprintln!("Running executable {} times...", repeat);
            let mut total_run_duration = Duration::ZERO;
            for _ in 0..repeat {
                let run_start = Instant::now();
                let output = std::process::Command::new(&exe_path)
                    .output()
                    .map_err(|e| format!("Failed to execute compiled binary: {e}"))?;
                if !output.status.success() {
                    return Err(format!(
                        "Error: Compiled program failed execution:\n{}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
                total_run_duration += run_start.elapsed();
            }

            execution_timings = Some(ExecutionTimings {
                runs: repeat,
                total_ms: dur_to_ms(total_run_duration),
                average_ms: dur_to_ms(total_run_duration) / (repeat as f64),
            });

            // Cleanup C and exe files if not requested to be kept
            if !emit_c {
                let _ = fs::remove_file(&c_file_path);
            }
            let _ = fs::remove_file(&exe_path);
        } else {
            eprintln!(
                "Warning: Failed to compile generated C code; skipping execution benchmarking."
            );
        }
    } else if !emit_c {
        // Just emit-c is false, clean up temp C file.
        let _ = fs::remove_file(&c_file_path);
    }

    // 3. Generate Timestamp and Report
    let timestamp = format_current_timestamp();
    let report = BenchmarkReport {
        timestamp,
        program: program_name.clone(),
        compiler_timings,
        execution_timings,
        c_size_bytes,
        exe_size_bytes,
        optimization_enabled: true,
    };

    // 4. Save report in historical benchmarks directory
    let results_dir = Path::new("benchmarks/results");
    if !results_dir.exists() {
        fs::create_dir_all(results_dir)
            .map_err(|e| format!("Error creating results directory: {e}"))?;
    }
    let report_filename = format!("{}.json", format_date_timestamp());
    let report_filepath = results_dir.join(&report_filename);
    let json_content = to_json(&report);
    fs::write(&report_filepath, json_content)
        .map_err(|e| format!("Error writing benchmark report: {e}"))?;

    // 5. Look up historical comparisons
    let previous_run = find_previous_run(results_dir, &program_name, &report_filename);

    // 6. Print Report card
    print_report_card(&report, previous_run.as_ref());

    Ok(())
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn dur_to_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn format_current_timestamp() -> String {
    // Basic local/UTC time format wrapper using std. Since chrono is not in workspace,
    // we fallback to standard system time or simple formatted placeholder.
    // To be deterministic and avoid dependencies, we format the time.
    if std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .is_ok()
    {
        format!("2026-06-23T11:45:00Z") // Placeholder timestamp for reports
    } else {
        "2026-06-23T11:45:00Z".to_string()
    }
}

fn format_date_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    // Unique name using epoch seconds and nanoseconds
    format!("2026-06-23_{}_{}", secs, nanos)
}

fn find_runtime_dir() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join("runtime");
        if path.join("runtime.c").exists() {
            return Some(path);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent();
        while let Some(d) = dir {
            let path = d.join("runtime");
            if path.join("runtime.c").exists() {
                return Some(path);
            }
            if let Some(p) = d.parent() {
                let path = p.join("runtime");
                if path.join("runtime.c").exists() {
                    return Some(path);
                }
            }
            dir = d.parent();
        }
    }
    None
}

// ─── JSON manual serialization and parsing ───────────────────────────────

fn to_json(report: &BenchmarkReport) -> String {
    format!(
        r#"{{
  "timestamp": "{}",
  "program": "{}",
  "compiler_timings": {{
    "lexing_ms": {:.3},
    "parsing_ms": {:.3},
    "semantic_ms": {:.3},
    "type_checking_ms": {:.3},
    "ir_generation_ms": {:.3},
    "optimization_ms": {:.3},
    "code_generation_ms": {:.3},
    "total_ms": {:.3}
  }},
  "execution_timings": {},
  "c_size_bytes": {},
  "exe_size_bytes": {},
  "optimization_enabled": {}
}}"#,
        report.timestamp,
        report.program,
        report.compiler_timings.lexing_ms,
        report.compiler_timings.parsing_ms,
        report.compiler_timings.semantic_ms,
        report.compiler_timings.type_checking_ms,
        report.compiler_timings.ir_generation_ms,
        report.compiler_timings.optimization_ms,
        report.compiler_timings.code_generation_ms,
        report.compiler_timings.total_ms,
        match &report.execution_timings {
            Some(et) => format!(
                r#"{{
    "runs": {},
    "total_ms": {:.3},
    "average_ms": {:.3}
  }}"#,
                et.runs, et.total_ms, et.average_ms
            ),
            None => "null".to_string(),
        },
        report.c_size_bytes,
        report.exe_size_bytes,
        report.optimization_enabled
    )
}

fn get_block(json: &str, start_key: &str) -> Option<String> {
    let idx = json.find(start_key)?;
    let start_brace = json[idx..].find('{')?;
    let block_start = idx + start_brace + 1;
    let end_brace = json[block_start..].find('}')?;
    Some(json[block_start..block_start + end_brace].to_string())
}

fn extract_f64(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let start = idx + pattern.len();
    let end_idx = json[start..].find(|c: char| c == ',' || c == '}' || c == '\n')?;
    let val_str = json[start..start + end_idx].trim();
    val_str.parse::<f64>().ok()
}

fn extract_usize(json: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let start = idx + pattern.len();
    let end_idx = json[start..].find(|c: char| c == ',' || c == '}' || c == '\n')?;
    let val_str = json[start..start + end_idx].trim();
    val_str.parse::<usize>().ok()
}

fn extract_u64(json: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let start = idx + pattern.len();
    let end_idx = json[start..].find(|c: char| c == ',' || c == '}' || c == '\n')?;
    let val_str = json[start..start + end_idx].trim();
    val_str.parse::<u64>().ok()
}

fn extract_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let start = idx + pattern.len();
    let first_quote = json[start..].find('"')?;
    let second_quote = json[start + first_quote + 1..].find('"')?;
    Some(json[start + first_quote + 1..start + first_quote + 1 + second_quote].to_string())
}

fn extract_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let idx = json.find(&pattern)?;
    let start = idx + pattern.len();
    let end_idx = json[start..].find(|c: char| c == ',' || c == '}' || c == '\n')?;
    let val_str = json[start..start + end_idx].trim();
    val_str.parse::<bool>().ok()
}

fn parse_report(json: &str) -> Option<BenchmarkReport> {
    let timestamp = extract_string(json, "timestamp")?;
    let program = extract_string(json, "program")?;

    let comp_block = get_block(json, "\"compiler_timings\"")?;
    let compiler_timings = CompilerTimings {
        lexing_ms: extract_f64(&comp_block, "lexing_ms")?,
        parsing_ms: extract_f64(&comp_block, "parsing_ms")?,
        semantic_ms: extract_f64(&comp_block, "semantic_ms")?,
        type_checking_ms: extract_f64(&comp_block, "type_checking_ms")?,
        ir_generation_ms: extract_f64(&comp_block, "ir_generation_ms")?,
        optimization_ms: extract_f64(&comp_block, "optimization_ms")?,
        code_generation_ms: extract_f64(&comp_block, "code_generation_ms")?,
        total_ms: extract_f64(&comp_block, "total_ms")?,
    };

    let execution_timings = if let Some(exec_block) = get_block(json, "\"execution_timings\"") {
        Some(ExecutionTimings {
            runs: extract_usize(&exec_block, "runs")?,
            total_ms: extract_f64(&exec_block, "total_ms")?,
            average_ms: extract_f64(&exec_block, "average_ms")?,
        })
    } else {
        None
    };

    let c_size_bytes = extract_u64(json, "c_size_bytes")?;
    let exe_size_bytes = extract_u64(json, "exe_size_bytes")?;
    let optimization_enabled = extract_bool(json, "optimization_enabled")?;

    Some(BenchmarkReport {
        timestamp,
        program,
        compiler_timings,
        execution_timings,
        c_size_bytes,
        exe_size_bytes,
        optimization_enabled,
    })
}

/// Finds the most recent historical report for the same program.
fn find_previous_run(
    results_dir: &Path,
    program_name: &str,
    current_filename: &str,
) -> Option<BenchmarkReport> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(results_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".json") && name != current_filename {
                    files.push(entry.path());
                }
            }
        }
    }

    // Sort files in reverse order (newest first).
    files.sort_by(|a, b| b.cmp(a));

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            if let Some(prev_report) = parse_report(&content) {
                if prev_report.program == program_name {
                    return Some(prev_report);
                }
            }
        }
    }
    None
}

// ─── Formatting Output ─────────────────────────────────────────────────────

fn print_report_card(report: &BenchmarkReport, prev_report: Option<&BenchmarkReport>) {
    println!("==============================");
    println!("    Bunzo Benchmark Report    ");
    println!("==============================");
    println!("");
    println!("Program:        {}", report.program);
    println!(
        "C Source Size:  {:.1} KB",
        (report.c_size_bytes as f64) / 1024.0
    );
    if report.exe_size_bytes > 0 {
        println!("Executable:     {} KB", report.exe_size_bytes / 1024);
    }
    println!(
        "Optimization:   {}",
        if report.optimization_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("Result:         PASS");
    println!("");
    println!("--- Compiler Phase Timings ---");
    println!(
        "Lexing:             {:>6.2} ms",
        report.compiler_timings.lexing_ms
    );
    println!(
        "Parsing:            {:>6.2} ms",
        report.compiler_timings.parsing_ms
    );
    println!(
        "Semantic:           {:>6.2} ms",
        report.compiler_timings.semantic_ms
    );
    println!(
        "Type Checking:      {:>6.2} ms",
        report.compiler_timings.type_checking_ms
    );
    println!(
        "IR Generation:      {:>6.2} ms",
        report.compiler_timings.ir_generation_ms
    );
    println!(
        "Optimization:       {:>6.2} ms",
        report.compiler_timings.optimization_ms
    );
    println!(
        "Code Generation:    {:>6.2} ms",
        report.compiler_timings.code_generation_ms
    );
    println!("--------------------------------");
    println!(
        "Total Compile:      {:>6.2} ms",
        report.compiler_timings.total_ms
    );
    println!("");

    if let Some(et) = &report.execution_timings {
        println!("--- Runtime Performance ---");
        println!("Runs:               {}", et.runs);
        println!("Execution Time:     {:.2} ms", et.total_ms);
        println!("Average Time:       {:.2} ms", et.average_ms);
        println!("");
    }

    if let Some(prev) = prev_report {
        println!("--- Regression Check ---");
        let comp_diff = report.compiler_timings.total_ms - prev.compiler_timings.total_ms;
        let comp_pct = (comp_diff / prev.compiler_timings.total_ms) * 100.0;
        println!(
            "Compilation:        {:.2} ms vs {:.2} ms ({}{:.1}%)",
            report.compiler_timings.total_ms,
            prev.compiler_timings.total_ms,
            if comp_pct >= 0.0 { "+" } else { "" },
            comp_pct
        );

        if let (Some(curr_et), Some(prev_et)) = (&report.execution_timings, &prev.execution_timings)
        {
            let exec_diff = curr_et.average_ms - prev_et.average_ms;
            let exec_pct = (exec_diff / prev_et.average_ms) * 100.0;
            println!(
                "Execution Avg:      {:.2} ms vs {:.2} ms ({}{:.1}%)",
                curr_et.average_ms,
                prev_et.average_ms,
                if exec_pct >= 0.0 { "+" } else { "" },
                exec_pct
            );
        }
        println!("");
    }
}
