//! HTTP Middleware
//!
//! HTTP middleware components for request/response processing, authentication,
// /authorization, logging, rate limiting, and other cross-cutting concerns.

use crate::domain::entities::SystemUser;
use crate::domain::services::{UserSessionService, AuditService};
use crate::shared::error::{AppError, AppResult};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

/// CORS middleware configuration
pub fn cors_middleware() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(true)
        .expose_headers(["x-request-id", "x-total-count"])
}

/// Request tracing middleware
pub fn tracing_middleware() -> TraceLayer {
    TraceLayer::new_for_http()
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let mut response = next.run(request).await;

    // Add security headers
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());

    Ok(response)
}

/// Request ID middleware
pub async fn request_id_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert("x-request-id", request_id.to_string().parse().unwrap());

    Ok(response)
}

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start_time.elapsed();
    let status = response.status();

    // Log request
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = duration.as_millis(),
        user_agent = user_agent,
        "HTTP request processed"
    );

    Ok(response)
}

/// Authentication middleware
pub struct AuthenticationMiddleware {
    user_session_service: Arc<UserSessionService>,
    audit_service: Option<Arc<AuditService>>,
}

impl AuthenticationMiddleware {
    pub fn new(
        user_session_service: Arc<UserSessionService>,
        audit_service: Option<Arc<AuditService>>,
    ) -> Self {
        Self {
            user_session_service,
            audit_service,
        }
    }
}

#[axum::async_trait]
impl<S> tower::Layer<S> for AuthenticationMiddleware
where
    S: tower::Service<Request, Response = Response> + Send + 'static + Sync,
    S::Future: Send,
{
    type Service = AuthenticationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationService {
            inner,
            user_session_service: self.user_session_service.clone(),
            audit_service: self.audit_service.clone(),
        }
    }
}

/// Authentication service
pub struct AuthenticationService<S> {
    inner: S,
    user_session_service: Arc<UserSessionService>,
    audit_service: Option<Arc<AuditService>>,
}

impl<S> tower::Service<Request> for AuthenticationService<S>
where
    S: tower::Service<Request, Response = Response> + Send + 'static + Sync,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let user_session_service = self.user_session_service.clone();
        let audit_service = self.audit_service.clone();

        Box::pin(async move {
            // Skip authentication for certain paths
            let path = request.uri().path();
            if should_skip_authentication(path) {
                return self.inner.call(request).await;
            }

            // Extract authorization header
            let auth_header = request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok());

            if let Some(auth_header) = auth_header {
                if let Some(token) = extract_bearer_token(auth_header) {
                    // Validate session token
                    match user_session_service.validate_session(&token).await {
                        Ok(Some(session)) => {
                            // Load user and attach to request
                            if let Ok(Some(user)) = user_session_service.get_user_by_id(&session.user_id).await {
                                // Attach user and session to request extensions
                                let mut request_with_extensions = request;
                                request_with_extensions.extensions_mut().insert(user);
                                request_with_extensions.extensions_mut().insert(session);

                                // Log authentication event
                                if let Some(audit_service) = &audit_service {
                                    let _ = audit_service.log_authentication_event(
                                        user.id,
                                        crate::domain::services::AuthAction::Login,
                                        serde_json::json!({"method": "token"}),
                                        crate::domain::services::AuditMetadata {
                                            ip_address: extract_client_ip(&request_with_extensions),
                                            user_agent: extract_user_agent(&request_with_extensions),
                                            module: "authentication".to_string(),
                                        }
                                    ).await;
                                }

                                return self.inner.call(request_with_extensions).await;
                            }
                        }
                        Ok(None) => {
                            // Session not found
                            tracing::warn!("Invalid session token provided");
                        }
                        Err(e) => {
                            tracing::error!("Session validation error: {:?}", e);
                        }
                    }
                }
            }

            // Authentication failed - return unauthorized response
            let error_response = axum::Json(json!({
                "success": false,
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": "Authentication required"
                },
                "timestamp": chrono::Utc::now()
            }));

            let response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap();

            Ok(response)
        })
    }
}

/// Authorization middleware
pub async fn authorization_middleware(
    Extension(user): Extension<SystemUser>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let method = request.method();

    // Check if user has required permissions for this endpoint
    if has_required_permission(&user, path, method) {
        Ok(next.run(request).await)
    } else {
        Err(AppError::Forbidden("Insufficient permissions".to_string()))
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    requests_per_minute: u32,
    client_buckets: Arc<tokio::sync::RwLock<HashMap<String, (Instant, u32)>>>,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            client_buckets: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    async fn is_rate_limited(&self, client_id: &str) -> bool {
        let mut buckets = self.client_buckets.write().await;
        let now = Instant::now();

        match buckets.get_mut(client_id) {
            Some((last_reset, count)) => {
                // Reset if more than a minute has passed
                if now.duration_since(*last_reset) > Duration::from_secs(60) {
                    *last_reset = now;
                    *count = 1;
                    false
                } else {
                    *count += 1;
                    *count > self.requests_per_minute
                }
            }
            None => {
                buckets.insert(client_id.to_string(), (now, 1));
                false
            }
        }
    }
}

#[axum::async_trait]
impl<S> tower::Layer<S> for RateLimitMiddleware
where
    S: tower::Service<Request, Response = Response> + Send + 'static + Sync,
    S::Future: Send,
{
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            requests_per_minute: self.requests_per_minute,
            client_buckets: self.client_buckets.clone(),
        }
    }
}

/// Rate limiting service
pub struct RateLimitService<S> {
    inner: S,
    requests_per_minute: u32,
    client_buckets: Arc<tokio::sync::RwLock<HashMap<String, (Instant, u32)>>>,
}

impl<S> tower::Service<Request> for RateLimitService<S>
where
    S: tower::Service<Request, Response = Response> + Send + 'static + Sync,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let client_buckets = self.client_buckets.clone();
        let requests_per_minute = self.requests_per_minute;

        Box::pin(async move {
            // Extract client IP for rate limiting
            let client_id = extract_client_ip(&request)
                .unwrap_or_else(|| "unknown".to_string());

            // Check rate limit
            let is_limited = {
                let mut buckets = client_buckets.write().await;
                let now = Instant::now();

                match buckets.get_mut(&client_id) {
                    Some((last_reset, count)) => {
                        if now.duration_since(*last_reset) > Duration::from_secs(60) {
                            *last_reset = now;
                            *count = 1;
                            false
                        } else {
                            *count += 1;
                            *count > requests_per_minute
                        }
                    }
                    None => {
                        buckets.insert(client_id, (now, 1));
                        false
                    }
                }
            };

            if is_limited {
                // Rate limit exceeded
                let error_response = axum::Json(json!({
                    "success": false,
                    "error": {
                        "code": "RATE_LIMIT_EXCEEDED",
                        "message": "Too many requests"
                    },
                    "timestamp": chrono::Utc::now()
                }));

                let response = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                    .unwrap();

                Ok(response)
            } else {
                self.inner.call(request).await
            }
        })
    }
}

// Helper functions

fn should_skip_authentication(path: &str) -> bool {
    path.starts_with("/health") ||
    path.starts_with("/metrics") ||
    path.starts_with("/api/v1/auth/login") ||
    path.starts_with("/api/v1/auth/register") ||
    path == "/" ||
    path.starts_with("/docs")
}

fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

fn extract_client_ip(request: &Request) -> Option<String> {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|addr| addr.0.ip().to_string())
        })
}

fn extract_user_agent(request: &Request) -> String {
    request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn has_required_permission(user: &SystemUser, path: &str, method: &Method) -> bool {
    // Simple permission check - in a real implementation, this would be more sophisticated
    if user.permissions.contains(&"admin".to_string()) {
        return true;
    }

    // Check read access for GET requests
    if *method == Method::GET {
        return user.permissions.contains(&"read".to_string());
    }

    // Check write access for other methods
    user.permissions.contains(&"write".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_authentication() {
        assert!(should_skip_authentication("/health"));
        assert!(should_skip_authentication("/health/detailed"));
        assert!(should_skip_authentication("/api/v1/auth/login"));
        assert!(should_skip_authentication("/docs"));
        assert!(!should_skip_authentication("/api/v1/users"));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_bearer_token("Basic abc123"),
            None
        );
        assert_eq!(
            extract_bearer_token("abc123"),
            None
        );
    }

    #[test]
    fn test_has_required_permission() {
        let admin_user = SystemUser {
            id: Uuid::new_v4(),
            name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            permissions: vec!["admin".to_string()],
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login_at: None,
            created_by: None,
        };

        let read_user = SystemUser {
            id: Uuid::new_v4(),
            name: "Reader".to_string(),
            email: "reader@example.com".to_string(),
            permissions: vec!["read".to_string()],
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login_at: None,
            created_by: None,
        };

        // Admin can access anything
        assert!(has_required_permission(&admin_user, "/api/v1/users", &Method::POST));
        assert!(has_required_permission(&admin_user, "/api/v1/users", &Method::GET));

        // Read-only user can only read
        assert!(!has_required_permission(&read_user, "/api/v1/users", &Method::POST));
        assert!(has_required_permission(&read_user, "/api/v1/users", &Method::GET));
    }
}