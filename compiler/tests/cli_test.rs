use bzc::cli;

fn make_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn missing_arguments_shows_usage() {
    let args = make_args(&["bzc"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn too_many_arguments_shows_usage() {
    let args = make_args(&["bzc", "run", "a.bz", "extra"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn unknown_command_shows_usage() {
    let args = make_args(&["bzc", "foobar", "hello.bz"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Usage"));
}

#[test]
fn run_missing_file_returns_error() {
    let args = make_args(&["bzc", "run", "nonexistent_file.bz"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("BZ0001"));
}

#[test]
fn run_valid_source_file() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("bzc_test_cli_run.bz");

    std::fs::write(&file_path, "let x = 42").expect("failed to write temp file");

    let args = make_args(&["bzc", "run", file_path.to_str().unwrap()]);
    let result = bzc::cli::run(&args);

    assert!(result.is_ok(), "should succeed: {result:?}");

    // Clean up.
    let _ = std::fs::remove_file(&file_path);
}
