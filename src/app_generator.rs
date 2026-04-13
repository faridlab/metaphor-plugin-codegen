//! Apps Generator
//!
//! Template-based application generator for creating new Metaphor Framework apps
//! with Clean Architecture structure and customizable configurations.

use anyhow::Result;
use anyhow::{anyhow, Context};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Application generator
pub struct AppGenerator {
    handlebars: Handlebars<'static>,
    template_dir: PathBuf,
}

impl AppGenerator {
    /// Create a new app generator
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register custom helpers
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));
        handlebars.register_helper("kebab_case", Box::new(kebab_case_helper));
        handlebars.register_helper("camel_case", Box::new(camel_case_helper));
        handlebars.register_helper("upper_case", Box::new(upper_case_helper));
        handlebars.register_helper("title_case", Box::new(title_case_helper));

        let template_dir = {
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;

        // Try multiple possible paths
        let possible_paths = vec![
            current_dir.join("crates/metaphor-cli/src/templates/app"),
            current_dir.join("templates/app"),
            current_dir.join("src/templates/app"),
        ];

        match possible_paths.into_iter().find(|path| path.exists()) {
            Some(path) => path,
            None => {
                return Err(anyhow!("Template directory not found. Current directory: {}\nSearched paths:\n- {}/crates/metaphor-cli/src/templates/app\n- {}/templates/app\n- {}/src/templates/app",
                    current_dir.display(), current_dir.display(), current_dir.display(), current_dir.display()));
            }
        }
    };

        Ok(Self {
            handlebars,
            template_dir,
        })
    }

    /// Generate a new application
    pub async fn generate_app(&self, config: &AppGeneratorConfig, output_dir: &Path) -> Result<()> {
        println!("🚀 Generating Metaphor Framework app: {}", config.app_name);

        // Create output directory
        let app_output_dir = output_dir.join(&config.app_name);
        fs::create_dir_all(&app_output_dir)
            .with_context(|| format!("Failed to create output directory: {}", app_output_dir.display()))?;

        // Prepare template variables
        let variables = self.prepare_variables(config)?;

        // Copy and process template files
        self.copy_template_files(&self.template_dir, &app_output_dir, &variables)
            .await?;

        // Update workspace Cargo.toml
        let workspace_root = self.find_workspace_root(output_dir)?;
        self.update_workspace_cargo_toml(&workspace_root, config).await?;

        println!("✅ Successfully generated app: {}", config.app_name);
        println!("📁 Location: {}", app_output_dir.display());
        println!("🔧 Next steps:");
        println!("   cd {}", config.app_name);
        println!("   cargo run");

        Ok(())
    }

    /// Prepare template variables from config
    fn prepare_variables(&self, config: &AppGeneratorConfig) -> Result<HashMap<String, String>> {
        let mut variables = HashMap::new();

        // Basic app information
        variables.insert("APP_NAME".to_string(), config.app_name.clone());
        variables.insert("APP_NAME_PASCAL".to_string(), to_pascal_case(&config.app_name));
        variables.insert("APP_NAME_SNAKE".to_string(), to_snake_case(&config.app_name));
        variables.insert("APP_NAME_KEBAB".to_string(), to_kebab_case(&config.app_name));
        variables.insert("APP_NAME_CAMEL".to_string(), to_camel_case(&config.app_name));
        variables.insert("APP_PORT".to_string(), config.app_port.to_string());
        variables.insert("APP_DESCRIPTION".to_string(), config.app_description.clone());
        variables.insert("APP_TYPE".to_string(), config.app_type.clone());
        variables.insert("DATABASE_TYPE".to_string(), config.database_type.clone());
        variables.insert("DATABASE_NAME".to_string(), config.database_name.clone());
        variables.insert("AUTH_ENABLED".to_string(), config.auth_enabled.to_string());
        variables.insert("HEALTH_ENABLED".to_string(), config.health_enabled.to_string());
        variables.insert("METRICS_ENABLED".to_string(), config.metrics_enabled.to_string());
        variables.insert("AUTHOR_NAME".to_string(), config.author_name.clone());
        variables.insert("AUTHOR_EMAIL".to_string(), config.author_email.clone());
        variables.insert("CREATION_YEAR".to_string(), config.creation_year.to_string());

        // Current timestamp
        let now = chrono::Utc::now();
        variables.insert("CREATION_DATE".to_string(), now.format("%Y-%m-%d").to_string());
        variables.insert("CREATION_DATETIME".to_string(), now.to_rfc3339());

        // App type specific configurations
        match config.app_type.as_str() {
            "auth" => {
                variables.insert("INCLUDE_AUTH".to_string(), "true".to_string());
                variables.insert("INCLUDE_SESSIONS".to_string(), "true".to_string());
            }
            "worker" => {
                variables.insert("INCLUDE_WORKER".to_string(), "true".to_string());
                variables.insert("INCLUDE_JOB_QUEUE".to_string(), "true".to_string());
            }
            "scheduler" => {
                variables.insert("INCLUDE_SCHEDULER".to_string(), "true".to_string());
                variables.insert("INCLUDE_CRON".to_string(), "true".to_string());
            }
            _ => {
                variables.insert("INCLUDE_AUTH".to_string(), "false".to_string());
                variables.insert("INCLUDE_WORKER".to_string(), "false".to_string());
                variables.insert("INCLUDE_SCHEDULER".to_string(), "false".to_string());
            }
        }

        Ok(variables)
    }

    /// Copy and process template files
    async fn copy_template_files(
        &self,
        template_dir: &Path,
        output_dir: &Path,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        self.copy_files_recursive(template_dir, output_dir, variables)
            .await
    }

    /// Recursively copy and process files
    async fn copy_files_recursive(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        let mut dirs_to_process = Vec::new();
        dirs_to_process.push((src_dir.to_path_buf(), dst_dir.to_path_buf()));

        while let Some((current_src_dir, current_dst_dir)) = dirs_to_process.pop() {
            let entries = fs::read_dir(&current_src_dir)
                .with_context(|| format!("Failed to read directory: {}", current_src_dir.display()))?;

            for entry in entries {
                let entry = entry
                    .with_context(|| format!("Failed to read directory entry"))?;
                let src_path = entry.path();
                let _file_name = entry.file_name();
                let relative_path = src_path.strip_prefix(&current_src_dir)
                    .with_context(|| format!("Failed to get relative path for: {}", src_path.display()))?;

                // Skip certain directories and files
                if self.should_skip_file(&relative_path) {
                    continue;
                }

                let dst_path = current_dst_dir.join(relative_path);

                if src_path.is_dir() {
                    // Create directory
                    fs::create_dir_all(&dst_path)
                        .with_context(|| format!("Failed to create directory: {}", dst_path.display()))?;

                    // Add subdirectory to processing stack
                    dirs_to_process.push((src_path, dst_path));
                } else {
                    // Process file
                    self.process_template_file(&src_path, &dst_path, variables)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Process a single template file
    async fn process_template_file(
        &self,
        src_path: &Path,
        dst_path: &Path,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
        }

        // Read file content
        let content = fs::read_to_string(src_path)
            .with_context(|| format!("Failed to read file: {}", src_path.display()))?;

        // Check if file should be processed as template
        if self.is_template_file(src_path) {
            // Process as handlebars template
            let processed_content = self.handlebars
                .render_template(&content, variables)
                .with_context(|| format!("Failed to process template: {}", src_path.display()))?;

            fs::write(dst_path, processed_content)
                .with_context(|| format!("Failed to write file: {}", dst_path.display()))?;
        } else {
            // Copy file as-is
            fs::copy(src_path, dst_path)
                .with_context(|| format!("Failed to copy file: {} -> {}", src_path.display(), dst_path.display()))?;
        }

        Ok(())
    }

    /// Find the workspace root by searching for Cargo.toml with [workspace] section
    fn find_workspace_root(&self, start_dir: &Path) -> Result<PathBuf> {
        let mut current_dir = start_dir.to_path_buf();

        loop {
            let cargo_toml_path = current_dir.join("Cargo.toml");

            if cargo_toml_path.exists() {
                let content = fs::read_to_string(&cargo_toml_path)
                    .with_context(|| format!("Failed to read: {}", cargo_toml_path.display()))?;

                // Check if this is a workspace Cargo.toml
                if content.contains("[workspace]") || content.contains("[package]") {
                    return Ok(current_dir);
                }
            }

            // Move up to parent directory
            match current_dir.parent() {
                Some(parent) => {
                    current_dir = parent.to_path_buf();
                    if current_dir.as_os_str() == "/" {
                        break;
                    }
                }
                None => break,
            }
        }

        Err(anyhow!("Workspace Cargo.toml not found starting from: {}", start_dir.display()))
    }

    /// Update workspace Cargo.toml to include new app
    async fn update_workspace_cargo_toml(
        &self,
        workspace_dir: &Path,
        config: &AppGeneratorConfig,
    ) -> Result<()> {
        let workspace_cargo_toml = workspace_dir.join("Cargo.toml");

        if !workspace_cargo_toml.exists() {
            return Err(anyhow!("Workspace Cargo.toml not found: {}", workspace_cargo_toml.display()));
        }

        let mut content = fs::read_to_string(&workspace_cargo_toml)
            .with_context(|| "Failed to read workspace Cargo.toml")?;

        // Add workspace member
        let workspace_member_line = format!(r#""{}"#, config.app_name);

        if !content.contains(&workspace_member_line) {
            // Find the workspace members section and add the new member
            if let Some(start) = content.find("members = [") {
                if let Some(end) = content[start..].find(']') {
                    let members_section = &content[start..start + end + 1];
                    let new_members_section = members_section.replace(']', &format!("\n    {},\n]", workspace_member_line));
                    content.replace_range(start..start + end + 1, &new_members_section);
                }
            }

            fs::write(&workspace_cargo_toml, content)
                .with_context(|| "Failed to update workspace Cargo.toml")?;
        }

        Ok(())
    }

    /// Check if a file should be skipped during copying
    fn should_skip_file(&self, relative_path: &Path) -> bool {
        let path_str = relative_path.to_string_lossy();

        // Skip directories and files that shouldn't be in templates
        path_str.contains("target/") ||
        path_str.contains(".git/") ||
        path_str.contains("node_modules/") ||
        path_str.ends_with(".log") ||
        path_str.ends_with(".DS_Store") ||
        path_str.ends_with(".env.local")
    }

    /// Check if a file should be processed as a template
    fn is_template_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => true,     // Rust files
            Some("toml") => true,   // Cargo.toml files
            Some("yml") => true,    // YAML files
            Some("yaml") => true,   // YAML files
            Some("json") => true,   // JSON files
            Some("md") => true,     // Markdown files
            _ => false,
        }
    }
}

impl Default for AppGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create AppGenerator")
    }
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

// Handlebars helpers

fn pascal_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_pascal_case(value))?;
    }
    Ok(())
}

fn snake_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_snake_case(value))?;
    }
    Ok(())
}

fn kebab_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_kebab_case(value))?;
    }
    Ok(())
}

fn camel_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_camel_case(value))?;
    }
    Ok(())
}

fn upper_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_upper_case(value))?;
    }
    Ok(())
}

fn title_case_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&to_title_case(value))?;
    }
    Ok(())
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