//! HTTP Controllers
//!
//! HTTP request handlers that translate HTTP requests into application layer calls
//! and format responses for HTTP clients following REST conventions.

use crate::application::commands::{Command, CommandType};
use crate::application::queries::{Query, QueryType};
use crate::domain::entities::{SystemUser, UserSession};
use crate::shared::error::{AppError, AppResult};
use axum::{
    extract::{Path, Query as AxumQuery, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Base controller trait
pub trait BaseController {
    fn controller_name(&self) -> &'static str;
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<Uuid>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Request successful".to_string(),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pagination information
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
    pub fn new(page: u32, limit: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            page,
            limit,
            total,
            total_pages: total_pages.max(1),
            has_next,
            has_prev,
        }
    }
}

/// Query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

/// Health check controller
pub struct HealthController;

impl HealthController {
    pub fn new() -> Self {
        Self
    }

    /// Basic health check endpoint
    pub async fn basic_health() -> Result<Json<ApiResponse<()>>, AppError> {
        Ok(Json(ApiResponse::success(())))
    }

    /// Detailed health check endpoint
    pub async fn detailed_health(
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
        // In a real implementation, this would use the health monitoring service
        let health_data = serde_json::json!({
            "status": "healthy",
            "components": {
                "database": "healthy",
                "cache": "healthy"
            },
            "uptime_seconds": 3600,
            "version": "2.0.0"
        });

        Ok(Json(ApiResponse::success(health_data)))
    }
}

impl BaseController for HealthController {
    fn controller_name(&self) -> &'static str {
        "health"
    }
}

/// System user controller
pub struct SystemUserController {
    // Dependencies would be injected here
}

impl SystemUserController {
    pub fn new() -> Self {
        Self
    }

    /// List system users
    pub async fn list_users(
        Query(params): Query<PaginationQuery>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<Vec<SystemUser>>>, AppError> {
        // In a real implementation, this would use application services
        let users = vec![]; // Placeholder

        Ok(Json(ApiResponse::success(users)))
    }

    /// Get user by ID
    pub async fn get_user(
        Path(user_id): Path<Uuid>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<SystemUser>>, AppError> {
        // In a real implementation, this would use application services
        Err(AppError::NotFound("User not found".to_string()))
    }

    /// Create system user
    pub async fn create_user(
        Json(request): Json<CreateUserRequest>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<SystemUser>>, AppError> {
        // In a real implementation, this would use command handlers
        Err(AppError::BadRequest("Not implemented".to_string()))
    }

    /// Update system user
    pub async fn update_user(
        Path(user_id): Path<Uuid>,
        Json(request): Json<UpdateUserRequest>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<SystemUser>>, AppError> {
        // In a real implementation, this would use command handlers
        Err(AppError::BadRequest("Not implemented".to_string()))
    }

    /// Delete system user
    pub async fn delete_user(
        Path(user_id): Path<Uuid>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<()>>, AppError> {
        // In a real implementation, this would use command handlers
        Ok(Json(ApiResponse::success_with_message((), "User deleted successfully".to_string())))
    }

    /// User authentication
    pub async fn authenticate(
        Json(request): Json<AuthenticationRequest>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<AuthenticationResponse>>, AppError> {
        // In a real implementation, this would use authentication services
        Err(AppError::Unauthorized("Invalid credentials".to_string()))
    }

    /// User logout
    pub async fn logout(
        Extension(session): Extension<UserSession>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<()>>, AppError> {
        // In a real implementation, this would invalidate the session
        Ok(Json(ApiResponse::success_with_message((), "Logged out successfully".to_string())))
    }
}

impl BaseController for SystemUserController {
    fn controller_name(&self) -> &'static str {
        "system_users"
    }
}

/// Configuration controller
pub struct ConfigurationController {
    // Dependencies would be injected here
}

impl ConfigurationController {
    pub fn new() -> Self {
        Self
    }

    /// Get configuration
    pub async fn get_configuration(
        Query(params): Query<ConfigurationQuery>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<HashMap<String, serde_json::Value>>>, AppError> {
        // In a real implementation, this would use configuration services
        let config = HashMap::new(); // Placeholder

        Ok(Json(ApiResponse::success(config)))
    }

    /// Update configuration
    pub async fn update_configuration(
        Json(request): Json<UpdateConfigurationRequest>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<ApiResponse<()>>, AppError> {
        // In a real implementation, this would use command handlers
        Ok(Json(ApiResponse::success_with_message((), "Configuration updated successfully".to_string())))
    }
}

impl BaseController for ConfigurationController {
    fn controller_name(&self) -> &'static str {
        "configuration"
    }
}

/// Audit log controller
pub struct AuditLogController {
    // Dependencies would be injected here
}

impl AuditLogController {
    pub fn new() -> Self {
        Self
    }

    /// Get audit logs
    pub async fn get_audit_logs(
        Query(params): Query<AuditLogQuery>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<PaginatedResponse<serde_json::Value>>, AppError> {
        // In a real implementation, this would use audit services
        let logs = vec![]; // Placeholder
        let pagination = PaginationInfo::new(params.page, params.limit, 0);

        Ok(Json(PaginatedResponse {
            success: true,
            data: logs,
            pagination,
            message: "Audit logs retrieved successfully".to_string(),
            timestamp: chrono::Utc::now(),
        }))
    }

    /// Search audit logs
    pub async fn search_audit_logs(
        Json(request): Json<SearchAuditLogRequest>,
        State(app_state): State<crate::shared::AppState>,
    ) -> Result<Json<PaginatedResponse<serde_json::Value>>, AppError> {
        // In a real implementation, this would use audit services
        let logs = vec![]; // Placeholder
        let pagination = PaginationInfo::new(request.page, request.limit, 0);

        Ok(Json(PaginatedResponse {
            success: true,
            data: logs,
            pagination,
            message: "Audit logs search completed".to_string(),
            timestamp: chrono::Utc::now(),
        }))
    }
}

impl BaseController for AuditLogController {
    fn controller_name(&self) -> &'static str {
        "audit_logs"
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticationRequest {
    pub email_or_username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AuthenticationResponse {
    pub user: SystemUser,
    pub session: UserSession,
    pub permissions: Vec<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigurationQuery {
    pub category: Option<String>,
    pub include_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigurationRequest {
    pub updates: HashMap<String, serde_json::Value>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub module: Option<String>,
    pub resource_type: Option<String>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchAuditLogRequest {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub query: String,
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// Error response for HTTP errors
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetails,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl From<AppError> for ErrorResponse {
    fn from(error: AppError) -> Self {
        let (code, message) = match &error {
            AppError::ValidationError(msg) => ("VALIDATION_ERROR", msg.as_str()),
            AppError::NotFound(msg) => ("NOT_FOUND", msg.as_str()),
            AppError::Unauthorized(msg) => ("UNAUTHORIZED", msg.as_str()),
            AppError::Forbidden(msg) => ("FORBIDDEN", msg.as_str()),
            AppError::Conflict(msg) => ("CONFLICT", msg.as_str()),
            AppError::BadRequest(msg) => ("BAD_REQUEST", msg.as_str()),
            AppError::InternalServerError(msg) => ("INTERNAL_SERVER_ERROR", msg.as_str()),
            _ => ("UNKNOWN_ERROR", "An unknown error occurred"),
        };

        Self {
            success: false,
            error: ErrorDetails {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            },
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.message, "Request successful");
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(1, 20, 100);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.limit, 20);
        assert_eq!(pagination.total, 100);
        assert_eq!(pagination.total_pages, 5);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);
    }

    #[test]
    fn test_error_response() {
        let app_error = AppError::NotFound("User not found".to_string());
        let error_response = ErrorResponse::from(app_error);

        assert!(!error_response.success);
        assert_eq!(error_response.error.code, "NOT_FOUND");
        assert_eq!(error_response.error.message, "User not found");
    }

    #[test]
    fn test_create_user_request() {
        let request = CreateUserRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            permissions: vec!["read".to_string()],
            is_active: Some(true),
        };

        assert_eq!(request.name, "Test User");
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.permissions.len(), 1);
        assert_eq!(request.is_active, Some(true));
    }

    #[test]
    fn test_authentication_request() {
        let request = AuthenticationRequest {
            email_or_username: "test@example.com".to_string(),
            password: "password123".to_string(),
            remember_me: Some(false),
        };

        assert_eq!(request.email_or_username, "test@example.com");
        assert_eq!(request.password, "password123");
        assert_eq!(request.remember_me, Some(false));
    }
}