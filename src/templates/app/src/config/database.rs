//! Database configuration loading and management

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Primary database connection URL
    pub url: String,

    /// Database host (parsed from URL if not provided)
    pub host: Option<String>,

    /// Database port (parsed from URL if not provided)
    pub port: Option<u16>,

    /// Database name (parsed from URL if not provided)
    pub database: Option<String>,

    /// Database username (parsed from URL if not provided)
    pub username: Option<String>,

    /// Database password (parsed from URL if not provided)
    pub password: Option<String>,

    /// Connection pool settings
    pub max_connections: u32,
    pub min_connections: u32,

    /// Connection timeout settings
    pub connect_timeout: u64,
    pub idle_timeout: u64,

    /// Health check settings
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://postgres:password@localhost:5432/bersihirdb".to_string(),
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            max_connections: 20,
            min_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
            health_check_interval: Duration::from_secs(60),
            health_check_timeout: Duration::from_secs(10),
        }
    }
}

impl DatabaseConfig {
    /// Create database config from environment variables and defaults
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/bersihirdb".to_string());

        let mut config = Self {
            url,
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            connect_timeout: std::env::var("DB_CONNECT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            idle_timeout: std::env::var("DB_IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600),
            health_check_interval: Duration::from_secs(
                std::env::var("DB_HEALTH_CHECK_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60)
            ),
            health_check_timeout: Duration::from_secs(
                std::env::var("DB_HEALTH_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10)
            ),
            ..Default::default()
        };

        // Parse connection details from URL
        config.parse_connection_details()?;

        Ok(config)
    }

    /// Parse database connection details from URL
    fn parse_connection_details(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple URL parsing for PostgreSQL
        if self.url.starts_with("postgresql://") {
            let url_part = &self.url[13..]; // Remove "postgresql://"

            let parts: Vec<&str> = url_part.split('@').collect();
            if parts.len() == 2 {
                // Parse credentials
                let auth_parts: Vec<&str> = parts[0].split(':').collect();
                if auth_parts.len() >= 2 {
                    self.username = Some(auth_parts[0].to_string());
                    self.password = Some(auth_parts[1].to_string());
                }

                // Parse host and database
                let host_db_parts: Vec<&str> = parts[1].split('/').collect();
                if !host_db_parts.is_empty() {
                    let host_port = host_db_parts[0];
                    let host_port_parts: Vec<&str> = host_port.split(':').collect();
                    self.host = Some(host_port_parts[0].to_string());
                    if host_port_parts.len() >= 2 {
                        self.port = host_port_parts[1].parse().ok();
                    }
                }
                if host_db_parts.len() >= 2 {
                    self.database = Some(host_db_parts[1].to_string());
                }
            }
        }

        Ok(())
    }

    /// Get connection options for SQLx
    pub fn sqlx_connect_options(&self) -> Result<sqlx::postgres::PgConnectOptions, sqlx::Error> {
        sqlx::postgres::PgConnectOptions::from_str(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout, 30);
    }

    #[test]
    fn test_database_config_from_env() {
        std::env::set_var("DB_MAX_CONNECTIONS", "50");
        std::env::set_var("DB_MIN_CONNECTIONS", "10");

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 10);

        std::env::remove_var("DB_MAX_CONNECTIONS");
        std::env::remove_var("DB_MIN_CONNECTIONS");
    }
}