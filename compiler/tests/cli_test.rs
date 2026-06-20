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
    let args = make_args(&["bzc", "build", "hello.bz"]);
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
