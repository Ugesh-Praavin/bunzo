# Release Engineering – Packaging & Distribution

This document outlines the release, packaging, and distribution processes of the Bunzo compiler.

## Release Process Overview

Bunzo releases are fully automated via `cargo xtask` and GitHub Actions. All build assets are derived from a single source of truth version located in `compiler/Cargo.toml`.

To compile, test, package, and validate a release candidate locally, run:
```bash
cargo xtask release
```

---

## Command Reference

The `cargo xtask` runner exposes the following subcommands:

### 1. `cargo xtask clean`
Removes the `release/` directory and any temporary build assets.

### 2. `cargo xtask dist`
Compiles `bzc` in release mode, gathers all documentation, license, examples, standard libraries, and runtime components, and creates portable packages and manifests under `release/`.

### 3. `cargo xtask validate`
Performs validation checks:
- Checks version consistency across `Cargo.toml`, metadata files, and user documentation.
- Computes SHA256 checksums and validates they match the `.sha256` files.
- Inspects the generated `.zip` or `.tar.gz` archive to verify that all necessary binaries, headers, standard libraries, and examples are present.

### 4. `cargo xtask release`
Runs `cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo build --release`, followed by `cargo xtask dist` and `cargo xtask validate`. It prints a release report at completion.

### 5. `cargo xtask merge <dir1> <dir2> <out_dir>`
Merges platform-specific `manifest.json` files from `dir1` and `dir2` into a single multi-architecture manifest. Used by GitHub Actions.

---

## Release Artifacts Structure

All generated release assets are outputted to the `/release/` directory (which is ignored by Git):

```text
release/
├── portable/
│   └── bunzo-v0.8.0-alpha-windows-x64.zip          # Portable zipped archive
├── installer/
│   ├── bzc.exe                                     # Unzipped release binary
│   ├── LICENSE                                     # Project license
│   ├── README.md
│   ├── CHANGELOG.md
│   ├── stdlib/                                     # Standard library
│   ├── examples/                                   # Project examples
│   ├── docs/                                       # Markdowns
│   └── runtime/                                    # GC & runtime headers/source
├── metadata/
│   └── release-metadata.json                       # Short release metadata
├── manifests/
│   └── manifest.json                               # Detailed manifest consumed by website/winget
├── checksums/
│   └── bunzo-v0.8.0-alpha-windows-x64.zip.sha256   # SHA256 checksum file
└── notes/
    └── release-notes.md                            # Release notes extracted from CHANGELOG
```

---

## Checksum Verification

Users can manually verify download integrity using the generated checksum:

### Windows (PowerShell)
```powershell
# Get the calculated hash
Get-FileHash .\portable\bunzo-v0.8.0-alpha-windows-x64.zip -Algorithm SHA256

# Or verify against the sha256 file contents
Get-Content .\checksums\bunzo-v0.8.0-alpha-windows-x64.zip.sha256
```

### Linux / macOS
```bash
sha256sum -c checksums/bunzo-v0.8.0-alpha-linux-x64.tar.gz.sha256
```

---

## GitHub Releases CI/CD Pipeline

When a version tag (e.g. `v0.8.0-alpha`) is pushed to GitHub:

1. **Build & Package**: Separate runner jobs build the project on Linux (`ubuntu-latest`) and Windows (`windows-latest`).
2. **Execution of `cargo xtask release`**: Each job runs the E2E release pipeline.
3. **Download & Consolidation**: A final orchestration job pulls both Linux and Windows artifacts.
4. **Manifest Merging**: It runs `cargo xtask merge` to generate a unified multi-architecture `manifest.json`.
5. **Draft Release**: The assets are uploaded to GitHub Releases as a **draft**. The release remains invisible to the public until a maintainer reviews the draft and publishes it manually.

---

## Versioning Policy

Bunzo uses **Semantic Versioning (SemVer)**:
- **Major**: Breaking changes in syntax, semantics, or standard library behaviors.
- **Minor**: Backward-compatible new features, built-in functions, or tool improvements.
- **Patch**: Bug fixes, minor linter adjustments, or performance enhancements.
- **Prerelease**: Suffixed with `-alpha` or `-beta` (e.g., `0.8.0-alpha`).
