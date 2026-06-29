use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct ReleaseMetadata {
    version: String,
    platform: String,
    architecture: String,
    artifact_name: String,
    release_channel: String,
}

#[derive(Serialize, Deserialize)]
struct Artifact {
    name: String,
    size: u64,
    sha256: String,
    kind: String,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    channel: String,
    release_date: String,
    platform: String,
    architecture: String,
    artifacts: Vec<Artifact>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "clean" => handle_clean()?,
        "dist" => handle_dist()?,
        "validate" => handle_validate()?,
        "release" => handle_release()?,
        "merge" => {
            if args.len() < 5 {
                eprintln!("Usage: cargo xtask merge <dir1> <dir2> <out_dir>");
                std::process::exit(1);
            }
            handle_merge(&args[2], &args[3], &args[4])?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: cargo xtask <command>");
    println!("\nCommands:");
    println!("  clean       Remove release artifacts and staging files");
    println!("  dist        Build and assemble the release assets");
    println!("  validate    Run validation checks on the built package");
    println!("  release     Run E2E pipeline (test, build, pack, validate) and report");
}

fn handle_clean() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cleaning release files...");
    let paths_to_remove = ["release", "target/release-artifacts"];
    for path in &paths_to_remove {
        if Path::new(path).exists() {
            println!("Removing {}...", path);
            fs::remove_dir_all(path)?;
        }
    }
    println!("Clean completed successfully.");
    Ok(())
}

fn get_version() -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("compiler/Cargo.toml")?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                return Ok(val.trim().trim_matches('"').to_string());
            }
        }
    }
    Err("Could not find version in compiler/Cargo.toml".into())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn get_platform_info() -> (&'static str, &'static str) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform = match os {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        _ => os,
    };

    let architecture = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => arch,
    };

    (platform, architecture)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn create_zip_archive(src_dir: &Path, zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut files = Vec::new();
    collect_files_recursive(src_dir, &mut files)?;

    let src_canonical = src_dir.canonicalize()?;

    for file_path in files {
        let file_canonical = file_path.canonicalize()?;
        let relative_path = file_canonical.strip_prefix(&src_canonical)?;
        let relative_str = relative_path
            .to_str()
            .ok_or("Invalid path name")?
            .replace('\\', "/");

        zip.start_file(relative_str, options)?;
        let mut f = File::open(&file_path)?;
        io::copy(&mut f, &mut zip)?;
    }
    zip.finish()?;
    Ok(())
}

fn create_tar_gz(src_dir: &Path, tar_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(tar_path)?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(enc);
    archive.append_dir_all(".", src_dir)?;
    archive.finish()?;
    Ok(())
}

fn compute_sha256(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn extract_release_notes(
    changelog_path: &Path,
    version: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(changelog_path)?;
    let mut notes = Vec::new();
    let mut capture = false;
    for line in content.lines() {
        if line.starts_with("## [") {
            if line.contains(version) {
                capture = true;
                notes.push(line.to_string());
            } else if capture {
                break;
            }
        } else if capture {
            notes.push(line.to_string());
        }
    }
    if notes.is_empty() {
        return Err(format!(
            "Could not find release notes for version {} in CHANGELOG.md",
            version
        )
        .into());
    }
    Ok(notes.join("\n"))
}

fn handle_dist() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building and assembling release packaging...");

    let version = get_version()?;
    let (platform, arch) = get_platform_info();
    let channel = if version.contains("alpha") {
        "alpha"
    } else if version.contains("beta") {
        "beta"
    } else {
        "stable"
    };

    // Staging structure setup
    let staging_root = Path::new("release");
    let portable_dir = staging_root.join("portable");
    let installer_dir = staging_root.join("installer");
    let metadata_dir = staging_root.join("metadata");
    let checksums_dir = staging_root.join("checksums");
    let notes_dir = staging_root.join("notes");
    let manifests_dir = staging_root.join("manifests");

    fs::create_dir_all(&portable_dir)?;
    fs::create_dir_all(&installer_dir)?;
    fs::create_dir_all(&metadata_dir)?;
    fs::create_dir_all(&checksums_dir)?;
    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&manifests_dir)?;

    println!("Assembling files in release/installer/...");

    // Copy compiler binary
    let bin_name = if platform == "windows" {
        "bzc.exe"
    } else {
        "bzc"
    };
    let release_bin_path = Path::new("target/release").join(bin_name);
    if !release_bin_path.exists() {
        return Err(format!(
            "Compiler binary not found at {}. Please run: cargo build --release",
            release_bin_path.display()
        )
        .into());
    }
    fs::copy(&release_bin_path, installer_dir.join(bin_name))?;

    // Copy repo assets
    let files_to_copy = ["LICENSE", "README.md", "CHANGELOG.md"];
    for file in &files_to_copy {
        if Path::new(file).exists() {
            fs::copy(file, installer_dir.join(file))?;
        }
    }

    // Copy repo folders
    let folders_to_copy = ["examples", "stdlib", "docs", "runtime"];
    for folder in &folders_to_copy {
        if Path::new(folder).exists() {
            copy_dir_all(folder, installer_dir.join(folder))?;
        }
    }

    // Archive creation
    let archive_base_name = format!("bunzo-v{}-{}-{}", version, platform, arch);
    let archive_name = if platform == "windows" {
        format!("{}.zip", archive_base_name)
    } else {
        format!("{}.tar.gz", archive_base_name)
    };
    let archive_path = portable_dir.join(&archive_name);

    println!("Generating portable archive: {}...", archive_name);
    if platform == "windows" {
        create_zip_archive(&installer_dir, &archive_path)?;
    } else {
        create_tar_gz(&installer_dir, &archive_path)?;
    }

    // Checksum generation
    println!("Generating SHA256 checksum...");
    let checksum = compute_sha256(&archive_path)?;
    let checksum_name = format!("{}.sha256", archive_name);
    let checksum_path = checksums_dir.join(&checksum_name);
    fs::write(&checksum_path, format!("{}  {}\n", checksum, archive_name))?;

    // Metadata & Manifest generation
    println!("Generating release metadata and manifest...");
    let meta = ReleaseMetadata {
        version: version.clone(),
        platform: platform.to_string(),
        architecture: arch.to_string(),
        artifact_name: archive_name.clone(),
        release_channel: channel.to_string(),
    };
    let meta_json = serde_json::to_string_pretty(&meta)?;
    fs::write(metadata_dir.join("release-metadata.json"), meta_json)?;

    let archive_size = fs::metadata(&archive_path)?.len();
    let release_date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let manifest = Manifest {
        name: "Bunzo".to_string(),
        version: version.clone(),
        channel: channel.to_string(),
        release_date,
        platform: platform.to_string(),
        architecture: arch.to_string(),
        artifacts: vec![Artifact {
            name: archive_name.clone(),
            size: archive_size,
            sha256: checksum,
            kind: "portable".to_string(),
        }],
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(manifests_dir.join("manifest.json"), manifest_json)?;

    // Release Notes extraction
    println!("Extracting release notes from CHANGELOG.md...");
    let notes = extract_release_notes(Path::new("CHANGELOG.md"), &version)?;
    fs::write(notes_dir.join("release-notes.md"), notes)?;

    println!("Dist staging packaging assembled successfully.");
    Ok(())
}

fn handle_validate() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running packaging validation checks...");

    let version = get_version()?;
    let (platform, arch) = get_platform_info();
    let staging_root = Path::new("release");

    // 1. Verify directory structures
    let required_dirs = [
        "portable",
        "installer",
        "metadata",
        "checksums",
        "notes",
        "manifests",
    ];
    for dir in &required_dirs {
        let dir_path = staging_root.join(dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(format!(
                "Validation failed: staging directory '{}' does not exist",
                dir
            )
            .into());
        }
    }

    // 2. Verify file presence
    let archive_name = if platform == "windows" {
        format!("bunzo-v{}-{}-{}.zip", version, platform, arch)
    } else {
        format!("bunzo-v{}-{}-{}.tar.gz", version, platform, arch)
    };
    let archive_path = staging_root.join("portable").join(&archive_name);
    let checksum_path = staging_root
        .join("checksums")
        .join(format!("{}.sha256", archive_name));
    let meta_path = staging_root.join("metadata").join("release-metadata.json");
    let manifest_path = staging_root.join("manifests").join("manifest.json");
    let notes_path = staging_root.join("notes").join("release-notes.md");

    let check_files = [
        &archive_path,
        &checksum_path,
        &meta_path,
        &manifest_path,
        &notes_path,
    ];
    for f in &check_files {
        if !f.exists() {
            return Err(format!("Validation failed: file '{}' is missing", f.display()).into());
        }
    }

    // 3. Verify version consistency in metadata, manifest and docs
    let meta_content = fs::read_to_string(&meta_path)?;
    if !meta_content.contains(&version) {
        return Err("Validation failed: version mismatch in release-metadata.json".into());
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    if !manifest_content.contains(&version) {
        return Err("Validation failed: version mismatch in manifest.json".into());
    }

    let docs_path = Path::new("docs/getting_started.md");
    if docs_path.exists() {
        let docs_content = fs::read_to_string(docs_path)?;
        if !docs_content.contains(&version) {
            return Err(
                "Validation failed: docs/getting_started.md does not contain current version"
                    .into(),
            );
        }
    }

    // 4. Verify checksum matching
    let calculated_hash = compute_sha256(&archive_path)?;
    let saved_checksum_content = fs::read_to_string(&checksum_path)?;
    if !saved_checksum_content.contains(&calculated_hash) {
        return Err(format!(
            "Validation failed: checksum mismatch for {}. Calculated: {}, saved: {}",
            archive_name, calculated_hash, saved_checksum_content
        )
        .into());
    }

    // 5. Inspect archive contents
    println!("Inspecting archive contents...");
    let bin_name = if platform == "windows" {
        "bzc.exe"
    } else {
        "bzc"
    };
    let expected_entries = [bin_name, "LICENSE", "README.md", "CHANGELOG.md"];
    let expected_prefixes = ["examples/", "docs/", "stdlib/", "runtime/"];

    if platform == "windows" {
        let zip_file = File::open(&archive_path)?;
        let mut zip_archive = zip::ZipArchive::new(zip_file)?;
        let mut names = std::collections::HashSet::new();
        for i in 0..zip_archive.len() {
            let file = zip_archive.by_index(i)?;
            names.insert(file.name().to_string());
        }

        for entry in &expected_entries {
            if !names.iter().any(|n| n == entry) {
                return Err(format!(
                    "Validation failed: ZIP archive missing expected file: {}",
                    entry
                )
                .into());
            }
        }
        for prefix in &expected_prefixes {
            if !names.iter().any(|n| n.starts_with(prefix)) {
                return Err(format!(
                    "Validation failed: ZIP archive missing directory contents: {}",
                    prefix
                )
                .into());
            }
        }
    } else {
        let tar_file = File::open(&archive_path)?;
        let tar_decoded = flate2::read::GzDecoder::new(tar_file);
        let mut tar_archive = tar::Archive::new(tar_decoded);
        let mut names = std::collections::HashSet::new();
        for entry in tar_archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;
            let path_str = path.to_str().ok_or("Invalid path name")?.replace('\\', "/");
            names.insert(path_str);
        }

        for entry in &expected_entries {
            if !names.iter().any(|n| n == entry) {
                return Err(format!(
                    "Validation failed: tar.gz archive missing expected file: {}",
                    entry
                )
                .into());
            }
        }
        for prefix in &expected_prefixes {
            if !names.iter().any(|n| n.starts_with(prefix)) {
                return Err(format!(
                    "Validation failed: tar.gz archive missing directory contents: {}",
                    prefix
                )
                .into());
            }
        }
    }

    println!("Validation checks passed successfully.");
    Ok(())
}

fn run_cargo_cmd(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running: cargo {}", args.join(" "));
    let status = std::process::Command::new("cargo").args(args).status()?;
    if !status.success() {
        return Err(format!("cargo {} failed", args.join(" ")).into());
    }
    Ok(())
}

fn handle_release() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running release packaging pipeline...");

    run_cargo_cmd(&["fmt", "--check"])?;
    run_cargo_cmd(&["clippy", "--all-targets"])?;
    run_cargo_cmd(&["test"])?;
    run_cargo_cmd(&["build", "--release"])?;

    handle_dist()?;
    handle_validate()?;

    let version = get_version()?;
    let (platform, arch) = get_platform_info();
    let archive_name = if platform == "windows" {
        format!("bunzo-v{}-{}-{}.zip", version, platform, arch)
    } else {
        format!("bunzo-v{}-{}-{}.tar.gz", version, platform, arch)
    };

    println!("\n==================================\n");
    println!("BUNZO RELEASE SUMMARY\n");
    println!("Version:\n    {}", version);
    println!("\nPlatform:\n    {}-{}", platform, arch);
    println!("\nArchive:\n    ✔ {}", archive_name);
    println!("\nChecksum:\n    ✔ Generated");
    println!("\nManifest:\n    ✔ Generated");
    println!("\nRelease Notes:\n    ✔ Generated");
    println!("\nValidation:\n    ✔ Passed");
    println!("\nReady for GitHub Release");
    println!("\n==================================\n");

    Ok(())
}

fn handle_merge(dir1: &str, dir2: &str, out_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path1 = Path::new(dir1).join("release/manifests/manifest.json");
    let path2 = Path::new(dir2).join("release/manifests/manifest.json");

    let m1_content = fs::read_to_string(&path1)?;
    let m2_content = fs::read_to_string(&path2)?;

    let mut m1: Manifest = serde_json::from_str(&m1_content)?;
    let m2: Manifest = serde_json::from_str(&m2_content)?;

    m1.platform = "multi".to_string();
    m1.architecture = "multi".to_string();
    m1.artifacts.extend(m2.artifacts);

    let out_manifest_dir = Path::new(out_dir).join("release/manifests");
    fs::create_dir_all(&out_manifest_dir)?;
    let merged_json = serde_json::to_string_pretty(&m1)?;
    fs::write(out_manifest_dir.join("manifest.json"), merged_json)?;
    Ok(())
}
