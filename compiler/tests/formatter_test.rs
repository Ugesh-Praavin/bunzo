use bzc::cli;
use std::fs;

fn make_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_cli_fmt_in_place_and_check() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("test_cli_fmt.bz");

    fs::write(&file_path, "let  x  =  42").expect("failed to write temp file");

    // 1. Run check - should fail
    let check_args = make_args(&["bzc", "fmt", file_path.to_str().unwrap(), "--check"]);
    let check_res = cli::run(&check_args);
    assert!(check_res.is_err());
    assert_eq!(check_res.unwrap_err().to_string(), "Formatting required");

    // 2. Format in-place
    let fmt_args = make_args(&["bzc", "fmt", file_path.to_str().unwrap()]);
    let fmt_res = cli::run(&fmt_args);
    assert!(fmt_res.is_ok());

    // 3. Verify content
    let formatted_content = fs::read_to_string(&file_path).expect("failed to read formatted file");
    assert_eq!(formatted_content, "let x = 42\n");

    // 4. Run check again - should succeed
    let check_res_2 = cli::run(&check_args);
    assert!(check_res_2.is_ok());

    // Clean up
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_cli_fmt_directory() {
    let temp_dir = std::env::temp_dir().join("bunzo_fmt_test_dir");
    let _ = fs::create_dir_all(&temp_dir);

    let file_a = temp_dir.join("a.bz");
    let file_b = temp_dir.join("b.bz");

    fs::write(&file_a, "let a=1").unwrap();
    fs::write(&file_b, "let b=2").unwrap();

    // Run formatting on the directory
    let fmt_args = make_args(&["bzc", "fmt", temp_dir.to_str().unwrap()]);
    let fmt_res = cli::run(&fmt_args);
    assert!(fmt_res.is_ok());

    // Verify contents
    let content_a = fs::read_to_string(&file_a).unwrap();
    let content_b = fs::read_to_string(&file_b).unwrap();

    assert_eq!(content_a, "let a = 1\n");
    assert_eq!(content_b, "let b = 2\n");

    // Run check - should succeed now
    let check_args = make_args(&["bzc", "fmt", temp_dir.to_str().unwrap(), "--check"]);
    let check_res = cli::run(&check_args);
    assert!(check_res.is_ok());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
