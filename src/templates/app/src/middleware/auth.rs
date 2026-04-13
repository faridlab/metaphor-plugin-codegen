//! Authentication middleware

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
usejsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::shared::error::AppError;

/// JWT Claims structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
    pub roles: Vec<String>, // User roles
    pub email: String,      // User email
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub issuer: String,
    pub audience: String,
    pub header_name: String,
    pub token_prefix: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key".to_string()),
            issuer: std::env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "metaphor-framework".to_string()),
            audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "metaphor-users".to_string()),
            header_name: "Authorization".to_string(),
            token_prefix: "Bearer".to_string(),
        }
    }
}

/// Authentication middleware state
#[derive(Debug, Clone)]
pub struct AuthState {
    pub config: AuthConfig,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            config: AuthConfig::default(),
        }
    }

    pub fn with_config(config: AuthConfig) -> Self {
        Self { config }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    debug!("🔐 Checking authentication");

    // Extract token from headers
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) => {
            if header.starts_with(&format!("{} ", auth_state.config.token_prefix)) {
                Some(&header[auth_state.config.token_prefix.len() + 1..])
            } else {
                None
            }
        }
        None => None,
    };

    let token = match token {
        Some(t) => t,
        None => {
            warn!("❌ Missing or invalid authorization header");
            return create_error_response(StatusCode::UNAUTHORIZED, "Missing or invalid authorization token");
        }
    };

    // Validate JWT token
    let claims = match validate_jwt_token(token, &auth_state.config) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("❌ Invalid JWT token: {}", e);
            return create_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token");
        }
    };

    // Add user context to request extensions
    req.extensions_mut().insert(claims);

    debug!("✅ Authentication successful");
    Ok(next.run(req).await)
}

/// Optional authentication middleware (doesn't return error if no token)
pub async fn optional_auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    debug!("🔐 Checking optional authentication");

    // Extract token from headers
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(header) = auth_header {
        if header.starts_with(&format!("{} ", auth_state.config.token_prefix)) {
            let token = &header[auth_state.config.token_prefix.len() + 1..];

            if let Ok(claims) = validate_jwt_token(token, &auth_state.config) {
                req.extensions_mut().insert(claims);
                debug!("✅ Optional authentication successful");
            } else {
                debug!("⚠️ Optional authentication failed, continuing without auth");
            }
        }
    }

    next.run(req).await
}

/// Role-based authorization middleware
pub async fn require_role_middleware(
    required_roles: Vec<String>,
) -> impl Fn(Request<Body>, Next) -> Result<Response, StatusCode> {
    move |req: Request<Body>, next: Next| {
        let required_roles = required_roles.clone();
        async move {
            let claims = req.extensions().get::<Claims>();

            match claims {
                Some(claims) => {
                    let user_has_role = required_roles.iter().any(|role| claims.roles.contains(role));

                    if user_has_role {
                        debug!("✅ User has required roles: {:?}", required_roles);
                        Ok(next.run(req).await)
                    } else {
                        warn!(
                            "❌ User {} lacks required roles. Has: {:?}, Needs: {:?}",
                            claims.sub, claims.roles, required_roles
                        );
                        create_error_response(StatusCode::FORBIDDEN, "Insufficient permissions")
                    }
                }
                None => {
                    warn!("❌ No authentication found for role-based access");
                    create_error_response(StatusCode::UNAUTHORIZED, "Authentication required")
                }
            }
        }
    }
}

/// Validate JWT token and extract claims
fn validate_jwt_token(token: &str, config: &AuthConfig) -> Result<Claims, AppError> {
    let header = decode_header(token)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token header: {}", e)))?;

    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.audience]);
    validation.validate_exp = true;
    validation.validate_iat = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

/// Create standardized error response
fn create_error_response(status: StatusCode, message: &str) -> Result<Response, StatusCode> {
    let body = axum::Json(json!({
        "success": false,
        "error": {
            "message": message,
            "code": status.as_u16()
        }
    }));

    Ok((status, body).into_response())
}

/// Extension trait to extract user claims from request
pub trait RequestClaimsExt {
    fn user_claims(&self) -> Option<&Claims>;
    fn user_id(&self) -> Option<&str>;
    fn user_email(&self) -> Option<&str>;
    fn user_roles(&self) -> Option<&Vec<String>>;
    fn has_role(&self, role: &str) -> bool;
    fn has_any_role(&self, roles: &[&str]) -> bool;
}

impl RequestClaimsExt for Request<Body> {
    fn user_claims(&self) -> Option<&Claims> {
        self.extensions().get::<Claims>()
    }

    fn user_id(&self) -> Option<&str> {
        self.user_claims().map(|claims| claims.sub.as_str())
    }

    fn user_email(&self) -> Option<&str> {
        self.user_claims().map(|claims| claims.email.as_str())
    }

    fn user_roles(&self) -> Option<&Vec<String>> {
        self.user_claims().map(|claims| &claims.roles)
    }

    fn has_role(&self, role: &str) -> bool {
        self.user_roles()
            .map_or(false, |roles| roles.contains(&role.to_string()))
    }

    fn has_any_role(&self, roles: &[&str]) -> bool {
        self.user_roles()
            .map_or(false, |user_roles| {
                roles.iter().any(|role| user_roles.contains(&role.to_string()))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::Request,
        http::{header, Method},
    };

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert_eq!(config.issuer, "metaphor-framework");
        assert_eq!(config.audience, "metaphor-users");
        assert_eq!(config.header_name, "Authorization");
        assert_eq!(config.token_prefix, "Bearer");
    }

    #[test]
    fn test_request_claims_ext() {
        let claims = Claims {
            sub: "user123".to_string(),
            exp: 1234567890,
            iat: 1234567880,
            iss: "metaphor-framework".to_string(),
            aud: "metaphor-users".to_string(),
            roles: vec!["user".to_string(), "admin".to_string()],
            email: "user@example.com".to_string(),
        };

        let mut req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        req.extensions_mut().insert(claims);

        assert_eq!(req.user_id(), Some("user123"));
        assert_eq!(req.user_email(), Some("user@example.com"));
        assert!(req.has_role("user"));
        assert!(req.has_role("admin"));
        assert!(!req.has_role("superuser"));
        assert!(req.has_any_role(&["user", "moderator"]));
        assert!(!req.has_any_role(&["superuser", "moderator"]));
    }
}