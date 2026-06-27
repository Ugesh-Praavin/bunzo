pub mod manifest;
#[cfg(test)]
pub mod tests;

use manifest::Manifest;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Installs all dependencies declared in bunzo.toml.
pub fn install() -> Result<(), String> {
    let manifest = Manifest::load_or_create("bunzo.toml")?;
    println!("Resolving dependencies for {}...", manifest.name);

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    fs::create_dir_all("modules")
        .map_err(|e| format!("Failed to create modules directory: {}", e))?;

    for (name, url) in &manifest.dependencies {
        let dest = format!("modules/{}", name);
        let dest_path = Path::new(&dest);

        if dest_path.exists() {
            println!("Dependency '{}' already installed. Updating...", name);
            update_package(name, dest_path)?;
        } else {
            println!("Installing '{}' from {}...", name, url);
            clone_package(url, dest_path)?;
        }
    }

    println!("All dependencies installed successfully!");
    Ok(())
}

/// Adds a dependency to bunzo.toml and installs it.
pub fn add(name: &str, url: &str) -> Result<(), String> {
    let mut manifest = Manifest::load_or_create("bunzo.toml")?;
    manifest.dependencies.insert(name.to_string(), url.to_string());
    manifest.save("bunzo.toml")?;

    fs::create_dir_all("modules")
        .map_err(|e| format!("Failed to create modules directory: {}", e))?;

    let dest = format!("modules/{}", name);
    let dest_path = Path::new(&dest);

    if dest_path.exists() {
        fs::remove_dir_all(dest_path)
            .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
    }

    println!("Installing '{}' from {}...", name, url);
    clone_package(url, dest_path)?;

    println!("Dependency '{}' added and installed!", name);
    Ok(())
}

/// Removes a dependency from bunzo.toml and deletes its folder.
pub fn remove(name: &str) -> Result<(), String> {
    let mut manifest = Manifest::load_or_create("bunzo.toml")?;
    if manifest.dependencies.remove(name).is_none() {
        return Err(format!("Dependency '{}' not found in bunzo.toml", name));
    }
    manifest.save("bunzo.toml")?;

    let dest = format!("modules/{}", name);
    let dest_path = Path::new(&dest);
    if dest_path.exists() {
        fs::remove_dir_all(dest_path)
            .map_err(|e| format!("Failed to delete module directory: {}", e))?;
    }

    println!("Dependency '{}' removed.", name);
    Ok(())
}

/// Updates all dependencies in bunzo.toml.
pub fn update() -> Result<(), String> {
    let manifest = Manifest::load_or_create("bunzo.toml")?;
    if manifest.dependencies.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    for name in manifest.dependencies.keys() {
        let dest = format!("modules/{}", name);
        let dest_path = Path::new(&dest);
        if dest_path.exists() {
            println!("Updating '{}'...", name);
            update_package(name, dest_path)?;
        } else {
            println!("Dependency '{}' is not installed yet. Installing...", name);
            let url = manifest.dependencies.get(name).unwrap();
            clone_package(url, dest_path)?;
        }
    }

    println!("All dependencies updated!");
    Ok(())
}

fn clone_package(url: &str, dest: &Path) -> Result<(), String> {
    let status = Command::new("git")
        .args(&["clone", url, dest.to_str().unwrap()])
        .status()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if !status.success() {
        return Err(format!("git clone failed for {}", url));
    }
    Ok(())
}

fn update_package(name: &str, dest: &Path) -> Result<(), String> {
    let status = Command::new("git")
        .args(&["-C", dest.to_str().unwrap(), "pull"])
        .status()
        .map_err(|e| format!("Failed to execute git pull for {}: {}", name, e))?;

    if !status.success() {
        return Err(format!("git pull failed for {}", name));
    }
    Ok(())
}
