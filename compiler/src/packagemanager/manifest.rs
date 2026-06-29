use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents the parsed manifest file.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Loads a manifest from disk, creating a default one if it doesn't exist.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.exists() {
            let default_content = "\
[package]
name = \"unnamed_project\"
version = \"0.1.0\"

[dependencies]
";
            fs::write(path, default_content)
                .map_err(|e| format!("Failed to create bunzo.toml: {}", e))?;
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read bunzo.toml: {}", e))?;

        Self::parse(&content)
    }

    /// Parses a raw string into a Manifest.
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut name = "unnamed_project".to_string();
        let mut dependencies = HashMap::new();
        let mut in_dependencies_section = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let section = line[1..line.len() - 1].trim();
                if section == "dependencies" {
                    in_dependencies_section = true;
                } else {
                    in_dependencies_section = false;
                }
                continue;
            }

            if let Some(equal_idx) = line.find('=') {
                let key = line[..equal_idx].trim().to_string();
                let value = line[equal_idx + 1..].trim();
                let value_trimmed = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value[1..value.len() - 1].to_string()
                } else {
                    value.to_string()
                };

                if in_dependencies_section {
                    dependencies.insert(key, value_trimmed);
                } else if key == "name" {
                    name = value_trimmed;
                }
            }
        }

        Ok(Manifest { name, dependencies })
    }

    /// Saves the manifest back to disk.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut content = String::new();
        content.push_str("[package]\n");
        content.push_str(&format!("name = \"{}\"\n\n", self.name));
        content.push_str("[dependencies]\n");

        let mut sorted_deps: Vec<(&String, &String)> = self.dependencies.iter().collect();
        sorted_deps.sort_by(|a, b| a.0.cmp(b.0));

        for (key, val) in sorted_deps {
            content.push_str(&format!("{} = \"{}\"\n", key, val));
        }

        fs::write(path, content).map_err(|e| format!("Failed to write bunzo.toml: {}", e))
    }
}
