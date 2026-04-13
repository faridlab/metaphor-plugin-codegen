//! Logging middleware

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, error, info, warn, Level};
use uuid::Uuid;

/// Request ID extension
pub struct RequestId(pub String);

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Enable request logging
    pub enable_request_logging: bool,

    /// Enable response logging
    pub enable_response_logging: bool,

    /// Log request body
    pub log_request_body: bool,

    /// Log response body
    pub log_response_body: bool,

    /// Log headers
    pub log_headers: bool,

    /// Skip logging for health endpoints
    pub skip_health_endpoints: bool,

    /// Skip logging for static assets
    pub skip_static_assets: bool,

    /// Slow request threshold in milliseconds
    pub slow_request_threshold: u64,

    /// Logging level for requests
    pub request_level: Level,

    /// Logging level for responses
    pub response_level: Level,

    /// Logging level for slow requests
    pub slow_request_level: Level,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_request_logging: true,
            enable_response_logging: true,
            log_request_body: false,
            log_response_body: false,
            log_headers: false,
            skip_health_endpoints: true,
            skip_static_assets: true,
            slow_request_threshold: 1000, // 1 second
            request_level: Level::INFO,
            response_level: Level::INFO,
            slow_request_level: Level::WARN,
        }
    }
}

impl LoggingConfig {
    /// Create logging config from environment variables
    pub fn from_env() -> Self {
        Self {
            enable_request_logging: std::env::var("LOG_REQUESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            enable_response_logging: std::env::var("LOG_RESPONSES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            log_request_body: std::env::var("LOG_REQUEST_BODY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            log_response_body: std::env::var("LOG_RESPONSE_BODY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            log_headers: std::env::var("LOG_HEADERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            skip_health_endpoints: std::env::var("LOG_SKIP_HEALTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            skip_static_assets: std::env::var("LOG_SKIP_STATIC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            slow_request_threshold: std::env::var("LOG_SLOW_REQUEST_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            request_level: std::env::var("LOG_REQUEST_LEVEL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Level::INFO),
            response_level: std::env::var("LOG_RESPONSE_LEVEL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Level::INFO),
            slow_request_level: std::env::var("LOG_SLOW_REQUEST_LEVEL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Level::WARN),
        }
    }
}

/// Comprehensive request logging middleware
pub async fn request_logging_middleware(
    State(config): State<LoggingConfig>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to extensions
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // Check if we should skip logging this request
    let should_skip = should_skip_logging(&req, &config);

    if !should_skip && config.enable_request_logging {
        log_request(&req, &config, &request_id);
    }

    let response = next.run(req).await;

    let duration = start_time.elapsed();

    if !should_skip && config.enable_response_logging {
        log_response(&response, &config, &request_id, duration);
    }

    // Log slow requests
    if duration.as_millis() as u64 > config.slow_request_threshold {
        log_slow_request(&request_id, duration);
    }

    response
}

/// Simple request logging middleware
pub async fn simple_request_logger(
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    info!("📥 {} {}", method, uri);

    let response = next.run(req).await;

    let status = response.status();
    info!("📤 {} {} {}", method, uri, status);

    response
}

/// Enhanced request logging with detailed information
pub async fn detailed_request_logger(
    req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    let method = req.method();
    let uri = req.uri();
    let version = req.version();

    // Log request details
    info!(
        "📥 [{}] {} {} {:?}",
        request_id,
        method,
        uri,
        version
    );

    // Log user agent if available
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(user_agent_str) = user_agent.to_str() {
            debug!("📥 [{}] User-Agent: {}", request_id, user_agent_str);
        }
    }

    // Log content length if available
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            debug!("📥 [{}] Content-Length: {}", request_id, length_str);
        }
    }

    let response = next.run(req).await;
    let duration = start_time.elapsed();
    let status = response.status();

    // Log response details
    info!(
        "📤 [{}] {} {} {:?} ({:.2?})",
        request_id,
        method,
        uri,
        status,
        duration
    );

    // Log if response was an error
    if status.is_server_error() {
        error!(
            "❌ [{}] Server error: {} {} {:?}",
            request_id,
            method,
            uri,
            status
        );
    } else if status.is_client_error() {
        warn!(
            "⚠️  [{}] Client error: {} {} {:?}",
            request_id,
            method,
            uri,
            status
        );
    }

    response
}

/// Check if request should be skipped based on configuration
fn should_skip_logging(req: &Request<Body>, config: &LoggingConfig) -> bool {
    let path = req.uri().path();

    // Skip health endpoints
    if config.skip_health_endpoints && (path.starts_with("/health") || path.starts_with("/ping")) {
        return true;
    }

    // Skip static assets
    if config.skip_static_assets && is_static_asset(path) {
        return true;
    }

    false
}

/// Check if path is for static asset
fn is_static_asset(path: &str) -> bool {
    path.starts_with("/static/") ||
    path.starts_with("/assets/") ||
    path.starts_with("/css/") ||
    path.starts_with("/js/") ||
    path.starts_with("/images/") ||
    path.starts_with("/fonts/") ||
    path.ends_with(".css") ||
    path.ends_with(".js") ||
    path.ends_with(".ico") ||
    path.ends_with(".png") ||
    path.ends_with(".jpg") ||
    path.ends_with(".jpeg") ||
    path.ends_with(".gif") ||
    path.ends_with(".svg") ||
    path.ends_with(".woff") ||
    path.ends_with(".woff2")
}

/// Log request details
fn log_request(req: &Request<Body>, config: &LoggingConfig, request_id: &str) {
    let method = req.method();
    let uri = req.uri();

    match config.request_level {
        Level::TRACE => {
            tracing::trace!(
                "📥 [{}] {} {} {:?}",
                request_id,
                method,
                uri,
                req.version()
            );

            if config.log_headers {
                for (name, value) in req.headers() {
                    if let Ok(value_str) = value.to_str() {
                        tracing::trace!("📥 [{}] {}: {}", request_id, name, value_str);
                    }
                }
            }
        }
        Level::DEBUG => {
            tracing::debug!(
                "📥 [{}] {} {} {:?}",
                request_id,
                method,
                uri,
                req.version()
            );

            if config.log_headers {
                if let Some(user_agent) = req.headers().get("user-agent") {
                    if let Ok(ua_str) = user_agent.to_str() {
                        tracing::debug!("📥 [{}] User-Agent: {}", request_id, ua_str);
                    }
                }
            }
        }
        Level::INFO => {
            tracing::info!("📥 [{}] {} {}", request_id, method, uri);
        }
        _ => {}
    }
}

/// Log response details
fn log_response(response: &Response, config: &LoggingConfig, request_id: &str, duration: std::time::Duration) {
    let status = response.status();

    match config.response_level {
        Level::TRACE => {
            tracing::trace!(
                "📤 [{}] {} ({:.2?})",
                request_id,
                status,
                duration
            );

            if config.log_headers {
                for (name, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        tracing::trace!("📤 [{}] {}: {}", request_id, name, value_str);
                    }
                }
            }
        }
        Level::DEBUG => {
            tracing::debug!(
                "📤 [{}] {} ({:.2?})",
                request_id,
                status,
                duration
            );
        }
        Level::INFO => {
            tracing::info!("📤 [{}] {} ({:.2?})", request_id, status, duration);
        }
        _ => {}
    }
}

/// Log slow requests
fn log_slow_request(request_id: &str, duration: std::time::Duration) {
    tracing::warn!(
        "⏱️  [{}] Slow request detected: {:.2?}",
        request_id,
        duration
    );
}

/// Extension trait to extract request ID
pub trait RequestIdExt {
    fn request_id(&self) -> Option<&str>;
}

impl RequestIdExt for Request<Body> {
    fn request_id(&self) -> Option<&str> {
        self.extensions().get::<RequestId>().map(|id| id.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, Version},
    };

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(config.enable_request_logging);
        assert!(config.enable_response_logging);
        assert!(!config.log_request_body);
        assert!(!config.log_response_body);
        assert_eq!(config.slow_request_threshold, 1000);
    }

    #[test]
    fn test_logging_config_from_env() {
        std::env::set_var("LOG_REQUESTS", "false");
        std::env::set_var("LOG_RESPONSE_BODY", "true");
        std::env::set_var("LOG_SLOW_REQUEST_THRESHOLD", "500");

        let config = LoggingConfig::from_env();
        assert!(!config.enable_request_logging);
        assert!(config.log_response_body);
        assert_eq!(config.slow_request_threshold, 500);

        std::env::remove_var("LOG_REQUESTS");
        std::env::remove_var("LOG_RESPONSE_BODY");
        std::env::remove_var("LOG_SLOW_REQUEST_THRESHOLD");
    }

    #[test]
    fn test_is_static_asset() {
        assert!(is_static_asset("/static/css/style.css"));
        assert!(is_static_asset("/assets/js/app.js"));
        assert!(is_static_asset("/image.png"));
        assert!(is_static_asset("/favicon.ico"));

        assert!(!is_static_asset("/api/users"));
        assert!(!is_static_asset("/health"));
        assert!(!is_static_asset("/login"));
    }

    #[test]
    fn test_request_id_ext() {
        let mut req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .version(Version::HTTP_11)
            .body(Body::empty())
            .unwrap();

        assert_eq!(req.request_id(), None);

        let request_id = "test-request-id".to_string();
        req.extensions_mut().insert(RequestId(request_id.clone()));

        assert_eq!(req.request_id(), Some("test-request-id"));
    }
}