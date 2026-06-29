//! Integration tests for the Bunzo Benchmarking & Performance Suite (Stage 15).

use std::fs;
use std::path::Path;

/// Helper to clean up results specifically generated for a program name to prevent race conditions.
fn cleanup_results(program_name: &str) {
    let results_dir = Path::new("benchmarks/results");
    if let Ok(entries) = fs::read_dir(results_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                if content.contains(&format!("\"program\": \"{}\"", program_name)) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
}

#[test]
fn test_benchmark_no_run_generates_report() {
    let program_name = "test_bench_norun.bz";
    cleanup_results(program_name);

    let source = "
        let x = 42
        print(x)
    ";
    let temp_dir = std::env::temp_dir();
    let temp_file_bz = temp_dir.join(program_name);
    fs::write(&temp_file_bz, source).unwrap();

    let args = vec![
        "bzc".to_string(),
        "benchmark".to_string(),
        temp_file_bz.to_str().unwrap().to_string(),
        "--no-run".to_string(),
    ];

    let result = bzc::cli::run(&args);
    let _ = fs::remove_file(&temp_file_bz);

    assert!(result.is_ok(), "Benchmark run failed: {:?}", result);

    // Verify report was written
    let results_dir = Path::new("benchmarks/results");
    assert!(results_dir.exists(), "Results directory was not created");

    let entries = fs::read_dir(results_dir).unwrap();
    let mut matching_reports = 0;
    for entry in entries {
        if let Ok(entry) = entry {
            let filename = entry.file_name().to_string_lossy().into_owned();
            if filename.ends_with(".json") {
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                if content.contains(&format!("\"program\": \"{}\"", program_name)) {
                    matching_reports += 1;
                    assert!(content.contains("\"compiler_timings\""));
                    assert!(content.contains("\"lexing_ms\""));
                }
            }
        }
    }
    assert_eq!(
        matching_reports, 1,
        "Expected exactly 1 matching JSON report"
    );

    // Clean up
    cleanup_results(program_name);
}

#[test]
fn test_benchmark_multiple_runs_shows_comparison() {
    let program_name = "test_bench_comparison.bz";
    cleanup_results(program_name);

    let source = "
        let sum = 0
        let i = 0
        while i < 1000 {
            sum = sum + i
            i = i + 1
        }
        print(sum)
    ";
    let temp_dir = std::env::temp_dir();
    let temp_file_bz = temp_dir.join(program_name);
    fs::write(&temp_file_bz, source).unwrap();

    let args = vec![
        "bzc".to_string(),
        "benchmark".to_string(),
        temp_file_bz.to_str().unwrap().to_string(),
        "--no-run".to_string(),
    ];

    // Run 1: establishes historical baseline
    let result1 = bzc::cli::run(&args);
    assert!(result1.is_ok());

    // Sleep for 1 second to ensure different timestamp filename
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Run 2: compares with historical baseline
    let result2 = bzc::cli::run(&args);
    let _ = fs::remove_file(&temp_file_bz);
    assert!(result2.is_ok());

    let results_dir = Path::new("benchmarks/results");
    let entries = fs::read_dir(results_dir).unwrap();
    let mut matching_reports = 0;
    for entry in entries {
        if let Ok(entry) = entry {
            let filename = entry.file_name().to_string_lossy().into_owned();
            if filename.ends_with(".json") {
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                if content.contains(&format!("\"program\": \"{}\"", program_name)) {
                    matching_reports += 1;
                }
            }
        }
    }
    assert_eq!(
        matching_reports, 2,
        "Expected exactly 2 matching JSON reports"
    );

    cleanup_results(program_name);
}
