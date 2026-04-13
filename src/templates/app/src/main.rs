//! Metaphor Framework Application Entry Point
//! Laravel-inspired monolith orchestrator for DDD bounded contexts
//!
//! This application follows Clean Architecture principles with:
//! - Domain Layer: Business rules and entities
//! - Application Layer: Use cases and orchestration
//! - Infrastructure Layer: Technical implementations
//! - Presentation Layer: HTTP/gRPC/CLI interfaces

use anyhow::Result;
use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use metaphor_health::HealthChecker;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};
use sqlx::PgPool;

// Clean Architecture Layers
mod configuration;
mod domain;
mod application;
mod infrastructure;
mod presentation;

// Legacy modules (will be refactored)
mod config;
mod shared;

use configuration::AppConfig;
use infrastructure::database::DatabaseManager;

/// Main application entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load()?;

    // Initialize logging
    init_tracing(&config)?;

    info!("🚀 Starting Metaphor Framework Application");
    info!("📋 Environment: {}", config.logging.tracing.environment);
    info!("🌐 Server: {}:{}", config.server.host, config.server.port);

    // Initialize database connection pool
    let db_manager = Arc::new(DatabaseManager::new(&config.database).await?);

    // Initialize health checker
    let health_config = metaphor_health::HealthConfig {
        check_interval: std::time::Duration::from_secs(config.health.check_interval),
        timeout: std::time::Duration::from_secs(30),
        failure_threshold: 3,
        success_threshold: 2,
        include_details: true,
        enable_metrics: true,
        app_version: Some("0.1.0".to_string()),
        app_name: Some("metaphor-app".to_string()),
    };
    let health_checker = HealthChecker::new(health_config);

    // Build the application with Clean Architecture
    let app = build_app(
        config.clone(),
        db_manager.clone(),
        health_checker,
    ).await?;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await?;

    info!("🎯 Metaphor app listening on http://{}", addr);
    info!("📊 Health check: http://{}/health", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 Metaphor application shutdown complete");
    Ok(())
}

/// Initialize tracing and logging
fn init_tracing(config: &AppConfig) -> Result<()> {
    // Simple tracing setup - the complex version can be added later
    tracing_subscriber::fmt()
        .with_max_level(match config.logging.level.as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .init();

    Ok(())
}

/// Create database connection pool
async fn create_database_pool(config: &AppConfig) -> Result<PgPool> {
    info!("🗄️ Connecting to database...");

    let pool = PgPool::connect(&config.database.url)
        .await
        .map_err(|e| {
            error!("❌ Failed to connect to database: {}", e);
            e
        })?;

    // Run health check on database
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            error!("❌ Database health check failed: {}", e);
            e
        })?;

    info!("✅ Database connection established");
    Ok(pool)
}

/// Build the main application router using Clean Architecture
async fn build_app(
    config: AppConfig,
    db_manager: Arc<DatabaseManager>,
    health_checker: HealthChecker,
) -> Result<Router> {
    info!("🏗️ Building application with Clean Architecture");

    // Create shared application state
    let shared_state = shared::AppState::new(
        config.clone(),
        db_manager.pool().clone(),
        health_checker,
    );

    // Build middleware stack
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Build the main router with Clean Architecture layers
    let app = Router::new()
        // Health check endpoints (infrastructure)
        .route("/health", get(health_check))
        .route("/health/detailed", get(detailed_health_check))

        // API routes (presentation layer)
        .route("/api/v1/status", get(api_status))
        .nest("/api/v1", presentation::http::routes::api_routes())

        // Admin routes (presentation layer)
        .nest("/admin", presentation::http::routes::admin_routes())

        // Web routes (presentation layer)
        .nest("/", presentation::http::routes::web_routes())

        // Root endpoint
        .route("/", get(root_endpoint))

        // Add shared state
        .with_state(shared_state)

        // Apply middleware
        .layer(middleware_stack);

    // TODO: Add module routes when modules are created using presentation layer
    // if config.modules.sapiens.enabled {
    //     app = app.nest("/api/v1/sapiens", sapiens_presentation::routes());
    // }

    info!("✅ Application router built successfully");
    Ok(app)
}

/// Health check endpoint
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "metaphor-app",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Detailed health check endpoint
async fn detailed_health_check(
    axum::extract::State(state): axum::extract::State<shared::AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_report = state.health_checker.health_report().await;

    Ok(Json(json!({
        "status": match health_report.status {
            metaphor_health::HealthStatus::Healthy => "healthy",
            metaphor_health::HealthStatus::Degraded => "degraded",
            metaphor_health::HealthStatus::Unhealthy => "unhealthy",
        },
        "service": "metaphor-app",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": health_report.components
    })))
}

/// API status endpoint
async fn api_status() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "operational",
        "api_version": "v1",
        "framework": "Metaphor Framework v2.0",
        "endpoints": {
            "health": "/health",
            "detailed_health": "/health/detailed",
            "modules": {
                "sapiens": "User management (TODO: implement)",
                "postman": "Email notifications (TODO: implement)",
                "bucket": "File storage (TODO: implement)"
            }
        }
    })))
}

/// Root endpoint
async fn root_endpoint() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "service": "Metaphor Framework Application",
        "version": "0.1.0",
        "description": "Laravel-inspired monolith orchestrator for DDD bounded contexts",
        "status": "running",
        "documentation": {
            "health": "/health",
            "api": "/api/v1/status",
            "admin": "/admin"
        },
        "message": "🦀 Welcome to Metaphor Framework - The Modular Monolith Powerhouse!"
    })))
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🛑 Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("🛑 Received terminate signal, shutting down gracefully...");
        },
    }

    // TODO: Add graceful shutdown for modules
    info!("🔄 Shutting down application components...");
}