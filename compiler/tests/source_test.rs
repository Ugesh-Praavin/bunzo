use std::path::Path;

use bzc::source::read_source;

#[test]
fn read_existing_file() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("bzc_test_read_existing.bz");

    std::fs::write(&file_path, "Hello Bunzo").expect("failed to write temp file");

    let contents = read_source(&file_path).expect("should read successfully");
    assert_eq!(contents, "Hello Bunzo");

    // Clean up.
    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn read_missing_file() {
    let result = read_source(Path::new("this_file_does_not_exist.bz"));

    assert!(result.is_err(), "should return an error");
    let err = result.unwrap_err();
    let message = format!("{err}");
    assert!(message.contains("BZ0001"));
}
