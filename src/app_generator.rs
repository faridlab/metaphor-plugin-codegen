//! Apps Generator
//!
//! Template-based application generator for creating new Metaphor Framework apps
//! with Clean Architecture structure and customizable configurations.

use anyhow::Result;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// The canonical application skeleton. `metaphor apps generate` clones this and renames it —
/// backbone-application is the single source of truth for runnable app/service structure.
/// Always cloned fresh from GitHub so every workspace scaffolds from the latest skeleton.
const APP_SKELETON_REPO: &str = "https://github.com/faridlab/backbone-application";

/// Literal package/binary name baked into the skeleton's load-bearing files (kebab form).
const SKELETON_NAME_KEBAB: &str = "backbone-app";
/// Literal package name in snake form (RUST_LOG targets, database name, etc.).
const SKELETON_NAME_SNAKE: &str = "backbone_app";

/// Application generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppGeneratorConfig {
    /// Application name (kebab-case)
    pub app_name: String,
    /// Application port
    pub app_port: u16,
    /// Application description
    pub app_description: String,
    /// Application type
    pub app_type: String,
    /// Database type
    pub database_type: String,
    /// Database name
    pub database_name: String,
    /// Enable authentication
    pub auth_enabled: bool,
    /// Enable health checks
    pub health_enabled: bool,
    /// Enable metrics
    pub metrics_enabled: bool,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Creation year
    pub creation_year: u32,
}

impl Default for AppGeneratorConfig {
    fn default() -> Self {
        Self {
            app_name: "my-service".to_string(),
            app_port: 3000,
            app_description: "Metaphor Framework Application".to_string(),
            app_type: "api".to_string(),
            database_type: "postgresql".to_string(),
            database_name: "my_service_db".to_string(),
            auth_enabled: false,
            health_enabled: true,
            metrics_enabled: false,
            author_name: "Metaphor Team".to_string(),
            author_email: "team@metaphor.dev".to_string(),
            creation_year: 2024,
        }
    }
}

/// Application generator.
///
/// Scaffolds a new app/service by cloning the canonical [`APP_SKELETON_REPO`] skeleton
/// fresh from GitHub and renaming its baked-in package/binary name to the requested app
/// name. No local template directory is involved — the GitHub repo is the single source
/// of truth, so every workspace scaffolds from the latest skeleton.
pub struct AppGenerator {
    /// Skeleton repository URL cloned for each generated app.
    skeleton_repo: String,
}

impl AppGenerator {
    /// Create a new app generator.
    pub fn new() -> Result<Self> {
        Ok(Self {
            skeleton_repo: APP_SKELETON_REPO.to_string(),
        })
    }

    /// Generate a new application by cloning the skeleton and renaming it.
    pub async fn generate_app(&self, config: &AppGeneratorConfig, output_dir: &Path) -> Result<()> {
        println!("🚀 Generating Metaphor Framework app: {}", config.app_name);

        let app_output_dir = output_dir.join(&config.app_name);
        if app_output_dir.exists() {
            return Err(anyhow!(
                "'{}' already exists — refusing to overwrite.",
                app_output_dir.display()
            ));
        }

        // Ensure the parent output directory exists (git clone creates the leaf itself).
        fs::create_dir_all(output_dir).with_context(|| {
            format!("Failed to create output directory: {}", output_dir.display())
        })?;

        // Scaffold by cloning the canonical skeleton project from GitHub (shallow).
        println!("📥 Cloning skeleton from {}", self.skeleton_repo);
        let status = Command::new("git")
            .args(["clone", "--depth", "1", &self.skeleton_repo])
            .arg(&app_output_dir)
            .status()
            .context("failed to run `git clone` (is git installed and on PATH?)")?;
        if !status.success() {
            return Err(anyhow!(
                "git clone of {} failed — ensure the repo is reachable and you have access.",
                self.skeleton_repo
            ));
        }

        // Detach from the skeleton repo: this is a fresh app, not a fork. Drop the lockfile
        // too so dependencies resolve fresh under the new package name.
        let _ = fs::remove_dir_all(app_output_dir.join(".git"));
        let _ = fs::remove_file(app_output_dir.join("Cargo.lock"));

        // Rename the skeleton's baked-in package/binary name to the requested app name.
        // The skeleton uses the literal `backbone-app` (kebab) and `backbone_app` (snake)
        // across load-bearing files (Cargo.toml, src/main.rs, Dockerfiles, config, deployment).
        let app_snake = to_snake_case(&config.app_name);
        let kebab_changed =
            replace_token_in_tree(&app_output_dir, SKELETON_NAME_KEBAB, &config.app_name)
                .context("renaming skeleton package name (kebab) into the new app")?;
        let snake_changed = replace_token_in_tree(&app_output_dir, SKELETON_NAME_SNAKE, &app_snake)
            .context("renaming skeleton package name (snake) into the new app")?;
        println!(
            "🔖 Renamed skeleton package to '{}' across {} file(s)",
            config.app_name,
            kebab_changed + snake_changed
        );

        println!("✅ Successfully generated app: {}", config.app_name);
        println!("📁 Location: {}", app_output_dir.display());
        println!("🔧 Next steps:");
        println!("   1. Register in metaphor.yaml (name: {} / type: backend-service)", config.app_name);
        println!("   cd {}", config.app_name);
        println!("   cargo run");

        Ok(())
    }
}

impl Default for AppGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create AppGenerator")
    }
}

/// Recursively replace `token` with `value` in every UTF-8 text file under `root`.
/// Skips `.git` and `target`, and any file that isn't valid UTF-8 (e.g. binaries). Returns
/// the number of files changed. Used to stamp the new app name into the cloned skeleton.
fn replace_token_in_tree(root: &Path, token: &str, value: &str) -> Result<usize> {
    let mut changed = 0;
    for entry in fs::read_dir(root).with_context(|| format!("reading dir {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = entry.file_name();
            if name == ".git" || name == "target" {
                continue;
            }
            changed += replace_token_in_tree(&path, token, value)?;
        } else if file_type.is_file() {
            // Only rewrite files that are valid UTF-8 and actually contain the token.
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains(token) {
                    fs::write(&path, content.replace(token, value))
                        .with_context(|| format!("writing {}", path.display()))?;
                    changed += 1;
                }
            }
        }
    }
    Ok(changed)
}

// Helper functions for string transformations

/// Convert string to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert string to snake_case
pub fn to_snake_case(s: &str) -> String {
    s.replace('-', "_")
}

/// Convert string to kebab-case
pub fn to_kebab_case(s: &str) -> String {
    s.replace('_', "-")
}

/// Convert string to camelCase
pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// Convert string to UPPER_CASE
pub fn to_upper_case(s: &str) -> String {
    s.replace('-', "_").to_uppercase()
}

/// Convert string to Title Case
pub fn to_title_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_transformations() {
        assert_eq!(to_pascal_case("my-service"), "MyService");
        assert_eq!(to_snake_case("my-service"), "my_service");
        assert_eq!(to_kebab_case("my_service"), "my-service");
        assert_eq!(to_camel_case("my-service"), "myService");
        assert_eq!(to_upper_case("my-service"), "MY_SERVICE");
        assert_eq!(to_title_case("my-service"), "My Service");
    }

    #[test]
    fn test_app_generator_config_default() {
        let config = AppGeneratorConfig::default();
        assert_eq!(config.app_name, "my-service");
        assert_eq!(config.app_port, 3000);
        assert_eq!(config.app_type, "api");
        assert_eq!(config.database_type, "postgresql");
        assert!(!config.auth_enabled);
        assert!(config.health_enabled);
    }
}