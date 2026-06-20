use std::path::PathBuf;

use bzc::diagnostics::CompilerError;

#[test]
fn display_file_not_found() {
    let err = CompilerError::FileNotFound(PathBuf::from("missing.bz"));
    let message = format!("{err}");

    assert!(message.contains("BZ0001"), "should contain error code");
    assert!(message.contains("missing.bz"), "should contain file path");
}

#[test]
fn display_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err = CompilerError::Io(io_err);
    let message = format!("{err}");

    assert!(message.contains("BZ0002"), "should contain error code");
    assert!(message.contains("access denied"), "should contain cause");
}
