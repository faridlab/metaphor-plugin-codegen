//! Template processor for generating modules from templates
//!
//! This module provides functionality to process template files by replacing
//! placeholders with actual values provided by the user.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;
use chrono;

/// Template context with replacement values
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub module_name: String,
    pub module_name_upper: String,
    pub module_name_lower: String,
    pub author: String,
    pub description: Option<String>,
    // Entity-specific fields
    pub entity_name: Option<String>,
    pub entity_name_pascal: Option<String>,
    pub entity_name_snake: Option<String>,
    pub entity_plural: Option<String>,
    pub with_common_fields: bool,
    // Aggregate-specific fields
    pub aggregate_name: Option<String>,
    pub aggregate_name_pascal: Option<String>,
    pub aggregate_name_snake: Option<String>,
    pub aggregate_plural: Option<String>,
    pub with_events: bool,
    pub with_repository: bool,
    pub entities: Option<Vec<String>>,
    pub value_objects: Option<Vec<String>>,
}

impl TemplateContext {
    pub fn new(name: &str, author: &str, description: Option<&str>) -> Self {
        Self {
            module_name: name.to_string(),
            module_name_upper: name.to_uppercase(),
            module_name_lower: name.to_lowercase(),
            author: author.to_string(),
            description: description.map(|d| d.to_string()),
            entity_name: None,
            entity_name_pascal: None,
            entity_name_snake: None,
            entity_plural: None,
            with_common_fields: false,
            aggregate_name: None,
            aggregate_name_pascal: None,
            aggregate_name_snake: None,
            aggregate_plural: None,
            with_events: false,
            with_repository: false,
            entities: None,
            value_objects: None,
        }
    }

    pub fn new_for_entity(module: &str, entity: &str, author: &str, with_common_fields: bool) -> Self {
        let entity_pascal = Self::to_pascal_case_string(entity);
        let entity_snake = Self::to_snake_case_string(entity);
        let entity_plural = Self::to_plural_string(&entity_snake);

        Self {
            module_name: module.to_string(),
            module_name_upper: module.to_uppercase(),
            module_name_lower: module.to_lowercase(),
            author: author.to_string(),
            description: None,
            entity_name: Some(entity.to_string()),
            entity_name_pascal: Some(entity_pascal.clone()),
            entity_name_snake: Some(entity_snake.clone()),
            entity_plural: Some(entity_plural),
            with_common_fields,
            aggregate_name: None,
            aggregate_name_pascal: None,
            aggregate_name_snake: None,
            aggregate_plural: None,
            with_events: false,
            with_repository: false,
            entities: None,
            value_objects: None,
        }
    }

    pub fn new_for_aggregate(
        module: &str,
        aggregate: &str,
        author: &str,
        with_common_fields: bool,
        with_events: bool,
        with_repository: bool,
        entities: Option<Vec<String>>,
        value_objects: Option<Vec<String>>,
    ) -> Self {
        let aggregate_pascal = Self::to_pascal_case_string(aggregate);
        let aggregate_snake = Self::to_snake_case_string(aggregate);
        let aggregate_plural = Self::to_plural_string(&aggregate_snake);

        Self {
            module_name: module.to_string(),
            module_name_upper: module.to_uppercase(),
            module_name_lower: module.to_lowercase(),
            author: author.to_string(),
            description: None,
            entity_name: Some(aggregate.to_string()), // Use aggregate as entity for templates
            entity_name_pascal: Some(aggregate_pascal.clone()),
            entity_name_snake: Some(aggregate_snake.clone()),
            entity_plural: Some(aggregate_plural.clone()),
            with_common_fields,
            aggregate_name: Some(aggregate.to_string()),
            aggregate_name_pascal: Some(aggregate_pascal),
            aggregate_name_snake: Some(aggregate_snake),
            aggregate_plural: Some(aggregate_plural),
            with_events,
            with_repository,
            entities,
            value_objects,
        }
    }

    /// Convert a string to plural form (simple English pluralization)
    pub fn to_plural_string(input: &str) -> String {
        if input.is_empty() {
            return input.to_string();
        }

        // Simple English pluralization rules
        if input.ends_with('s') || input.ends_with('x') || input.ends_with('z') ||
           input.ends_with("ch") || input.ends_with("sh") {
            format!("{}es", input)
        } else if input.ends_with('y') && input.len() > 1 {
            let base = &input[..input.len()-1];
            format!("{}ies", base)
        } else {
            format!("{}s", input)
        }
    }

    /// Convert a string to PascalCase
    pub fn to_pascal_case_string(input: &str) -> String {
        if input.is_empty() {
            return input.to_string();
        }

        // Handle kebab-case to PascalCase
        if input.contains('-') {
            input.split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        } else {
            // Handle snake_case to PascalCase
            input.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        }
    }

    /// Convert a string to snake_case
    pub fn to_snake_case_string(input: &str) -> String {
        if input.is_empty() {
            return input.to_string();
        }

        // Handle kebab-case to snake_case
        if input.contains('-') {
            input.replace('-', "_")
        } else {
            // Handle PascalCase to snake_case
            input.chars()
                .enumerate()
                .map(|(i, c)| {
                    if c.is_uppercase() && i > 0 {
                        format!("_{}", c.to_lowercase())
                    } else {
                        c.to_lowercase().to_string()
                    }
                })
                .collect()
        }
    }

    /// Convert module name to PascalCase (first letter capitalized, rest as is)
    fn to_pascal_case(&self) -> String {
        let name = &self.module_name;
        if name.is_empty() {
            return name.clone();
        }

        // Handle kebab-case to PascalCase
        if name.contains('-') {
            name.split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        } else {
            // Handle snake_case to PascalCase
            name.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        }
    }

    /// Convert module name to snake_case (all lowercase with underscores)
    fn to_snake_case(&self) -> String {
        let name = &self.module_name;
        if name.is_empty() {
            return name.clone();
        }

        // Handle kebab-case to snake_case
        if name.contains('-') {
            name.replace('-', "_")
        } else {
            // Handle PascalCase to snake_case
            name.chars()
                .enumerate()
                .map(|(i, c)| {
                    if c.is_uppercase() && i > 0 {
                        format!("_{}", c.to_lowercase())
                    } else {
                        c.to_lowercase().to_string()
                    }
                })
                .collect()
        }
    }

    /// Get all placeholder mappings for template replacement
    fn get_replacements(&self) -> HashMap<String, String> {
        let mut replacements = HashMap::new();
        let pascal_case = self.to_pascal_case();
        let snake_case = self.to_snake_case();

        // Note: We no longer replace bare "metaphor" or "Metaphor" to avoid
        // accidentally replacing dependency names like "metaphor-core".
        // Use explicit {{MODULE_NAME}} or {{MODULE_NAME_PASCAL}} placeholders instead.
        replacements.insert("{{MODULE_NAME}}".to_string(), self.module_name.clone());
        replacements.insert("{{MODULE_NAME_PASCAL}}".to_string(), pascal_case.clone());
        replacements.insert("{{PascalCaseModuleName}}".to_string(), pascal_case.clone());
        replacements.insert("{{MODULE_NAME_SNAKE}}".to_string(), snake_case.clone());
        replacements.insert("{{MODULE_NAME_UPPER}}".to_string(), self.module_name_upper.clone());
        replacements.insert("{{MODULE_NAME_LOWER}}".to_string(), self.module_name_lower.clone());
        replacements.insert("{{AUTHOR}}".to_string(), self.author.clone());

        if let Some(desc) = &self.description {
            replacements.insert("{{DESCRIPTION}}".to_string(), desc.clone());
        }

        // Entity-specific replacements
        if let Some(entity_name) = &self.entity_name {
            replacements.insert("{{ENTITY_NAME}}".to_string(), entity_name.clone());
        }
        if let Some(entity_pascal) = &self.entity_name_pascal {
            replacements.insert("{{PascalCaseEntity}}".to_string(), entity_pascal.clone());
        }
        if let Some(entity_snake) = &self.entity_name_snake {
            replacements.insert("{{ENTITY_NAME_SNAKE}}".to_string(), entity_snake.clone());
            replacements.insert("{{entity_name_snake}}".to_string(), entity_snake.clone()); // Add lowercase version for CRUD templates
        }
        if let Some(entity_plural) = &self.entity_plural {
            replacements.insert("{{ENTITY_NAME_PLURAL}}".to_string(), entity_plural.clone());
            replacements.insert("{{entity_name_plural}}".to_string(), entity_plural.clone()); // Add lowercase version for CRUD templates
        }

        // Add current timestamp
        replacements.insert("{{CURRENT_TIMESTAMP}}".to_string(), chrono::Utc::now().to_rfc3339());

        replacements
    }
}

/// Process a template file and write the result to output path
pub fn process_template_file(
    template_path: &Path,
    output_path: &Path,
    context: &TemplateContext,
) -> Result<()> {
    let template_content = fs::read_to_string(template_path)?;
    let processed_content = replace_placeholders(&template_content, context);

    // Create parent directories if they don't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, processed_content)?;
    Ok(())
}

/// Replace all placeholders in template content with context values
pub fn replace_placeholders(template: &str, context: &TemplateContext) -> String {
    let replacements = context.get_replacements();
    let mut result = template.to_string();

    for (placeholder, value) in replacements {
        result = result.replace(&placeholder, &value);
    }

    // Handle conditional replacements
    if context.with_common_fields {
        let timestamp_fields = r#"  // Timestamps
  google.protobuf.Timestamp created_at = 2;
  google.protobuf.Timestamp updated_at = 3;
  optional google.protobuf.Timestamp deleted_at = 4;"#;
        result = result.replace("TIMESTAMP_FIELDS_PLACEHOLDER", timestamp_fields);

        // For Rust templates - common fields in struct
        let rust_common_fields = r#"    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Soft deletion timestamp
    pub deleted_at: Option<DateTime<Utc>,"#;
        result = result.replace("COMMON_FIELDS_PLACEHOLDER", rust_common_fields);
    } else {
        result = result.replace("TIMESTAMP_FIELDS_PLACEHOLDER", "");
        result = result.replace("COMMON_FIELDS_PLACEHOLDER", "");
    }

    result
}

/// Copy template directory to target path while processing all files
pub fn copy_and_process_template_dir(
    template_dir: &Path,
    target_dir: &Path,
    context: &TemplateContext,
) -> Result<()> {
    // Create target directory
    fs::create_dir_all(target_dir)?;

    // Walk through template directory using simple fs operations
    copy_directory_recursive(template_dir, target_dir, context)?;

    Ok(())
}

/// Recursively copy directory and process templates
fn copy_directory_recursive(
    source_dir: &Path,
    target_dir: &Path,
    context: &TemplateContext,
) -> Result<()> {
    // Create target directory if it doesn't exist
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    // Read directory entries
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let target_path = target_dir.join(file_name);

        if source_path.is_dir() {
            // Recursively copy subdirectory
            copy_directory_recursive(&source_path, &target_path, context)?;
        } else {
            // Skip build directories and non-code files
            if file_name.starts_with("target") || file_name.starts_with("build") || file_name.starts_with("Cargo.lock") {
                continue;
            }

            // Copy file as-is but replace placeholders in filename
            // Note: We use explicit {{PLACEHOLDERS}} only, not bare "metaphor"/"Metaphor"
            let pascal_case = context.to_pascal_case();
            let snake_case = context.to_snake_case();
            let mut output_file_name = file_name
                .replace("{{MODULE_NAME}}", &context.module_name)
                .replace("{{MODULE_NAME_PASCAL}}", &pascal_case)
                .replace("{{PascalCaseModuleName}}", &pascal_case)
                .replace("{{MODULE_NAME_SNAKE}}", &snake_case)
                .replace("{{MODULE_NAME_UPPER}}", &context.module_name_upper)
                .replace("{{MODULE_NAME_LOWER}}", &context.module_name_lower);

            // Add entity-specific placeholder replacements for CRUD templates
            if let (Some(entity_name_snake), Some(entity_plural), Some(entity_pascal)) = (
                &context.entity_name_snake,
                &context.entity_plural,
                &context.entity_name_pascal
            ) {
                output_file_name = output_file_name
                    .replace("{{entity_name_snake}}", entity_name_snake)
                    .replace("{{entity_name_plural}}", entity_plural)
                    .replace("{{PascalCaseEntity}}", entity_pascal)
                    .replace("{{ENTITY_NAME_SNAKE}}", entity_name_snake)
                    .replace("{{ENTITY_NAME_PLURAL}}", entity_plural);
            }
            let output_path = target_dir.join(output_file_name);

            // Process file content for placeholders (but only for source files)
            if file_name.ends_with(".rs") || file_name.ends_with(".proto") || file_name.ends_with(".toml") || file_name.ends_with(".yaml") || file_name.ends_with(".md") {
                process_template_file(&source_path, &output_path, context)?;
            } else {
                fs::copy(&source_path, &output_path)?;
            }
        }
    }

    Ok(())
}

/// Get template directory path for module templates
pub fn get_module_template_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_default();
    current_dir
        .join("crates")
        .join("metaphor-cli")
        .join("src")
        .join("templates")
        .join("module")
}

/// Get template directory path for entity templates
pub fn get_entity_template_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_default();
    current_dir
        .join("crates")
        .join("metaphor-cli")
        .join("src")
        .join("templates")
        .join("entity")
}

/// Get template directory path for CRUD templates
pub fn get_crud_template_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_default();
    current_dir
        .join("crates")
        .join("metaphor-cli")
        .join("src")
        .join("templates")
        .join("crud")
}

/// Get template directory path for aggregate templates
pub fn get_aggregate_template_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_default();
    current_dir
        .join("crates")
        .join("metaphor-cli")
        .join("src")
        .join("templates")
        .join("aggregate")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_replacement() {
        let context = TemplateContext::new("payments", "John Doe", Some("Payment processing module"));
        let template = "Hello {{AUTHOR}}, welcome to {{MODULE_NAME}} bounded context!";

        let result = replace_placeholders(template, &context);

        assert_eq!(
            result,
            "Hello John Doe, welcome to payments bounded context!"
        );
    }

    #[test]
    fn test_case_conversion() {
        let context = TemplateContext::new("PaymentGateway", "Jane Smith", None);

        assert_eq!(context.module_name, "PaymentGateway");
        assert_eq!(context.module_name_upper, "PAYMENTGATEWAY");
        assert_eq!(context.module_name_lower, "paymentgateway");
    }
}