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
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
    pub refresh_expiration: i64,
    pub algorithm: String,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub allowed_origins: String,
    pub allowed_methods: String,
    pub allowed_headers: String,
    pub max_age: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// Modules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModulesConfig {
    pub sapiens: SapiensConfig,
    pub postman: PostmanConfig,
    pub bucket: BucketConfig,
}

/// Sapiens module configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SapiensConfig {
    pub enabled: bool,
    pub database_url: String,
    pub jwt_secret: String,
    pub features: SapiensFeatures,
}

/// Sapiens features configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SapiensFeatures {
    pub registration: bool,
    pub email_verification: bool,
    pub password_reset: bool,
}

/// Postman module configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostmanConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_use_tls: bool,
    pub default_from: String,
}

/// Bucket module configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketConfig {
    pub enabled: bool,
    pub storage_type: String,
    pub local_storage_path: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub max_file_size: u64,
}

/// External services configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalServicesConfig {
    pub redis: RedisConfig,
    pub elasticsearch: Option<ElasticsearchConfig>,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: u64,
}

/// Elasticsearch configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub index_prefix: String,
    pub max_connections: u32,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval: u64,
    pub endpoints: HealthEndpoints,
}

/// Health endpoints configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthEndpoints {
    pub database: bool,
    pub redis: bool,
    pub external_services: bool,
}

/// Jobs configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobsConfig {
    pub enabled: bool,
    pub max_workers: usize,
    pub queue_size: usize,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus: PrometheusConfig,
}

/// Prometheus configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub port: u16,
}

impl AppConfig {
    /// Load configuration from files and environment
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        info!("Loading configuration for environment: {}", app_env);

        // Load base configuration
        let mut config: Self = config::load_yaml("config/application.yml")?;

        // Load environment-specific overrides
        let env_file = format!("config/application-{}.yml", app_env);
        if Path::new(&env_file).exists() {
            info!("Loading environment-specific config: {}", env_file);
            let env_config: Self = config::load_yaml(&env_file)?;
            config = config.merge(env_config);
        }

        // Validate critical configuration
        config.validate()?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Validate critical configuration values
    fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        // Validate database configuration
        if self.database.url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        // Validate JWT secret
        if self.security.jwt.secret.is_empty() || self.security.jwt.secret == "your-super-secret-jwt-key-change-in-production" {
            if std::env::var("APP_ENV").unwrap_or_default() == "production" {
                return Err(anyhow::anyhow!("JWT secret must be set in production"));
            } else {
                warn!("Using default JWT secret - please set JWT_SECRET in production");
            }
        }

        Ok(())
    }

    /// Merge configuration with environment-specific overrides
    fn merge(mut self, other: Self) -> Self {
        // Simple merge strategy - replace nested structures
        // In a real implementation, you might want more sophisticated merging
        if other.server.host != "0.0.0.0" {
            self.server = other.server;
        }
        if other.logging.level != "info" {
            self.logging = other.logging;
        }
        if other.security.rate_limit.requests_per_minute != 100 {
            self.security = other.security;
        }
        if !other.modules.sapiens.enabled {
            self.modules = other.modules;
        }
        if other.external_services.redis.url != "redis://localhost:6379" {
            self.external_services = other.external_services;
        }
        if other.health.check_interval != 30 {
            self.health = other.health;
        }
        if other.jobs.max_workers != 4 {
            self.jobs = other.jobs;
        }
        if other.metrics.prometheus.enabled != self.metrics.prometheus.enabled {
            self.metrics = other.metrics;
        }

        self
    }
}

mod config {
    use super::*;
    use serde_yaml;
    use std::fs;

    pub fn load_yaml<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path, e))?;

        serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0,
                workers: 4,
                keep_alive: 75,
                read_timeout: 30,
                write_timeout: 30,
            },
            database: DatabaseConfig {
                url: "".to_string(),
                max_connections: 20,
                min_connections: 5,
                connect_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 1800,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                tracing: TracingConfig {
                    service_name: "metaphor-app".to_string(),
                    service_version: "0.1.0".to_string(),
                    environment: "test".to_string(),
                },
            },
            security: SecurityConfig {
                jwt: JwtConfig {
                    secret: "".to_string(),
                    expiration: 3600,
                    refresh_expiration: 86400,
                    algorithm: "HS256".to_string(),
                },
                cors: CorsConfig {
                    allowed_origins: "*".to_string(),
                    allowed_methods: "GET,POST".to_string(),
                    allowed_headers: "*".to_string(),
                    max_age: 3600,
                },
                rate_limit: RateLimitConfig {
                    requests_per_minute: 100,
                    burst_size: 20,
                },
            },
            modules: ModulesConfig {
                sapiens: SapiensConfig {
                    enabled: true,
                    database_url: "test".to_string(),
                    jwt_secret: "test".to_string(),
                    features: SapiensFeatures {
                        registration: true,
                        email_verification: false,
                        password_reset: true,
                    },
                },
                postman: PostmanConfig {
                    enabled: true,
                    smtp_host: "localhost".to_string(),
                    smtp_port: 1025,
                    smtp_username: "".to_string(),
                    smtp_password: "".to_string(),
                    smtp_use_tls: false,
                    default_from: "test@example.com".to_string(),
                },
                bucket: BucketConfig {
                    enabled: true,
                    storage_type: "local".to_string(),
                    local_storage_path: "./storage".to_string(),
                    s3_region: "us-east-1".to_string(),
                    s3_bucket: "test-bucket".to_string(),
                    max_file_size: 10485760,
                },
            },
            external_services: ExternalServicesConfig {
                redis: RedisConfig {
                    url: "redis://localhost:6379".to_string(),
                    pool_size: 10,
                    connection_timeout: 5,
                },
                elasticsearch: None,
            },
            health: HealthConfig {
                enabled: true,
                check_interval: 30,
                endpoints: HealthEndpoints {
                    database: true,
                    redis: true,
                    external_services: true,
                },
            },
            jobs: JobsConfig {
                enabled: true,
                max_workers: 4,
                queue_size: 1000,
            },
            metrics: MetricsConfig {
                enabled: true,
                prometheus: PrometheusConfig {
                    enabled: false,
                    endpoint: "/metrics".to_string(),
                    port: 9090,
                },
            },
        };

        // Test validation failures
        assert!(config.validate().is_err()); // port = 0

        config.server.port = 3001;
        assert!(config.validate().is_err()); // empty database URL

        config.database.url = "postgresql://test".to_string();
        assert!(config.validate().is_err()); // empty JWT secret

        config.security.jwt.secret = "valid-secret".to_string();
        assert!(config.validate().is_ok());
    }
}