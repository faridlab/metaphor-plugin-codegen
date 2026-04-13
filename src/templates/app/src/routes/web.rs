//! Web routes (HTML responses, web interface)

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Response},
    routing::get,
    Router,
};
use crate::shared::AppState;

/// Create web routes
pub fn web_routes() -> Router<AppState> {
    Router::new()
        // Root web page
        .route("/", get(web_home))

        // Documentation routes
        .route("/docs", get(web_docs))
        .route("/docs/api", get(api_docs))
        .route("/docs/api/v1", get(api_v1_docs))
        .route("/docs/guide", get(web_guide))

        // Admin web interface
        .nest("/admin", admin_web_routes())

        // Static assets (if needed)
        .route("/static/*path", get(static_assets))

        // Error pages
        .route("/404", get(not_found_page))
        .route("/500", get(internal_error_page))
}

/// Admin web interface routes
fn admin_web_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_dashboard_page))
        .route("/dashboard", get(admin_dashboard_page))
        .route("/users", get(admin_users_page))
        .route("/users/:id", get(admin_user_details_page))
        .route("/system", get(admin_system_page))
        .route("/logs", get(admin_logs_page))
        .route("/configuration", get(admin_configuration_page))
        .route("/health", get(admin_health_page))
}

// Web Page Handlers

/// Home page handler
async fn web_home() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Metaphor Framework</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
            color: #333;
            margin-bottom: 0.5rem;
        }
        .subtitle {
            color: #666;
            font-size: 1.2rem;
        }
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            margin: 2rem 0;
        }
        .feature {
            padding: 1.5rem;
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            text-align: center;
        }
        .feature h3 {
            color: #667eea;
            margin-bottom: 0.5rem;
        }
        .links {
            text-align: center;
            margin-top: 2rem;
        }
        .links a {
            display: inline-block;
            margin: 0.5rem;
            padding: 0.75rem 1.5rem;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            transition: background-color 0.2s;
        }
        .links a:hover {
            background: #5a6fd8;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🦀</div>
            <h1 class="title">Metaphor Framework</h1>
            <p class="subtitle">Laravel-inspired Modular Monolith for Rust</p>
        </div>

        <div class="features">
            <div class="feature">
                <h3>🏛️ DDD Architecture</h3>
                <p>Domain-Driven Design with bounded contexts</p>
            </div>
            <div class="feature">
                <h3>🔧 Schema-First</h3>
                <p>YAML schemas for type-safe development</p>
            </div>
            <div class="feature">
                <h3>📦 Modular Design</h3>
                <p>Independent, composable business modules</p>
            </div>
            <div class="feature">
                <h3>⚡ High Performance</h3>
                <p>Built on Rust with Axum web framework</p>
            </div>
        </div>

        <div class="links">
            <a href="/docs/api">📚 API Documentation</a>
            <a href="/admin">⚙️ Admin Panel</a>
            <a href="/health">❤️ Health Check</a>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// Documentation home page
async fn web_docs() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Documentation - Metaphor Framework</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 2rem; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { text-align: center; margin-bottom: 2rem; }
        .nav { background: #f5f5f5; padding: 1rem; border-radius: 5px; margin-bottom: 2rem; }
        .nav a { margin: 0 1rem; text-decoration: none; color: #667eea; }
        .content { background: white; padding: 2rem; border-radius: 5px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .section { margin-bottom: 2rem; }
        .section h2 { color: #333; border-bottom: 2px solid #667eea; padding-bottom: 0.5rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📚 Metaphor Framework Documentation</h1>
        </div>

        <div class="nav">
            <a href="/">Home</a>
            <a href="/docs/api">API Documentation</a>
            <a href="/admin">Admin Panel</a>
            <a href="/health">Health Check</a>
        </div>

        <div class="content">
            <div class="section">
                <h2>🚀 Getting Started</h2>
                <p>Welcome to Metaphor Framework! This documentation will help you get started with building modular monolith applications using Rust.</p>

                <h3>Key Features:</h3>
                <ul>
                    <li><strong>Domain-Driven Design (DDD)</strong> - Organize your code with bounded contexts</li>
                    <li><strong>Schema-First Development</strong> - Define your domain in YAML schemas</li>
                    <li><strong>Modular Architecture</strong> - Compose applications from independent modules</li>
                    <li><strong>Laravel-Inspired</strong> - Familiar patterns and conventions</li>
                    <li><strong>High Performance</strong> - Built on Rust with Axum</li>
                </ul>
            </div>

            <div class="section">
                <h2>📖 Documentation Sections</h2>
                <ul>
                    <li><a href="/docs/api">API Documentation</a> - REST API reference and examples</li>
                    <li><a href="/docs/guide">Developer Guide</a> - Development patterns and best practices</li>
                    <li><a href="/admin">Admin Panel</a> - Administrative interface</li>
                </ul>
            </div>

            <div class="section">
                <h2>🔗 Quick Links</h2>
                <ul>
                    <li><a href="/api/v1/status">API Status</a></li>
                    <li><a href="/health">Health Check</a></li>
                    <li><a href="/admin">Admin Dashboard</a></li>
                </ul>
            </div>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// API documentation page
async fn api_docs() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation - Metaphor Framework</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 2rem; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { text-align: center; margin-bottom: 2rem; }
        .content { background: white; padding: 2rem; border-radius: 5px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .endpoint { border: 1px solid #ddd; margin: 1rem 0; padding: 1rem; border-radius: 5px; }
        .method { display: inline-block; padding: 0.25rem 0.5rem; border-radius: 3px; color: white; font-weight: bold; }
        .get { background: #61affe; }
        .post { background: #49cc90; }
        .put { background: #fca130; }
        .delete { background: #f93e3e; }
        .patch { background: #50e3c2; }
        .path { font-family: monospace; background: #f5f5f5; padding: 0.25rem 0.5rem; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔌 Metaphor Framework API Documentation</h1>
            <p>Version 1.0.0</p>
        </div>

        <div class="content">
            <h2>📡 API Endpoints</h2>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/status</span>
                <p>Get API status and information</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/health</span>
                <p>Basic health check</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/modules</span>
                <p>List available modules and their status</p>
            </div>

            <h3>👤 User Management (Sapiens Module)</h3>
            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/users</span>
                <p>List users (with pagination, filtering, sorting)</p>
            </div>

            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/users</span>
                <p>Create new user</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/users/:id</span>
                <p>Get user by ID</p>
            </div>

            <div class="endpoint">
                <span class="method put">PUT</span>
                <span class="path">/api/v1/users/:id</span>
                <p>Update user (full update)</p>
            </div>

            <div class="endpoint">
                <span class="method patch">PATCH</span>
                <span class="path">/api/v1/users/:id</span>
                <p>Update user (partial update)</p>
            </div>

            <div class="endpoint">
                <span class="method delete">DELETE</span>
                <span class="path">/api/v1/users/:id</span>
                <p>Soft delete user</p>
            </div>

            <h3>🔐 Authentication</h3>
            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/auth/login</span>
                <p>User login</p>
            </div>

            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/auth/logout</span>
                <p>User logout</p>
            </div>

            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/auth/register</span>
                <p>User registration</p>
            </div>

            <h3>📧 Email Notifications (Postman Module)</h3>
            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/email/send</span>
                <p>Send email</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/email/templates</span>
                <p>List email templates</p>
            </div>

            <h3>📁 File Storage (Bucket Module)</h3>
            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/files</span>
                <p>List files</p>
            </div>

            <div class="endpoint">
                <span class="method post">POST</span>
                <span class="path">/api/v1/files</span>
                <p>Upload file</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/files/:id/download</span>
                <p>Download file</p>
            </div>

            <h3>⚙️ Admin Endpoints</h3>
            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/admin</span>
                <p>Admin dashboard information</p>
            </div>

            <div class="endpoint">
                <span class="method get">GET</span>
                <span class="path">/api/v1/admin/system/info</span>
                <p>System information</p>
            </div>

            <p><strong>Note:</strong> Many endpoints are currently placeholder implementations and will be fully functional when the respective modules are implemented.</p>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// API v1 specific documentation
async fn api_v1_docs() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API v1 Documentation - Metaphor Framework</title>
</head>
<body>
    <div style="font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem;">
        <h1>🔌 Metaphor Framework API v1</h1>
        <p>Detailed API v1 documentation with examples and response formats.</p>

        <h2>🔗 Related Links</h2>
        <ul>
            <li><a href="/docs/api">API Overview</a></li>
            <li><a href="/docs">Documentation Home</a></li>
            <li><a href="/">Home</a></li>
        </ul>

        <div style="margin-top: 2rem; padding: 1rem; background: #f5f5f5; border-radius: 5px;">
            <h3>🚧 Under Development</h3>
            <p>This section is currently under development. Please check the <a href="/docs/api">API Overview</a> for available endpoints.</p>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// Developer guide page
async fn web_guide() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Developer Guide - Metaphor Framework</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 2rem; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; }
        .content { background: white; padding: 2rem; border-radius: 5px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .section { margin-bottom: 2rem; }
        .code { background: #f5f5f5; padding: 1rem; border-radius: 3px; font-family: monospace; }
    </style>
</head>
<body>
    <div class="container">
        <div class="content">
            <h1>📖 Metaphor Framework Developer Guide</h1>

            <div class="section">
                <h2>🏛️ Architecture Overview</h2>
                <p>Metaphor Framework follows Domain-Driven Design (DDD) principles with a modular monolith architecture.</p>

                <h3>Key Concepts:</h3>
                <ul>
                    <li><strong>Bounded Contexts</strong> - Independent business domains</li>
                    <li><strong>Schema-First</strong> - Define contracts before implementation</li>
                    <li><strong>Clean Architecture</strong> - Separation of concerns</li>
                    <li><strong>Modular Design</strong> - Composable business modules</li>
                </ul>
            </div>

            <div class="section">
                <h2>🔧 Development Setup</h2>
                <div class="code">
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone <repository-url>
cd monorepo-metaphor

# Build the application
cargo build --release

# Run development server
cargo run --bin metaphor
                </div>
            </div>

            <div class="section">
                <h2>📦 Module Development</h2>
                <p>Modules are created using the Metaphor CLI:</p>
                <div class="code">
# Create a new module
metaphor module create mymodule --description "My business module"

# Generate an entity
metaphor entity:create User mymodule

# Generate CRUD endpoints
metaphor crud:generate User mymodule
                </div>
            </div>

            <div class="section">
                <h2>🚀 Next Steps</h2>
                <p>This guide is currently being expanded. Check back for more detailed development patterns and examples.</p>
            </div>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

// Admin Web Interface Handlers

/// Admin dashboard page
async fn admin_dashboard_page() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin Dashboard - Metaphor Framework</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 1rem; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { background: white; padding: 1rem; border-radius: 5px; margin-bottom: 1rem; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
        .dashboard { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; }
        .card { background: white; padding: 1.5rem; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
        .nav { background: white; padding: 1rem; border-radius: 5px; margin-bottom: 1rem; }
        .nav a { margin-right: 1rem; text-decoration: none; color: #667eea; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⚙️ Admin Dashboard</h1>
            <p>Metaphor Framework Administration</p>
        </div>

        <div class="nav">
            <a href="/admin">Dashboard</a>
            <a href="/admin/users">Users</a>
            <a href="/admin/system">System</a>
            <a href="/admin/logs">Logs</a>
            <a href="/admin/configuration">Configuration</a>
            <a href="/admin/health">Health</a>
            <a href="/">Back to Home</a>
        </div>

        <div class="dashboard">
            <div class="card">
                <h3>📊 System Status</h3>
                <p>All systems operational</p>
                <p><strong>Uptime:</strong> 2 days, 14 hours</p>
                <p><strong>Version:</strong> 1.0.0</p>
            </div>

            <div class="card">
                <h3>👥 Users</h3>
                <p><strong>Total:</strong> 1,234</p>
                <p><strong>Active:</strong> 987</p>
                <p><strong>New today:</strong> 12</p>
            </div>

            <div class="card">
                <h3>📧 Email Statistics</h3>
                <p><strong>Sent today:</strong> 456</p>
                <p><strong>Failed:</strong> 2</p>
                <p><strong>Queue size:</strong> 23</p>
            </div>

            <div class="card">
                <h3>📁 File Storage</h3>
                <p><strong>Total files:</strong> 5,678</p>
                <p><strong>Storage used:</strong> 12.3 GB</p>
                <p><strong>Uploads today:</strong> 34</p>
            </div>
        </div>
    </div>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// Placeholder admin pages
async fn admin_users_page() -> Html<String> { Html("<h1>👥 User Management</h1><p>Coming soon...</p>".to_string()) }
async fn admin_user_details_page() -> Html<String> { Html("<h1>👤 User Details</h1><p>Coming soon...</p>".to_string()) }
async fn admin_system_page() -> Html<String> { Html("<h1>🖥️ System Information</h1><p>Coming soon...</p>".to_string()) }
async fn admin_logs_page() -> Html<String> { Html("<h1>📄 System Logs</h1><p>Coming soon...</p>".to_string()) }
async fn admin_configuration_page() -> Html<String> { Html("<h1>⚙️ Configuration</h1><p>Coming soon...</p>".to_string()) }
async fn admin_health_page() -> Html<String> { Html("<h1>❤️ Health Status</h1><p>Coming soon...</p>".to_string()) }

// Utility Handlers

/// Static assets handler (placeholder)
async fn static_assets() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Static assets not configured".into())
        .unwrap()
}

/// 404 Not Found page
async fn not_found_page() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Not Found - Metaphor Framework</title>
</head>
<body style="font-family: Arial, sans-serif; text-align: center; padding: 2rem;">
    <h1>🔍 404 - Page Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <a href="/" style="color: #667eea; text-decoration: none;">Go Home</a>
</body>
</html>
    "#;
    Html(html.to_string())
}

/// 500 Internal Error page
async fn internal_error_page() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Internal Error - Metaphor Framework</title>
</head>
<body style="font-family: Arial, sans-serif; text-align: center; padding: 2rem;">
    <h1>⚠️ 500 - Internal Server Error</h1>
    <p>Something went wrong. Please try again later.</p>
    <a href="/" style="color: #667eea; text-decoration: none;">Go Home</a>
</body>
</html>
    "#;
    Html(html.to_string())
}