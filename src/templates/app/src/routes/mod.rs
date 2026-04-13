//! Application routes module

use axum::Router;
use crate::shared::AppState;

/// API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/status", status_routes())
        // TODO: Add module-specific routes when modules are created
        // .nest("/sapiens", sapiens::routes())
        // .nest("/postman", postman::routes())
        // .nest("/bucket", bucket::routes())
}

/// Status routes
pub fn status_routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(api_status))
        .route("/health", axum::routing::get(health_status))
        .route("/modules", axum::routing::get(modules_status))
}

/// Admin routes
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", axum::routing::get(admin_dashboard))
        .route("/modules", axum::routing::get(admin_modules))
        .route("/config", axum::routing::get(admin_config))
        .route("/health/detailed", axum::routing::get(admin_health))
}

// Route handlers
async fn api_status() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "status": "operational",
        "api_version": "v1",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn health_status() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn modules_status() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "modules": {
            "sapiens": {
                "name": "User Management",
                "status": "configured",
                "enabled": true,
                "description": "User authentication, authorization, and management"
            },
            "postman": {
                "name": "Email Notifications",
                "status": "configured",
                "enabled": true,
                "description": "Email sending and notification management"
            },
            "bucket": {
                "name": "File Storage",
                "status": "configured",
                "enabled": true,
                "description": "File storage and document management"
            }
        }
    }))
}

async fn admin_dashboard() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "admin_panel": "Metaphor Framework Administration",
        "version": "0.1.0",
        "available_sections": [
            "modules",
            "config",
            "health",
            "logs",
            "metrics"
        ]
    }))
}

async fn admin_modules() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "modules": [
            {
                "name": "sapiens",
                "display_name": "User Management",
                "description": "Handle user authentication, profiles, and permissions",
                "status": "configured",
                "enabled": true,
                "endpoints": [
                    "/api/v1/sapiens/users",
                    "/api/v1/sapiens/auth",
                    "/api/v1/sapiens/profiles"
                ]
            },
            {
                "name": "postman",
                "display_name": "Email Notifications",
                "description": "Send emails and manage notifications",
                "status": "configured",
                "enabled": true,
                "endpoints": [
                    "/api/v1/postman/send",
                    "/api/v1/postman/templates",
                    "/api/v1/postman/queue"
                ]
            },
            {
                "name": "bucket",
                "display_name": "File Storage",
                "description": "Store and manage files and documents",
                "status": "configured",
                "enabled": true,
                "endpoints": [
                    "/api/v1/bucket/files",
                    "/api/v1/bucket/upload",
                    "/api/v1/bucket/download"
                ]
            }
        ]
    }))
}

async fn admin_config() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "configuration": {
            "server": {
                "host": "loaded_from_config",
                "port": "loaded_from_config",
                "workers": "loaded_from_config"
            },
            "database": {
                "status": "connected",
                "pool_size": "configured"
            },
            "modules": {
                "sapiens": "enabled",
                "postman": "enabled",
                "bucket": "enabled"
            },
            "security": {
                "jwt": "configured",
                "cors": "enabled",
                "rate_limiting": "enabled"
            }
        }
    }))
}

async fn admin_health(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Json<serde_json::Value> {
    let health_report = state.health_checker.health_report().await;

    axum::response::Json(serde_json::json!({
        "health": {
            "overall": match health_report.status {
            metaphor_health::HealthStatus::Healthy => "healthy",
            metaphor_health::HealthStatus::Degraded => "degraded",
            metaphor_health::HealthStatus::Unhealthy => "unhealthy",
        },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "components": health_report.components,
            "checks_performed": [
                "database_connection",
                "database_queries",
                "memory_usage",
                "disk_space"
            ]
        }
    }))
}