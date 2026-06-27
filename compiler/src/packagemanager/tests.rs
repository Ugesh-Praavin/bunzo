use super::manifest::Manifest;
use crate::source::resolve_module;
use std::fs;
use std::path::Path;

#[test]
fn test_parse_manifest() {
    let manifest_str = "
[package]
name = \"test_proj\"
version = \"1.0.0\"

[dependencies]
foo = \"https://github.com/user/foo.git\"
bar = \"https://github.com/user/bar.git\"
";
    let manifest = Manifest::parse(manifest_str).unwrap();
    assert_eq!(manifest.name, "test_proj");
    assert_eq!(manifest.dependencies.len(), 2);
    assert_eq!(
        manifest.dependencies.get("foo").unwrap(),
        "https://github.com/user/foo.git"
    );
    assert_eq!(
        manifest.dependencies.get("bar").unwrap(),
        "https://github.com/user/bar.git"
    );
}

#[test]
fn test_save_manifest() {
    let mut manifest = Manifest {
        name: "save_test".to_string(),
        dependencies: std::collections::HashMap::new(),
    };
    manifest.dependencies.insert(
        "test_dep".to_string(),
        "https://github.com/user/test_dep.git".to_string(),
    );

    let temp_file = "temp_bunzo.toml";
    manifest.save(temp_file).unwrap();

    let loaded = Manifest::load_or_create(temp_file).unwrap();
    assert_eq!(loaded.name, "save_test");
    assert_eq!(
        loaded.dependencies.get("test_dep").unwrap(),
        "https://github.com/user/test_dep.git"
    );

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_module_resolution_order() {
    // Create mock modules directory structure
    let _ = fs::create_dir_all("modules/mock_package");
    
    // 1. Mock local file
    fs::write("mock_package.bz", "print(\"local\")").unwrap();
    let (path, content) = resolve_module("mock_package", None, 1, 1).unwrap();
    assert_eq!(path, "mock_package.bz");
    assert_eq!(content, "print(\"local\")");
    let _ = fs::remove_file("mock_package.bz");

    // 2. Mock modules folder mod.bz
    fs::write("modules/mock_package/mod.bz", "print(\"module_mod\")").unwrap();
    let (path, content) = resolve_module("mock_package", None, 1, 1).unwrap();
    assert_eq!(path, "modules/mock_package/mod.bz");
    assert_eq!(content, "print(\"module_mod\")");
    let _ = fs::remove_file("modules/mock_package/mod.bz");

    // 3. Mock modules folder name.bz
    fs::write("modules/mock_package.bz", "print(\"module_name\")").unwrap();
    let (path, content) = resolve_module("mock_package", None, 1, 1).unwrap();
    assert_eq!(path, "modules/mock_package.bz");
    assert_eq!(content, "print(\"module_name\")");
    let _ = fs::remove_file("modules/mock_package.bz");

    // Cleanup
    let _ = fs::remove_dir_all("modules/mock_package");
}
