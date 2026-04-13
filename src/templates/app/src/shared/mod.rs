//! Shared application state and utilities

use metaphor_health::HealthChecker;
use crate::config::AppConfig;
use sqlx::PgPool;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: PgPool,
    pub health_checker: HealthChecker,
}

impl AppState {
    /// Create new application state
    pub fn new(config: AppConfig, db_pool: PgPool, health_checker: HealthChecker) -> Self {
        Self {
            config,
            db_pool,
            health_checker,
        }
    }
}

/// Shared response utilities
pub mod response {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde_json::{json, Value};
    use tracing::debug;

    /// Create a success response
    pub fn success<T: serde::Serialize>(data: T) -> Json<Value> {
        debug!("Creating success response");
        Json(json!({
            "success": true,
            "data": data
        }))
    }

    /// Create an error response
    pub fn error(message: &str, status: StatusCode) -> Response {
        debug!("Creating error response: {} - {}", status, message);
        (
            status,
            Json(json!({
                "success": false,
                "error": {
                    "message": message,
                    "code": status.as_u16()
                }
            })),
        )
            .into_response()
    }

    /// Create a validation error response
    pub fn validation_error(errors: Vec<&str>) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": {
                    "message": "Validation failed",
                    "code": 400,
                    "details": errors
                }
            })),
        )
            .into_response()
    }
}

/// Shared pagination utilities
pub mod pagination {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Pagination parameters
    #[derive(Debug, Deserialize)]
    pub struct PaginationParams {
        pub page: Option<u32>,
        pub limit: Option<u32>,
        pub offset: Option<u32>,
        pub sort_by: Option<String>,
        pub sort_order: Option<String>,
    }

    impl Default for PaginationParams {
        fn default() -> Self {
            Self {
                page: Some(1),
                limit: Some(20),
                offset: Some(0),
                sort_by: None,
                sort_order: Some("asc".to_string()),
            }
        }
    }

    impl PaginationParams {
        /// Get page number (default: 1)
        pub fn page(&self) -> u32 {
            self.page.unwrap_or(1).max(1)
        }

        /// Get limit (default: 20, max: 100)
        pub fn limit(&self) -> u32 {
            self.limit.unwrap_or(20).min(100).max(1)
        }

        /// Get offset calculated from page
        pub fn offset(&self) -> u32 {
            self.offset
                .unwrap_or((self.page() - 1) * self.limit())
                .max(0)
        }

        /// Get sort field (default: "created_at")
        pub fn sort_by(&self) -> String {
            self.sort_by
                .clone()
                .unwrap_or_else(|| "created_at".to_string())
        }

        /// Get sort order (default: "asc")
        pub fn sort_order(&self) -> String {
            let order = self
                .sort_order
                .clone()
                .unwrap_or_else(|| "asc".to_string())
                .to_lowercase();
            if order == "desc" {
                "desc".to_string()
            } else {
                "asc".to_string()
            }
        }

        /// Convert to SQL ORDER BY clause
        pub fn to_order_by(&self) -> String {
            format!("{} {}", self.sort_by(), self.sort_order())
        }
    }

    /// Paginated response wrapper
    #[derive(Debug, Serialize)]
    pub struct PaginatedResponse<T: Serialize> {
        pub data: Vec<T>,
        pub pagination: PaginationInfo,
    }

    /// Pagination metadata
    #[derive(Debug, Serialize)]
    pub struct PaginationInfo {
        pub page: u32,
        pub limit: u32,
        pub total: u64,
        pub total_pages: u32,
        pub has_next: bool,
        pub has_prev: bool,
    }

    impl PaginationInfo {
        /// Create pagination info
        pub fn new(page: u32, limit: u32, total: u64) -> Self {
            let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
            let has_next = page < total_pages;
            let has_prev = page > 1;

            Self {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            }
        }
    }

    /// Create paginated response
    pub fn create_response<T: Serialize>(
        data: Vec<T>,
        page: u32,
        limit: u32,
        total: u64,
    ) -> PaginatedResponse<T> {
        PaginatedResponse {
            data,
            pagination: PaginationInfo::new(page, limit, total),
        }
    }
}

/// Shared error handling
pub mod error {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde_json::json;
    use thiserror::Error;

    /// Application error type
    #[derive(Error, Debug)]
    pub enum AppError {
        #[error("Database error: {0}")]
        Database(#[from] sqlx::Error),

        #[error("Configuration error: {0}")]
        Config(String),

        #[error("Validation error: {0}")]
        Validation(String),

        #[error("Not found: {0}")]
        NotFound(String),

        #[error("Unauthorized: {0}")]
        Unauthorized(String),

        #[error("Forbidden: {0}")]
        Forbidden(String),

        #[error("Conflict: {0}")]
        Conflict(String),

        #[error("Serialization error: {0}")]
        Serialization(String),

        #[error("Internal server error: {0}")]
        Internal(#[from] anyhow::Error),
    }

    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, error_message) = match self {
                AppError::Database(ref err) => {
                    tracing::error!("Database error: {:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database operation failed")
                }
                AppError::Config(ref msg) => {
                    tracing::error!("Configuration error: {}", msg);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
                }
                AppError::Validation(ref msg) => {
                    (StatusCode::BAD_REQUEST, msg.as_str())
                }
                AppError::NotFound(ref msg) => {
                    (StatusCode::NOT_FOUND, msg.as_str())
                }
                AppError::Unauthorized(ref msg) => {
                    (StatusCode::UNAUTHORIZED, msg.as_str())
                }
                AppError::Forbidden(ref msg) => {
                    (StatusCode::FORBIDDEN, msg.as_str())
                }
                AppError::Conflict(ref msg) => {
                    (StatusCode::CONFLICT, msg.as_str())
                }
                AppError::Serialization(ref msg) => {
                    tracing::error!("Serialization error: {}", msg);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Data processing error")
                }
                AppError::Internal(ref err) => {
                    tracing::error!("Internal error: {:?}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                }
            };

            let body = Json(json!({
                "success": false,
                "error": {
                    "message": error_message,
                    "code": status.as_u16()
                }
            }));

            (status, body).into_response()
        }
    }

    /// Result type alias for application operations
    pub type AppResult<T> = Result<T, AppError>;
}

#[cfg(test)]
mod tests {
    use super::pagination::*;

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.page(), 1);
        assert_eq!(params.limit(), 20);
        assert_eq!(params.offset(), 0);
        assert_eq!(params.sort_by(), "created_at");
        assert_eq!(params.sort_order(), "asc");
    }

    #[test]
    fn test_pagination_params_custom() {
        let params = PaginationParams {
            page: Some(2),
            limit: Some(50),
            offset: Some(25),
            sort_by: Some("name".to_string()),
            sort_order: Some("desc".to_string()),
        };

        assert_eq!(params.page(), 2);
        assert_eq!(params.limit(), 50);
        assert_eq!(params.offset(), 25); // Uses explicit offset
        assert_eq!(params.sort_by(), "name");
        assert_eq!(params.sort_order(), "desc");
    }

    #[test]
    fn test_pagination_limits() {
        let params = PaginationParams {
            page: Some(0), // Should be clamped to 1
            limit: Some(150), // Should be clamped to 100
            offset: Some(-10), // Should be clamped to 0
            sort_by: None,
            sort_order: Some("INVALID".to_string()),
        };

        assert_eq!(params.page(), 1);
        assert_eq!(params.limit(), 100);
        assert_eq!(params.offset(), 0);
        assert_eq!(params.sort_by(), "created_at");
        assert_eq!(params.sort_order(), "asc"); // Invalid becomes asc
    }

    #[test]
    fn test_pagination_info() {
        let info = pagination::PaginationInfo::new(2, 20, 55);
        assert_eq!(info.page, 2);
        assert_eq!(info.limit, 20);
        assert_eq!(info.total, 55);
        assert_eq!(info.total_pages, 3); // ceil(55/20) = 3
        assert!(info.has_next);
        assert!(info.has_prev);
    }
}