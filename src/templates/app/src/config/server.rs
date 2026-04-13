//! Server configuration settings

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host IP address
    pub host: IpAddr,

    /// Server port
    pub port: u16,

    /// Number of worker threads (None = auto)
    pub workers: Option<usize>,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Enable graceful shutdown
    pub graceful_shutdown: bool,

    /// Graceful shutdown timeout in seconds
    pub graceful_shutdown_timeout: u64,

    /// Keep-alive timeout in seconds
    pub keep_alive: Option<u64>,

    /// Maximum request body size in bytes
    pub max_request_body_size: Option<usize>,

    /// Enable compression
    pub compression: bool,

    /// Compression level (1-9)
    pub compression_level: Option<u32>,

    /// TLS/SSL settings
    pub tls: Option<TlsConfig>,
}

/// TLS/SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,

    /// Path to TLS certificate file
    pub cert_path: Option<String>,

    /// Path to TLS private key file
    pub key_path: Option<String>,

    /// Certificate password (if encrypted)
    pub cert_password: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 3001,
            workers: None,
            connection_timeout: 60,
            request_timeout: 30,
            graceful_shutdown: true,
            graceful_shutdown_timeout: 30,
            keep_alive: Some(30),
            max_request_body_size: Some(10 * 1024 * 1024), // 10MB
            compression: true,
            compression_level: Some(6),
            tls: None,
        }
    }
}

impl ServerConfig {
    /// Create server config from environment variables and defaults
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host_str = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let host: IpAddr = host_str.parse()
            .map_err(|e| format!("Invalid host IP address '{}': {}", host_str, e))?;

        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3001);

        let workers = std::env::var("WORKERS")
            .ok()
            .and_then(|s| s.parse().ok());

        let compression_level = std::env::var("COMPRESSION_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&level| level >= 1 && level <= 9);

        let max_request_body_size = std::env::var("MAX_REQUEST_BODY_SIZE")
            .ok()
            .and_then(|s| s.parse().ok());

        let tls = if std::env::var("TLS_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
            Some(TlsConfig {
                enabled: true,
                cert_path: std::env::var("TLS_CERT_PATH").ok(),
                key_path: std::env::var("TLS_KEY_PATH").ok(),
                cert_password: std::env::var("TLS_CERT_PASSWORD").ok(),
            })
        } else {
            None
        };

        Ok(Self {
            host,
            port,
            workers,
            connection_timeout: std::env::var("CONNECTION_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            request_timeout: std::env::var("REQUEST_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            graceful_shutdown: std::env::var("GRACEFUL_SHUTDOWN")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            graceful_shutdown_timeout: std::env::var("GRACEFUL_SHUTDOWN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            keep_alive: std::env::var("KEEP_ALIVE")
                .ok()
                .and_then(|s| s.parse().ok()),
            max_request_body_size,
            compression: std::env::var("COMPRESSION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            compression_level,
            tls,
        })
    }

    /// Get socket address for the server
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }

    /// Get server URL string
    pub fn url(&self) -> String {
        let scheme = if self.tls.as_ref().map_or(false, |tls| tls.enabled) {
            "https"
        } else {
            "http"
        };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls.as_ref().map_or(false, |tls| tls.enabled)
    }

    /// Get compression level with default
    pub fn get_compression_level(&self) -> u32 {
        self.compression_level.unwrap_or(6)
    }

    /// Get max request body size with default
    pub fn get_max_request_body_size(&self) -> usize {
        self.max_request_body_size.unwrap_or(10 * 1024 * 1024) // 10MB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(config.port, 3001);
        assert_eq!(config.workers, None);
        assert!(config.graceful_shutdown);
        assert!(config.compression);
        assert_eq!(config.get_compression_level(), 6);
    }

    #[test]
    fn test_server_config_from_env() {
        std::env::set_var("HOST", "127.0.0.1");
        std::env::set_var("PORT", "8080");
        std::env::set_var("WORKERS", "4");
        std::env::set_var("COMPRESSION_LEVEL", "9");

        let config = ServerConfig::from_env().unwrap();
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, Some(4));
        assert_eq!(config.get_compression_level(), 9);

        std::env::remove_var("HOST");
        std::env::remove_var("PORT");
        std::env::remove_var("WORKERS");
        std::env::remove_var("COMPRESSION_LEVEL");
    }

    #[test]
    fn test_socket_addr() {
        let config = ServerConfig {
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            ..Default::default()
        };
        let addr = config.socket_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_server_url() {
        let config = ServerConfig::default();
        assert_eq!(config.url(), "http://0.0.0.0:3001");

        let tls_config = ServerConfig {
            tls: Some(TlsConfig {
                enabled: true,
                cert_path: None,
                key_path: None,
                cert_password: None,
            }),
            ..Default::default()
        };
        assert_eq!(tls_config.url(), "https://0.0.0.0:3001");
    }
}