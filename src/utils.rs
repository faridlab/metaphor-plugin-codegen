//! Utility functions for Metaphor CLI

use std::fs;
use std::path::Path;
use once_cell::sync::Lazy;
use regex::Regex;

/// Utility functions for file operations and CLI helpers

/// Static regex for environment variable expansion
/// Compiled once at startup for performance
static ENV_VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")
        .expect("ENV_VAR_REGEX pattern is valid")
});

/// Get database URL from project configuration
///
/// This is the **centralized** function that all CLI tools should use
/// to get database configuration. It ensures consistent behavior across:
/// - migration commands
/// - seed commands
/// - any other database-connected commands
///
/// # Configuration Search Order
/// 1. `.env` file in project root (loads DATABASE_URL, POSTGRES_DB, etc.)
/// 2. `apps/metaphor/config/application.yml`
/// 3. `config/application.yml` (fallback)
///
/// # Returns
/// - `Some(String)` - Database URL if found in config
/// - `None` - Config file not found or database.url not set
///
/// # Example
/// ```no_run
/// use metaphor_cli::utils::get_database_url;
///
/// if let Some(url) = get_database_url() {
///     println!("Using database: {}", url);
/// }
/// ```
pub fn get_database_url() -> Option<String> {
    // First, try to load .env file and check for DATABASE_URL or POSTGRES_DB
    if let Ok(env_content) = fs::read_to_string(".env") {
        for line in env_content.lines() {
            let line = line.trim();
            if line.starts_with("DATABASE_URL=") || line.starts_with("POSTGRES_DB=") {
                if let Some(value) = line.split('=').nth(1) {
                    // If it's POSTGRES_DB, construct full URL
                    let url = if line.starts_with("POSTGRES_DB=") {
                        // Try to get other parts from .env or use defaults
                        let host = get_env_value(".env", "POSTGRES_HOST").unwrap_or_else(|| "localhost".to_string());
                        let port = get_env_value(".env", "POSTGRES_PORT").unwrap_or_else(|| "5432".to_string());
                        let user = get_env_value(".env", "POSTGRES_USER").unwrap_or_else(|| "postgres".to_string());
                        let password = get_env_value(".env", "POSTGRES_PASSWORD").unwrap_or_else(|| "password".to_string());
                        format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, value)
                    } else {
                        value.to_string()
                    };
                    return Some(url);
                }
            }
        }
    }

    // Try YAML config files
    let app_config_paths = [
        "apps/metaphor/config/application.yml",
        "config/application.yml",
    ];

    for config_path in &app_config_paths {
        let content = fs::read_to_string(config_path).ok()?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;

        let database = yaml.get("database")?;
        let db_config = database.as_mapping()?;
        let url = db_config.get("url")?;
        let url_str = url.as_str()?;

        return Some(expand_env_vars(url_str));
    }

    None
}

/// Expand environment variables in a string
///
/// Supports the format `${VAR:default}` or `${VAR}`:
/// - `${VAR}` - Expands to environment variable VAR, or empty string if not set
/// - `${VAR:default}` - Expands to VAR, or "default" if VAR is not set
///
/// # Arguments
/// * `input` - String containing environment variable placeholders
///
/// # Returns
/// String with all `${VAR:default}` patterns replaced with their values
///
/// # Examples
/// ```no_run
/// use metaphor_cli::utils::expand_env_vars;
/// use std::env;
///
/// env::set_var("HOST", "localhost");
/// assert_eq!(expand_env_vars("${HOST}"), "localhost");
/// assert_eq!(expand_env_vars("${PORT:5432}"), "5432");
/// assert_eq!(expand_env_vars("${MISSING:default}"), "default");
/// ```
pub fn expand_env_vars(input: &str) -> String {
    ENV_VAR_REGEX.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];
        let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        std::env::var(var_name).unwrap_or_else(|_| default.to_string())
    })
    .to_string()
}

/// Helper to get a specific value from .env file
///
/// # Arguments
/// * `env_file` - Path to .env file
/// * `key` - Environment variable key to look for (e.g., "POSTGRES_HOST")
///
/// # Returns
/// - `Some(String)` - Value if found
/// - `None` - Key not found or file cannot be read
pub fn get_env_value(env_file: &str, key: &str) -> Option<String> {
    if let Ok(env_content) = fs::read_to_string(env_file) {
        for line in env_content.lines() {
            let line = line.trim();
            if line.starts_with(&format!("{}=", key)) {
                return line.split('=').nth(1).map(|s| s.to_string());
            }
        }
    }
    None
}

/// Check if a path exists
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Create directory with parents if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get current timestamp as string
pub fn timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Generate a random UUID string
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Validate module name
pub fn validate_module_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Module name cannot be empty"));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow::anyhow!(
            "Module name can only contain letters, numbers, and underscores"
        ));
    }

    if name.len() > 50 {
        return Err(anyhow::anyhow!("Module name too long (max 50 characters)"));
    }

    Ok(())
}

/// Validate entity name
pub fn validate_entity_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Entity name cannot be empty"));
    }

    // Check if it's PascalCase
    if !name.chars().next().unwrap().is_uppercase() {
        return Err(anyhow::anyhow!("Entity name must be PascalCase (e.g., 'User', 'Payment')"));
    }

    if name.len() > 100 {
        return Err(anyhow::anyhow!("Entity name too long (max 100 characters)"));
    }

    Ok(())
}

/// Sanitize database URL for display (hide password)
///
/// # Arguments
/// * `url` - Database connection URL
///
/// # Returns
/// String with password masked
///
/// # Example
/// ```no_run
/// use metaphor_cli::utils::sanitize_db_url;
///
/// let url = "postgresql://user:secret@localhost:5432/db";
/// assert_eq!(sanitize_db_url(url), "postgresql://user:***@localhost:5432/db");
/// ```
pub fn sanitize_db_url(url: &str) -> String {
    // Use regex to hide password in connection URL
    static DB_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(postgresql|postgres|mysql|sqlite)://([^:]+):([^@]+)@")
            .expect("DB_URL_REGEX pattern is valid")
    });

    DB_URL_REGEX
        .replace_all(url, "$1://$2:***@")
        .to_string()
}