//! Module management commands (Priority 1.1 from BACKFRAME_TODO.md)
//!
//! Implements:
//! - `metaphor module:create <name>` - Generate new module structure using templates
//! - `metaphor module:list` - List all available modules
//! - `metaphor module:info <name>` - Show module details
//! - `metaphor module:enable <name>` - Enable module
//! - `metaphor module:disable <name>` - Disable module

use anyhow::Result;
use anyhow::Context;
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::templates::template_processor::{TemplateContext, copy_and_process_template_dir, get_module_template_dir};

/// Module action enum for command handling
#[derive(Debug, Clone)]
pub enum ModuleAction {
    Create {
        name: String,
        author: String,
        description: Option<String>,
    },
    List,
    Info {
        name: String,
    },
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    Install {
        package: String,
        production: bool,
        version: Option<String>,
        git: bool,
    },
}

/// Module management command handler
pub async fn handle_command(action: &ModuleAction) -> Result<()> {
    match action {
        ModuleAction::Create { name, author, description } => {
            create_module(name, author, description.as_deref()).await
        }
        ModuleAction::List => list_modules().await,
        ModuleAction::Info { name } => show_module_info(name).await,
        ModuleAction::Enable { name } => enable_module(name).await,
        ModuleAction::Disable { name } => disable_module(name).await,
        ModuleAction::Install { package, production, version, git } => {
            install_module(package, *production, version.as_deref(), *git).await
        }
    }
}

/// Create a new module with standard structure using templates
async fn create_module(name: &str, author: &str, description: Option<&str>) -> Result<()> {
    println!("🏗️  {} bounded context module: {}", "Creating".bright_green(), name.bright_cyan());

    // Module structure follows FINAL_ARCHITECTURE_DECISIONS.md:
    // libs/modules/{module}/ (bounded contexts - complete domain ownership)
    let module_path = Path::new("libs").join("modules").join(name);

    // Check if module already exists
    if module_path.exists() {
        return Err(anyhow::anyhow!("Module '{}' already exists at libs/modules/{}", name, name));
    }

    // Create libs/modules directory if it doesn't exist
    if !Path::new("libs/modules").exists() {
        fs::create_dir_all("libs/modules")?;
        println!("📁 Created libs/modules/ directory for bounded contexts");
    }

    // Create template context
    let context = TemplateContext::new(name, author, description);

    // Get template directory
    let template_dir = get_module_template_dir();
    if !template_dir.exists() {
        return Err(anyhow::anyhow!(
            "Template directory not found at: {:?}. Please ensure templates are available.",
            template_dir
        ));
    }

    // Process and copy template files
    copy_and_process_template_dir(&template_dir, &module_path, &context)?;

    println!("✅ Bounded context module '{}' created successfully!", name.bright_green());
    println!("📁 Location: {}", module_path.display().to_string().cyan());
    println!();
    println!("📋 Next steps:");
    println!("   1. {} entity schema: {}", "Define".bright_yellow(), format!("libs/modules/{}/schema/models/<entity>.model.yaml", name).cyan());
    println!("   2. {} code: {}", "Generate".bright_yellow(), format!("metaphor schema generate {} --target all", name).cyan());
    println!("   3. {} migrations: {}", "Run".bright_yellow(), "sqlx migrate run".cyan());
    println!("   4. {} development: {}", "Start".bright_yellow(), "metaphor dev:serve".cyan());
    println!();
    println!("🏛️  This is a DDD Bounded Context with schema-first approach!");
    println!("    📋 Schema definitions: libs/modules/{}/schema/", name);
    println!("    🔗 Single source of truth for this bounded context");

    Ok(())
}

/// List all available modules
async fn list_modules() -> Result<()> {
    println!("📋 {} modules:", "Available".bright_green());

    // Show Official Metaphor Modules (Priority 1.2+)
    println!("\n🏢 {} Official Metaphor Modules:", "Framework".bright_blue());
    let official_modules = [
        ("metaphor-cache", "Redis and in-memory caching support"),
        ("metaphor-email", "Multi-provider email service (SMTP, SES, Mailgun)"),
        ("metaphor-queue", "Message queue system (Redis, SQS)"),
        ("metaphor-search", "Search integration (Elasticsearch, Algolia)"),
        ("metaphor-storage", "File storage system (S3, local, MinIO)"),
    ];

    for (module, description) in &official_modules {
        println!("  ✅ {} - {}", module.cyan(), description.bright_black());
    }

    // Show user-created bounded contexts
    println!("\n🏗️  {} Bounded Context Modules:", "Business".bright_green());
    let modules_dir = Path::new("libs").join("modules");

    if !modules_dir.exists() {
        println!("  💡 No bounded contexts found. Create your first bounded context with:");
        println!("     {}", "metaphor module:create <name>".cyan());
        return Ok(());
    }

    let mut modules_found = false;

    for entry in fs::read_dir(&modules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let module_name = path.file_name().unwrap().to_str().unwrap();
            modules_found = true;

            // Check if it has Cargo.toml (should be metaphor-<module>)
            let cargo_toml = path.join("Cargo.toml");
            let status = if cargo_toml.exists() {
                "✅".bright_green()
            } else {
                "❌".bright_red()
            };

            // Check if it has schema/ structure (schema-first approach)
            let schema_dir = path.join("schema");
            let schema_status = if schema_dir.exists() {
                "📋".bright_blue()
            } else {
                "⚠️".bright_yellow()
            };

            println!("  {} {} {} {}", status, schema_status, module_name.cyan(), "(bounded context)".bright_black());
        }
    }

    if !modules_found {
        println!("  💡 No bounded contexts found. Create your first bounded context with:");
        println!("     {}", "metaphor module:create <name>".cyan());
    }

    println!("\n📖 Usage:");
    println!("   🏢 Official modules: Add to your app's Cargo.toml dependencies");
    println!("   🏗️  Bounded contexts: Business-specific modules with DDD structure");
    println!("   📋 Location: libs/modules/<name>/ (not apps/metaphor/)");

    Ok(())
}

/// Show detailed information about a module
async fn show_module_info(name: &str) -> Result<()> {
    println!("ℹ️  {} information for: {}", "Module".bright_blue(), name.bright_cyan());

    // Check if it's an Official Metaphor Module
    if let Some(info) = get_official_module_info(name) {
        show_official_module_info(name, &info).await?;
        return Ok(());
    }

    // Handle user-created bounded contexts
    let module_path = Path::new("libs").join("modules").join(name);

    if !module_path.exists() {
        return Err(anyhow::anyhow!("Bounded context '{}' not found. Available modules:\n\nOfficial modules: metaphor-cache, metaphor-email, metaphor-queue, metaphor-search, metaphor-storage\n\nBounded contexts: Check 'metaphor module:list'\n\nLocation: libs/modules/<name>/ (not apps/metaphor/)", name));
    }

    // Read Cargo.toml for module info
    let cargo_toml_path = module_path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml_path)?;

        // Parse basic info from Cargo.toml
        for line in cargo_content.lines() {
            if line.starts_with("description = ") {
                println!("📝 {}", line.replace("description = ", "").trim_matches('"'));
            } else if line.starts_with("version = ") {
                println!("🏷️  {}", line.replace("version = ", "").trim_matches('"'));
            }
        }
    }

    // Show directory structure
    println!("\n📁 Directory structure:");
    for entry in fs::read_dir(&module_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            println!("  📂 {}", path.file_name().unwrap().to_str().unwrap());
        } else if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.ends_with(".rs") || name_str.ends_with(".proto") || name_str == "Cargo.toml" || name_str == "README.md" {
                    println!("  📄 {}", name_str);
                }
            }
        }
    }

    Ok(())
}

/// Get information about Official Metaphor Modules
fn get_official_module_info(name: &str) -> Option<OfficialModuleInfo> {
    match name {
        "metaphor-cache" => Some(OfficialModuleInfo {
            description: "Redis and in-memory caching support for Metaphor Framework".to_string(),
            version: "2.0.0".to_string(),
            features: vec![
                "Redis backend with cluster support".to_string(),
                "In-memory caching for development".to_string(),
                "Generic CacheService trait".to_string(),
                "Cache invalidation strategies".to_string(),
                "Performance monitoring and metrics".to_string(),
            ],
            usage: "metaphor-cache = { path = \"../../../crates/metaphor-cache\" }".to_string(),
        }),
        "metaphor-email" => Some(OfficialModuleInfo {
            description: "Multi-provider email service with SMTP, SES, and Mailgun support".to_string(),
            version: "2.0.0".to_string(),
            features: vec![
                "SMTP support for self-hosted email".to_string(),
                "Amazon SES integration".to_string(),
                "Mailgun API support".to_string(),
                "Email template system".to_string(),
                "Async email delivery with retry".to_string(),
                "Email tracking and analytics".to_string(),
            ],
            usage: "metaphor-email = { path = \"../../../crates/metaphor-email\" }".to_string(),
        }),
        "metaphor-queue" => Some(OfficialModuleInfo {
            description: "Message queue system with Redis and AWS SQS support".to_string(),
            version: "2.0.0".to_string(),
            features: vec![
                "Redis queue with priority support".to_string(),
                "AWS SQS integration".to_string(),
                "Batch processing capabilities".to_string(),
                "Dead letter queue handling".to_string(),
                "Queue monitoring and metrics".to_string(),
                "Generic QueueService trait".to_string(),
            ],
            usage: "metaphor-queue = { path = \"../../../crates/metaphor-queue\" }".to_string(),
        }),
        "metaphor-search" => Some(OfficialModuleInfo {
            description: "Search integration with Elasticsearch and Algolia support".to_string(),
            version: "2.0.0".to_string(),
            features: vec![
                "Elasticsearch backend with full-text search".to_string(),
                "Algolia integration for search-as-a-service".to_string(),
                "Advanced search features (filtering, faceting, aggregations)".to_string(),
                "Search suggestions and autocomplete".to_string(),
                "Analytics and search metrics".to_string(),
                "Generic SearchService trait".to_string(),
            ],
            usage: "metaphor-search = { path = \"../../../crates/metaphor-search\" }".to_string(),
        }),
        "metaphor-storage" => Some(OfficialModuleInfo {
            description: "File storage system with S3, local filesystem, and MinIO support".to_string(),
            version: "2.0.0".to_string(),
            features: vec![
                "AWS S3 integration with multipart upload".to_string(),
                "Local filesystem storage".to_string(),
                "MinIO S3-compatible storage".to_string(),
                "Presigned URL generation".to_string(),
                "File encryption and compression".to_string(),
                "Generic StorageService trait".to_string(),
            ],
            usage: "metaphor-storage = { path = \"../../../crates/metaphor-storage\" }".to_string(),
        }),
        _ => None,
    }
}

/// Show information about Official Metaphor Modules
async fn show_official_module_info(name: &str, info: &OfficialModuleInfo) -> Result<()> {
    println!("🏢 {} Official Metaphor Module", "Framework".bright_blue());
    println!("📝 {}", info.description);
    println!("🏷️  Version: {}", info.version.bright_green());

    println!("\n✨ {} Features:", "Key".bright_yellow());
    for feature in &info.features {
        println!("  • {}", feature);
    }

    println!("\n📦 {} Usage:", "Cargo".bright_cyan());
    println!("  {}", info.usage.bright_black());

    println!("\n🚀 {} Example:", "Quick Start".bright_green());
    println!("  // In your Cargo.toml");
    println!("  {}", info.usage);
    println!();
    println!("  // In your Rust code");
    match name {
        "metaphor-cache" => {
            println!("  use metaphor_cache::{{CacheService, RedisCache}};");
            println!("  let cache = RedisCache::new(config).await?;");
            println!("  cache.set(\"key\", \"value\").await?;");
        }
        "metaphor-email" => {
            println!("  use metaphor_email::{{EmailService, SmtpProvider}};");
            println!("  let email = EmailService::new(SmtpProvider::new(config))?;");
            println!("  email.send_email(\"to@example.com\", \"Subject\", \"Body\").await?;");
        }
        "metaphor-queue" => {
            println!("  use metaphor_queue::{{QueueService, RedisQueue}};");
            println!("  let queue = RedisQueue::new(config).await?;");
            println!("  queue.enqueue(message).await?;");
        }
        "metaphor-search" => {
            println!("  use metaphor_search::{{SearchService, ElasticsearchSearch}};");
            println!("  let search = ElasticsearchSearch::new(config).await?;");
            println!("  search.index_document(\"index\", document).await?;");
        }
        "metaphor-storage" => {
            println!("  use metaphor_storage::{{StorageService, S3Storage}};");
            println!("  let storage = S3Storage::new(config).await?;");
            println!("  storage.upload_bytes(\"file.txt\", data, None).await?;");
        }
        _ => {
            println!("  // Check module documentation for usage examples");
        }
    }

    println!("\n📖 {} Documentation:", name);
    println!("  📁 Location: {}", format!("crates/{}/", name).cyan());
    println!("  📚 Full API docs: {}", format!("https://docs.rs/{}", name).bright_blue());

    Ok(())
}

/// Information structure for Official Metaphor Modules
struct OfficialModuleInfo {
    description: String,
    version: String,
    features: Vec<String>,
    usage: String,
}

/// Toggle a module's enabled state in apps/metaphor/config/application.yml
fn set_module_enabled(name: &str, enabled: bool) -> Result<()> {
    let known_modules = ["sapiens", "postman", "bucket"];
    let name_lower = name.to_lowercase();
    if !known_modules.contains(&name_lower.as_str()) {
        return Err(anyhow::anyhow!(
            "Unknown module: {}. Available modules: {}",
            name,
            known_modules.join(", ")
        ));
    }

    let config_path = "apps/metaphor/config/application.yml";
    let content = std::fs::read_to_string(config_path)
        .context("Failed to read config. Does apps/metaphor/config/application.yml exist?")?;
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .context("Failed to parse application.yml")?;

    // Update modules.<name>.enabled
    if let Some(modules) = yaml.get_mut("modules") {
        if let Some(module) = modules.get_mut(&name_lower) {
            module["enabled"] = serde_yaml::Value::Bool(enabled);
        }
    }

    // Update services.<name>.enabled
    if let Some(services) = yaml.get_mut("services") {
        if let Some(service) = services.get_mut(&name_lower) {
            service["enabled"] = serde_yaml::Value::Bool(enabled);
        }
    }

    let output = serde_yaml::to_string(&yaml)
        .context("Failed to serialize configuration")?;
    std::fs::write(config_path, output)
        .context("Failed to write configuration")?;

    Ok(())
}

/// Enable a module in configuration
async fn enable_module(name: &str) -> Result<()> {
    println!("🔧 {} module: {}", "Enabling".bright_yellow(), name.bright_cyan());

    set_module_enabled(name, true)?;

    println!("✅ {} module enabled", name.bright_green());
    println!("💾 Configuration updated. Restart services for changes to take effect.");
    Ok(())
}

/// Disable a module in configuration
async fn disable_module(name: &str) -> Result<()> {
    println!("🔧 {} module: {}", "Disabling".bright_red(), name.bright_cyan());

    set_module_enabled(name, false)?;

    println!("✅ {} module disabled", name.bright_red());
    println!("💾 Configuration updated. Restart services for changes to take effect.");
    Ok(())
}

/// Install an external module package
async fn install_module(package: &str, production: bool, version: Option<&str>, git: bool) -> Result<()> {
    println!("📦 {} module: {}", "Installing".bright_green(), package.bright_cyan());

    let root_dir = Path::new(".");
    let cargo_toml_path = root_dir.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("Cargo.toml not found. Run this command from the monorepo root."));
    }

    // Determine dependency section and format
    let dependency_section = if production { "[dependencies]" } else { "[dev-dependencies]" };

    let dependency_spec = if git {
        if let Some(v) = version {
            format!("{} = {{ git = \"{}\", branch = \"{}\" }}", package, package, v)
        } else {
            format!("{} = {{ git = \"{}\" }}", package, package)
        }
    } else {
        if let Some(v) = version {
            format!("{} = \"{}\"", package, v)
        } else {
            format!("{} = \"*\"", package)
        }
    };

    println!("📋 {} to add: {} {} = {}", "Dependency", dependency_section, package, dependency_spec);

    // Use cargo add to install the dependency
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.args(["add", package]);

    if production {
        cargo_cmd.arg("--");
    } else {
        cargo_cmd.args(["--dev"]);
    }

    if let Some(v) = version {
        if git {
            cargo_cmd.args(["--git", package, "--branch", v]);
        } else {
            cargo_cmd.args(["--version", v]);
        }
    }

    if git && version.is_none() {
        cargo_cmd.args(["--git", package]);
    }

    let args: Vec<String> = cargo_cmd.get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();
    println!("🚀 {} command: {}", "Running", format!("cargo {}", args.join(" ")).bright_black());

    let output = cargo_cmd.output()?;

    if output.status.success() {
        println!("✅ Module '{}' installed successfully!", package.bright_green());
        println!("📚 Added to {} section", dependency_section);
        println!();
        println!("📋 Next steps:");
        println!("   1. {} the module in your code: {}", "Import".bright_yellow(), format!("use {}::...", package).cyan());
        println!("   2. {} your application: {}", "Build".bright_yellow(), "cargo build".cyan());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        return Err(anyhow::anyhow!(
            "Failed to install module '{}':\nstdout: {}\nstderr: {}",
            package, stdout, stderr
        ));
    }

    Ok(())
}