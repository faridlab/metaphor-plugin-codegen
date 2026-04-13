//! HTTP Routes
//!
//! Route definitions for HTTP endpoints in the presentation layer

use axum::{routing::get, Router};
use crate::shared::AppState;

/// API v1 routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(api_status))
        .nest("/status", status_routes())

        // TODO: Add module-specific routes when modules are created
        // .nest("/sapiens", sapiens::presentation::http::routes())
        // .nest("/postman", postman::presentation::http::routes())
        // .nest("/bucket", bucket::presentation::http::routes())
}

/// Status routes
pub fn status_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(api_status))
        .route("/health", get(health_status))
        .route("/modules", get(modules_status))
}

/// Admin routes
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_dashboard))
        .route("/modules", get(admin_modules))
        .route("/config", get(admin_config))
        .route("/health/detailed", get(admin_health))
}

/// Web routes (HTML interface)
pub fn web_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(web_home))
        .route("/docs", get(web_docs))
        .route("/docs/api", get(api_docs))
}

// Route handlers
async fn api_status() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "status": "operational",
        "api_version": "v1",
        "architecture": "Clean Architecture",
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
            "domain": {
                "name": "Cross-Cutting Domain",
                "status": "configured",
                "enabled": true,
                "description": "Application-wide business rules and entities"
            },
            "application": {
                "name": "Application Use Cases",
                "status": "configured",
                "enabled": true,
                "description": "Orchestration and multi-module workflows"
            },
            "infrastructure": {
                "name": "Technical Infrastructure",
                "status": "configured",
                "enabled": true,
                "description": "Database, external services, and technical concerns"
            },
            "presentation": {
                "name": "HTTP/gRPC/CLI Interfaces",
                "status": "configured",
                "enabled": true,
                "description": "API endpoints and user interfaces"
            }
        }
    }))
}

async fn admin_dashboard() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "admin_panel": "Metaphor Framework Administration",
        "version": "0.1.0",
        "architecture": "Clean Architecture",
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
        "architecture_layers": [
            {
                "name": "domain",
                "description": "Business rules and cross-cutting entities",
                "path": "src/domain/",
                "components": [
                    "entities",
                    "value_objects",
                    "services",
                    "repositories",
                    "events"
                ]
            },
            {
                "name": "application",
                "description": "Use cases and orchestration",
                "path": "src/application/",
                "components": [
                    "commands",
                    "queries",
                    "services",
                    "use_cases"
                ]
            },
            {
                "name": "infrastructure",
                "description": "Database and technical implementations",
                "path": "src/infrastructure/",
                "components": [
                    "database",
                    "external",
                    "messaging",
                    "health"
                ]
            },
            {
                "name": "presentation",
                "description": "HTTP, gRPC, and CLI interfaces",
                "path": "src/presentation/",
                "components": [
                    "http",
                    "grpc",
                    "cli"
                ]
            }
        ]
    }))
}

async fn admin_config() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "architecture": "Clean Architecture",
        "description": "The application follows Clean Architecture principles",
        "benefits": [
            "Independent of frameworks",
            "Testable",
            "Independent of UI",
            "Independent of database",
            "Independent of external services",
            "Clear separation of concerns"
        ],
        "dependency_direction": {
            "presentation": "-> application -> domain",
            "infrastructure": "-> domain",
            "application": "-> domain",
            "domain": "no dependencies (core)"
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
            "architecture": "Clean Architecture"
        }
    }))
}

// Web interface handlers
async fn web_home() -> axum::response::Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Metaphor Framework - Clean Architecture</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #2c3e50 0%, #3498db 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            max-width: 800px;
            padding: 2rem;
            background: white;
            border-radius: 10px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }
        .header {
            text-align: center;
            margin-bottom: 2rem;
        }
        .logo {
            font-size: 3rem;
            margin-bottom: 1rem;
        }
        .title {
            font-size: 2.5rem;
            color: #2c3e50;
            margin-bottom: 0.5rem;
        }
        .subtitle {
            color: #7f8c8d;
            font-size: 1.2rem;
        }
        .architecture {
            background: #f8f9fa;
            padding: 2rem;
            border-radius: 8px;
            margin: 2rem 0;
        }
        .layers {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-top: 1rem;
        }
        .layer {
            padding: 1rem;
            border-left: 4px solid #3498db;
            background: white;
            border-radius: 0 4px 4px 0;
        }
        .layer h4 {
            margin: 0 0 0.5rem 0;
            color: #2c3e50;
        }
        .layer p {
            margin: 0;
            color: #7f8c8d;
            font-size: 0.9rem;
        }
        .links {
            text-align: center;
            margin-top: 2rem;
        }
        .links a {
            display: inline-block;
            margin: 0.5rem;
            padding: 0.75rem 1.5rem;
            background: #3498db;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            transition: background-color 0.2s;
        }
        .links a:hover {
            background: #2980b9;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🏗️</div>
            <h1 class="title">Metaphor Framework</h1>
            <p class="subtitle">Clean Architecture Modular Monolith</p>
        </div>

        <div class="architecture">
            <h3>🏛️ Clean Architecture Layers</h3>
            <div class="layers">
                <div class="layer">
                    <h4>🎯 Domain</h4>
                    <p>Business rules and cross-cutting entities</p>
                </div>
                <div class="layer">
                    <h4>📋 Application</h4>
                    <p>Use cases and multi-module orchestration</p>
                </div>
                <div class="layer">
                    <h4>🔧 Infrastructure</h4>
                    <p>Database and technical implementations</p>
                </div>
                <div class="layer">
                    <h4>🌐 Presentation</h4>
                    <p>HTTP, gRPC, and CLI interfaces</p>
                </div>
            </div>
        </div>

        <div class="links">
            <a href="/api/v1/status">📚 API Documentation</a>
            <a href="/admin">⚙️ Admin Panel</a>
            <a href="/health">❤️ Health Check</a>
        </div>
    </div>
</body>
</html>
    "#;
    axum::response::Html(html.to_string())
}

async fn web_docs() -> axum::response::Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Clean Architecture Documentation - Metaphor Framework</title>
</head>
<body style="font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem;">
    <h1>🏗️ Clean Architecture Documentation</h1>

    <h2>Architecture Overview</h2>
    <p>The Metaphor Framework follows Clean Architecture principles with these layers:</p>

    <ul>
        <li><strong>Domain Layer</strong> - Core business rules and entities</li>
        <li><strong>Application Layer</strong> - Use cases and orchestration</li>
        <li><strong>Infrastructure Layer</strong> - Database and external services</li>
        <li><strong>Presentation Layer</strong> - HTTP, gRPC, and CLI interfaces</li>
    </ul>

    <h3>Benefits of Clean Architecture</h3>
    <ul>
        <li>✅ Independent of frameworks</li>
        <li>✅ Testable</li>
        <li>✅ Independent of UI</li>
        <li>✅ Independent of database</li>
        <li>✅ Clear separation of concerns</li>
    </ul>

    <p><a href="/">← Back to Home</a></p>
</body>
</html>
    "#;
    axum::response::Html(html.to_string())
}

async fn api_docs() -> axum::response::Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>API Documentation - Metaphor Framework</title>
</head>
<body style="font-family: Arial, sans-serif; max-width: 1000px; margin: 0 auto; padding: 2rem;">
    <h1>🔌 API Documentation</h1>

    <h2>Clean Architecture API Design</h2>
    <p>API endpoints are organized by Clean Architecture layers:</p>

    <h3>Endpoints</h3>
    <ul>
        <li><a href="/api/v1/status">GET /api/v1/status</a> - API status</li>
        <li><a href="/api/v1/status/health">GET /api/v1/status/health</a> - Health check</li>
        <li><a href="/api/v1/status/modules">GET /api/v1/status/modules</a> - Module status</li>
    </ul>

    <p><a href="/">← Back to Home</a></p>
</body>
</html>
    "#;
    axum::response::Html(html.to_string())
}