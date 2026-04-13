//! Configuration management system
//! Laravel-inspired configuration loading with environment variable support

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

/// Application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub modules: ModulesConfig,
    pub external_services: ExternalServicesConfig,
    pub health: HealthConfig,
    pub jobs: JobsConfig,
    pub metrics: MetricsConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive: u64,
    pub read_timeout: u64,
    pub write_timeout: u64,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub tracing: TracingConfig,
}

/// Tracing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    pub environment: String,
    pub service_name: String,
    pub version: String,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub password_min_length: u8,
    pub bcrypt_rounds: u32,
    pub cors_origins: Vec<String>,
}

/// Modules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModulesConfig {
    pub sapiens: ModuleConfig,
    pub postman: ModuleConfig,
    pub bucket: ModuleConfig,
}

/// Individual module configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleConfig {
    pub enabled: bool,
    pub database_url: Option<String>,
    pub settings: serde_json::Value,
}

/// External services configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalServicesConfig {
    pub redis: Option<RedisConfig>,
    pub smtp: Option<SmtpConfig>,
    pub storage: Option<StorageConfig>,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

/// SMTP configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    pub provider: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

/// Health check configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthConfig {
    pub check_interval: u64,
    pub timeout: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

/// Background jobs configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobsConfig {
    pub enabled: bool,
    pub max_concurrent_jobs: u32,
    pub queue_size: u32,
    pub retry_attempts: u32,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval: u64,
    pub retention_days: u32,
}

impl AppConfig {
    /// Load configuration from files and environment variables
    pub fn load() -> Result<Self> {
        info!("🔧 Loading application configuration");

        // Determine environment
        let environment = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
        info!("📋 Environment: {}", environment);

        // Load base configuration
        let mut config = Self::load_from_file("config/application.yml")?;

        // Load environment-specific configuration
        let env_file = format!("config/application-{}.yml", environment);
        if Path::new(&env_file).exists() {
            let env_config = Self::load_from_file(&env_file)?;
            config = config.merge(env_config);
        }

        // Override with environment variables
        config = config.apply_env_overrides();

        info!("✅ Configuration loaded successfully");
        Ok(config)
    }

    /// Load configuration from a YAML file
    fn load_from_file(file_path: &str) -> Result<Self> {
        let yaml_content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", file_path, e))?;

        let config: AppConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", file_path, e))?;

        Ok(config)
    }

    /// Merge configurations (second config overrides first)
    fn merge(self, other: AppConfig) -> Self {
        AppConfig {
            server: other.server,
            database: other.database,
            logging: other.logging,
            security: other.security,
            modules: other.modules,
            external_services: other.external_services,
            health: other.health,
            jobs: other.jobs,
            metrics: other.metrics,
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(mut self) -> Self {
        // Server overrides
        if let Ok(host) = std::env::var("HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }

        // Database overrides
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        if let Ok(max_conn) = std::env::var("DB_MAX_CONNECTIONS") {
            if let Ok(max_conn_num) = max_conn.parse() {
                self.database.max_connections = max_conn_num;
            }
        }

        // Logging overrides
        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }

        // Security overrides
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            self.security.jwt_secret = jwt_secret;
        }

        self
    }

    /// Get database URL with fallback
    pub fn get_database_url(&self) -> &str {
        &self.database.url
    }

    /// Get server address as SocketAddr
    pub fn server_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid server address")
    }

    /// Check if a module is enabled
    pub fn is_module_enabled(&self, module_name: &str) -> bool {
        match module_name {
            "sapiens" => self.modules.sapiens.enabled,
            "postman" => self.modules.postman.enabled,
            "bucket" => self.modules.bucket.enabled,
            _ => false,
        }
    }

    /// Get module configuration
    pub fn get_module_config(&self, module_name: &str) -> Option<&ModuleConfig> {
        match module_name {
            "sapiens" => Some(&self.modules.sapiens),
            "postman" => Some(&self.modules.postman),
            "bucket" => Some(&self.modules.bucket),
            _ => None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3001,
                workers: num_cpus::get(),
                keep_alive: 75,
                read_timeout: 30,
                write_timeout: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://postgres:password@localhost:5432/bersihirdb".to_string(),
                max_connections: 20,
                min_connections: 5,
                connect_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 3600,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                tracing: TracingConfig {
                    environment: "development".to_string(),
                    service_name: "metaphor-app".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
            security: SecurityConfig {
                jwt_secret: "your-super-secret-jwt-key".to_string(),
                jwt_expiration: 3600,
                password_min_length: 8,
                bcrypt_rounds: 12,
                cors_origins: vec!["*".to_string()],
            },
            modules: ModulesConfig {
                sapiens: ModuleConfig {
                    enabled: true,
                    database_url: None,
                    settings: serde_json::json!({}),
                },
                postman: ModuleConfig {
                    enabled: true,
                    database_url: None,
                    settings: serde_json::json!({}),
                },
                bucket: ModuleConfig {
                    enabled: true,
                    database_url: None,
                    settings: serde_json::json!({}),
                },
            },
            external_services: ExternalServicesConfig {
                redis: None,
                smtp: None,
                storage: None,
            },
            health: HealthConfig {
                check_interval: 60,
                timeout: 10,
                failure_threshold: 3,
                success_threshold: 2,
            },
            jobs: JobsConfig {
                enabled: false,
                max_concurrent_jobs: 10,
                queue_size: 100,
                retry_attempts: 3,
            },
            metrics: MetricsConfig {
                enabled: false,
                collection_interval: 60,
                retention_days: 30,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3001);
        assert_eq!(config.database.max_connections, 20);
        assert!(config.modules.sapiens.enabled);
    }

    #[test]
    fn test_env_overrides() {
        env::set_var("PORT", "8080");
        env::set_var("LOG_LEVEL", "debug");

        let config = AppConfig::default().apply_env_overrides();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "debug");

        env::remove_var("PORT");
        env::remove_var("LOG_LEVEL");
    }

    #[test]
    fn test_module_enabled() {
        let config = AppConfig::default();
        assert!(config.is_module_enabled("sapiens"));
        assert!(config.is_module_enabled("postman"));
        assert!(config.is_module_enabled("bucket"));
        assert!(!config.is_module_enabled("nonexistent"));
    }
}