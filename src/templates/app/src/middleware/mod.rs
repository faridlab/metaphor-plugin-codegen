//! Application middleware

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use tower::Layer;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// Create application middleware stack
pub fn create_middleware_stack() -> impl tower::Layer<axum::routing::Router> {
    TraceLayer::new_for_http()
}

/// CORS middleware configuration
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
}

/// Request logging middleware
pub async fn request_logger(
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    info!("📥 Incoming request: {} {}", method, uri);

    let response = next.run(req).await;

    let status = response.status();
    info!("📤 Outgoing response: {} {} {}", method, uri, status);

    response
}

/// Rate limiting middleware placeholder
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    // TODO: Implement actual rate limiting
    // For now, just pass through
    next.run(req).await
}