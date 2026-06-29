use bzc::cli;

fn make_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn missing_arguments_shows_usage() {
    let args = make_args(&["bzc"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Usage"));
    assert!(err_str.contains("error[E0002]"));
}

#[test]
fn too_many_arguments_shows_usage() {
    let args = make_args(&["bzc", "run", "a.bz", "extra"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Usage"));
    assert!(err_str.contains("error[E0003]"));
}

#[test]
fn unknown_command_shows_usage() {
    let args = make_args(&["bzc", "foobar", "hello.bz"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("error[E0001]"));
    assert!(err_str.contains("unknown command 'foobar'"));
}

#[test]
fn run_missing_file_returns_error() {
    let args = make_args(&["bzc", "run", "nonexistent_file.bz"]);
    let result = cli::run(&args);

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("BZ0001"));
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

fn get_bzc_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    // env::current_exe() is target/debug/deps/cli_test-xxxxx.exe
    // pop -> target/debug/deps
    // pop -> target/debug
    path.pop();
    path.pop();
    #[cfg(target_os = "windows")]
    {
        path.push("bzc.exe");
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.push("bzc");
    }
    path
}

fn run_bzc_binary(args: &[&str]) -> (i32, String, String) {
    let binary = get_bzc_binary();
    assert!(binary.exists(), "Binary not found at {}", binary.display());

    let output = std::process::Command::new(binary)
        .args(args)
        .output()
        .expect("Failed to execute bzc binary");

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    (status, stdout, stderr)
}

#[test]
fn binary_exit_codes_and_outputs() {
    // 1. Unknown command -> Exit code 2, error message
    let (status, stdout, stderr) = run_bzc_binary(&["abc"]);
    assert_eq!(status, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("error[E0001]: unknown command 'abc'"));
    assert!(stderr.contains("Run 'bzc --help'"));

    // 2. Missing source file for run -> Exit code 2, error message
    let (status, stdout, stderr) = run_bzc_binary(&["run"]);
    assert_eq!(status, 2);
    assert!(stdout.is_empty());
    assert!(stderr.contains("error[E0002]: missing source file"));
    assert!(stderr.contains("bzc run <file.bz>"));

    // 3. Version flag -> Exit code 0, version output
    let (status, stdout, stderr) = run_bzc_binary(&["--version"]);
    assert_eq!(status, 0);
    assert!(stdout.contains("Bunzo Compiler"));
    assert!(stdout.contains("Version"));
    assert!(stdout.contains("Target"));
    assert!(stderr.is_empty());

    // 4. -V flag -> Exit code 0, version output
    let (status, stdout, stderr) = run_bzc_binary(&["-V"]);
    assert_eq!(status, 0);
    assert!(stdout.contains("Bunzo Compiler"));
    assert!(stderr.is_empty());

    // 5. Help flag -> Exit code 0, help output
    let (status, stdout, stderr) = run_bzc_binary(&["--help"]);
    assert_eq!(status, 0);
    assert!(stdout.contains("Bunzo Compiler"));
    assert!(stdout.contains("Project Commands"));
    assert!(stdout.contains("Development Commands"));
    assert!(stdout.contains("Options"));
    assert!(stderr.is_empty());

    // 6. Running a nonexistent file -> Exit code 1
    let (status, stdout, stderr) = run_bzc_binary(&["run", "nonexistent_file.bz"]);
    assert_eq!(status, 1);
    assert!(stdout.is_empty());
    assert!(stderr.contains("BZ0001"));

    // 7. Successful execution -> Exit code 0
    let dir = std::env::temp_dir();
    let file_path = dir.join("bzc_test_binary_success.bz");
    std::fs::write(&file_path, "let x = 42").expect("failed to write temp file");

    let (status, _stdout, stderr) = run_bzc_binary(&["run", file_path.to_str().unwrap()]);
    assert_eq!(status, 0);
    assert!(stderr.is_empty());

    let _ = std::fs::remove_file(&file_path);
}
