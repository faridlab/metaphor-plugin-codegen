//! Apps command handler
//!
//! Handles the `apps:generate` command for creating new Metaphor Framework applications
//! from templates with Clean Architecture structure.

use anyhow::{Result, anyhow};
use colored::*;
use std::path::PathBuf;

use crate::app_generator::{AppGenerator, AppGeneratorConfig};

/// Apps commands
#[derive(Debug, clap::Subcommand, Clone)]
pub enum AppsArgs {
    /// Generate a new application from template
    Generate {
        /// Application name (kebab-case)
        #[arg(help = "Application name in kebab-case (e.g., my-service)")]
        name: String,

        /// Application type
        #[arg(long, short = 't', help = "Application type", default_value = "api")]
        app_type: String,

        /// Port number
        #[arg(long, short = 'p', help = "Port number")]
        port: Option<u16>,

        /// Database type
        #[arg(long, short = 'd', help = "Database type", default_value = "postgresql")]
        database: String,

        /// Application description
        #[arg(long, short = 'm', help = "Application description")]
        description: Option<String>,

        /// Enable authentication
        #[arg(long, help = "Enable authentication features")]
        auth: bool,

        /// Enable metrics collection
        #[arg(long, help = "Enable metrics collection")]
        metrics: bool,

        /// Output directory
        #[arg(long, short = 'o', help = "Output directory", default_value = "apps")]
        output: String,

        /// Author name
        #[arg(long, help = "Author name")]
        author: Option<String>,

        /// Author email
        #[arg(long, help = "Author email")]
        email: Option<String>,
    },

    /// List available app templates
    List {
        /// Show detailed information
        #[arg(long, short = 'd')]
        detailed: bool,
    },

    /// Validate app configuration
    Validate {
        /// Application name to validate
        name: String,
    },
}

/// Handle apps command
pub async fn handle_command(args: AppsArgs) -> Result<()> {
    match args {
        AppsArgs::Generate {
            name,
            app_type,
            port,
            database,
            description,
            auth,
            metrics,
            output,
            author,
            email,
        } => {
            handle_generate_command(
                GenerateArgs {
                    name,
                    app_type,
                    port,
                    database,
                    description,
                    auth,
                    metrics,
                    output,
                    author,
                    email,
                },
            ).await
        }
        AppsArgs::List { detailed } => {
            handle_list_command(detailed).await
        }
        AppsArgs::Validate { name } => {
            handle_validate_command(&name).await
        }
    }
}

/// Generate command arguments
struct GenerateArgs {
    name: String,
    app_type: String,
    port: Option<u16>,
    database: String,
    description: Option<String>,
    auth: bool,
    metrics: bool,
    output: String,
    author: Option<String>,
    email: Option<String>,
}

/// Handle app generation command
async fn handle_generate_command(args: GenerateArgs) -> Result<()> {
    println!("🚀 {} {}", "Generating Metaphor Framework app:".green().bold(), args.name.cyan());

    // Validate app name
    validate_app_name(&args.name)?;

    // Validate app type
    validate_app_type(&args.app_type)?;

    // Validate database type
    validate_database_type(&args.database)?;

    // Create generator configuration
    let mut config = AppGeneratorConfig::default();
    config.app_name = args.name.clone();
    config.app_type = args.app_type.clone();
    config.database_type = args.database.clone();
    config.auth_enabled = args.auth;
    config.metrics_enabled = args.metrics;

    // Set optional parameters
    if let Some(port) = args.port {
        config.app_port = port;
    } else {
        // Set default port based on app type
        config.app_port = get_default_port(&args.app_type);
    }

    if let Some(description) = args.description {
        config.app_description = description;
    } else {
        config.app_description = format!("{} service", to_title_case(&args.name));
    }

    if let Some(author) = args.author {
        config.author_name = author;
    }

    if let Some(email) = args.email {
        config.author_email = email;
    }

    config.database_name = format!("{}_db", args.name.replace('-', "_"));

    // Show configuration summary
    print_configuration_summary(&config);

    // Create app generator
    let generator = AppGenerator::new()?;

    // Generate the app
    let output_dir = PathBuf::from(&args.output);
    generator.generate_app(&config, &output_dir).await?;

    // Show next steps
    print_next_steps(&config, &args.output);

    Ok(())
}

/// Handle list command
async fn handle_list_command(detailed: bool) -> Result<()> {
    println!("📋 {}:", "Available App Templates".cyan().bold());

    let templates = vec![
        ("api", "REST API service", "Standard HTTP API with REST endpoints"),
        ("auth", "Authentication service", "User authentication and authorization"),
        ("worker", "Background worker", "Asynchronous background job processor"),
        ("scheduler", "Task scheduler", "Cron-based task scheduling service"),
    ];

    for (name, description, details) in templates {
        if detailed {
            println!("  {} {} ({})", name.cyan(), description.green(), details.dimmed());
        } else {
            println!("  {} {} - {}", name.cyan(), description.green(), details.dimmed());
        }
    }

    println!();
    println!("💡 {}:", "Usage Examples".cyan().bold());
    println!("  {} my-service                    # Generate API service", "metaphor apps:generate".yellow());
    println!("  {} auth-service --type auth       # Generate auth service", "metaphor apps:generate".yellow());
    println!("  {} worker --port 3004            # Generate worker with custom port", "metaphor apps:generate".yellow());
    println!("  {} scheduler --auth --metrics     # Generate scheduler with features", "metaphor apps:generate".yellow());

    Ok(())
}

/// Handle validate command
async fn handle_validate_command(name: &str) -> Result<()> {
    println!("🔍 {}:", format!("Validating app name: {}", name).cyan());

    match validate_app_name(name) {
        Ok(()) => {
            println!("✅ {} {} is a valid app name", "Validation passed".green(), name.cyan());
        }
        Err(e) => {
            println!("❌ {} {}: {}", "Validation failed".red(), name.cyan(), e);
            return Err(e);
        }
    }

    // Check if app already exists
    let apps_dir = std::path::Path::new("apps");
    if apps_dir.join(name).exists() {
        println!("⚠️  {} {} {}", "Warning".yellow(), name.cyan(), "already exists in apps/");
    } else {
        println!("✅ {} {} is available for creation", "App name available".green(), name.cyan());
    }

    Ok(())
}

/// Validate app name format
fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("App name cannot be empty"));
    }

    if name.len() < 3 {
        return Err(anyhow!("App name must be at least 3 characters long"));
    }

    if name.len() > 50 {
        return Err(anyhow!("App name must be less than 50 characters long"));
    }

    // Check for kebab-case format
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(anyhow!("App name must contain only lowercase letters, numbers, and hyphens"));
    }

    // Check that it doesn't start or end with a hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow!("App name cannot start or end with a hyphen"));
    }

    // Check for consecutive hyphens
    if name.contains("--") {
        return Err(anyhow!("App name cannot contain consecutive hyphens"));
    }

    // Check for reserved names
    let reserved_names = vec!["metaphor", "framework", "cli", "test", "demo"];
    if reserved_names.contains(&name) {
        return Err(anyhow!("App name '{}' is reserved", name));
    }

    Ok(())
}

/// Validate app type
fn validate_app_type(app_type: &str) -> Result<()> {
    let valid_types = vec!["api", "auth", "worker", "scheduler"];
    if !valid_types.contains(&app_type) {
        return Err(anyhow!(
            "Invalid app type '{}'. Valid types: {}",
            app_type,
            valid_types.join(", ")
        ));
    }
    Ok(())
}

/// Validate database type
fn validate_database_type(database: &str) -> Result<()> {
    let valid_databases = vec!["postgresql", "mongodb", "sqlite", "none"];
    if !valid_databases.contains(&database) {
        return Err(anyhow!(
            "Invalid database type '{}'. Valid types: {}",
            database,
            valid_databases.join(", ")
        ));
    }
    Ok(())
}

/// Get default port for app type
fn get_default_port(app_type: &str) -> u16 {
    match app_type {
        "auth" => 3002,
        "worker" => 3003,
        "scheduler" => 3004,
        _ => 3000,
    }
}

/// Print configuration summary
fn print_configuration_summary(config: &AppGeneratorConfig) {
    println!("📝 {}:", "Configuration".cyan().bold());
    println!("  {} App Name: {}", "•".dimmed(), config.app_name.cyan());
    println!("  {} Type: {}", "•".dimmed(), config.app_type.green());
    println!("  {} Port: {}", "•".dimmed(), config.app_port.to_string().yellow());
    println!("  {} Database: {}", "•".dimmed(), config.database_type.blue());
    println!("  {} Auth: {}", "•".dimmed(), if config.auth_enabled { "Enabled".green() } else { "Disabled".red() });
    println!("  {} Metrics: {}", "•".dimmed(), if config.metrics_enabled { "Enabled".green() } else { "Disabled".red() });
    println!("  {} Description: {}", "•".dimmed(), config.app_description.dimmed());
    println!("  {} Author: {} <{}>", "•".dimmed(), config.author_name.green(), config.author_email.blue());
    println!("  {} Database Name: {}", "•".dimmed(), config.database_name.yellow());
    println!();
}

/// Print next steps after generation
fn print_next_steps(config: &AppGeneratorConfig, output_dir: &str) {
    println!("🎯 {}:", "Next Steps".green().bold());
    println!("  1. {} {} {}", "cd".yellow(), output_dir, config.app_name.cyan());
    println!("  2. {} {} {}", "cargo".yellow(), "build".green(), "- Build the application");
    println!("  3. {} {} {}", "cargo".yellow(), "run".green(), "- Start the application");
    println!("  4. {} {}", "Visit".yellow(), format!("http://localhost:{}", config.app_port).cyan());
    println!("  5. {} {} {} {}", "Check".yellow(), "health".green(), "endpoint at", "/health".cyan());
    println!();
    println!("📚 {}:", "Documentation".blue().bold());
    println!("  {} {} {}", "•".dimmed(), "API:".dimmed(), format!("http://localhost:{}/api/v1", config.app_port).cyan());
    println!("  {} {} {}", "•".dimmed(), "Health:".dimmed(), format!("http://localhost:{}/health", config.app_port).cyan());
    println!("  {} {} {}", "•".dimmed(), "README:".dimmed(), format!("{}/README.md", output_dir).cyan());
    println!();
}

/// Convert string to title case
fn to_title_case(s: &str) -> String {
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
    fn test_validate_app_name() {
        assert!(validate_app_name("my-service").is_ok());
        assert!(validate_app_name("api").is_ok());
        assert!(validate_app_name("user-auth-service").is_ok());

        assert!(validate_app_name("").is_err());
        assert!(validate_app_name("ab").is_err());
        assert!(validate_app_name("My-Service").is_err());
        assert!(validate_app_name("my_service").is_err());
        assert!(validate_app_name("-service").is_err());
        assert!(validate_app_name("service-").is_err());
        assert!(validate_app_name("my--service").is_err());
    }

    #[test]
    fn test_validate_app_type() {
        assert!(validate_app_type("api").is_ok());
        assert!(validate_app_type("auth").is_ok());
        assert!(validate_app_type("worker").is_ok());
        assert!(validate_app_type("scheduler").is_ok());

        assert!(validate_app_type("invalid").is_err());
    }

    #[test]
    fn test_validate_database_type() {
        assert!(validate_database_type("postgresql").is_ok());
        assert!(validate_database_type("mongodb").is_ok());
        assert!(validate_database_type("sqlite").is_ok());
        assert!(validate_database_type("none").is_ok());

        assert!(validate_database_type("invalid").is_err());
    }

    #[test]
    fn test_get_default_port() {
        assert_eq!(get_default_port("api"), 3000);
        assert_eq!(get_default_port("auth"), 3002);
        assert_eq!(get_default_port("worker"), 3003);
        assert_eq!(get_default_port("scheduler"), 3004);
        assert_eq!(get_default_port("unknown"), 3000);
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("my-service"), "My Service");
        assert_eq!(to_title_case("api"), "Api");
        assert_eq!(to_title_case("user-auth-service"), "User Auth Service");
    }
}