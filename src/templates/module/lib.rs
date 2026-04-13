//! Metaphor Bounded Context
//!
//! Metaphor Framework bounded context for metaphor domain.
//! Each bounded context owns its complete domain (DDD principle).
//! Uses CORE packages: metaphor-core, metaphor-orm, metaphor-auth
//!
//! ## Architecture
//!
//! This is a **DDD Bounded Context** with:
//! - **Module-Local Schema-First**: All domain starts as YAML schemas in schema/
//! - **Complete Domain Ownership**: No sharing entities across bounded contexts
//! - **Hexagonal Architecture**: Clean Architecture layers within this context
//! - **Event-Driven Integration**: Communication via domain events
//!
//! ## Module Structure
//!
//! ```
//! libs/modules/metaphor/
//! ├── proto/domain/               # 🏛️ DOMAIN LAYER (Proto Definitions - Single Source of Truth)
//! │   ├── entity/                # Domain Entities (Aggregates & Entities)
//! │   ├── value_object/          # Value Objects
//! │   ├── repository/            # Repository Interfaces
//! │   ├── usecase/               # Use Cases (CQRS Commands & Queries)
//! │   ├── service/               # Domain Services
//! │   ├── event/                 # Domain Events
//! │   └── specification/         # Business Rules
//! ├── src/
//! │   ├── domain/                # 🎯 DOMAIN LAYER (Rust Implementation)
//! │   ├── application/           # 📋 APPLICATION LAYER (Use Cases)
//! │   ├── infrastructure/        # 🔧 INFRASTRUCTURE LAYER (Repository Impl)
//! │   └── presentation/          # 🌐 PRESENTATION LAYER (APIs)
//! ├── tests/                     # 🧪 TESTS (Module-Scoped)
//! └── migrations/postgres/       # Database migrations
//! ```
//!
//! ## Quick Start
//!
//! 1. Define entities in `proto/domain/entity/` (Protocol Buffers)
//! 2. Run `metaphor proto:generate` to generate Rust code
//! 3. Implement business logic in domain layer
//! 4. Run `metaphor crud:generate <Entity> metaphor` to generate CRUD endpoints
//! 5. Register this bounded context in `apps/metaphor/metaphor.toml`

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

/// Bounded context version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export generated proto types when available
// Generated from proto/domain/*.proto files
pub use generated::rust::metaphor::*;

// Import CORE Metaphor functionality
use metaphor_core::*;
use metaphor_orm::*;
use metaphor_auth::*;

// Module configuration
#[derive(Debug, Clone)]
pub struct MetaphorModuleConfig {
    pub database_url: String,
    pub service_name: String,
    pub grpc_port: u16,
    pub http_port: u16,
    pub cache_url: Option<String>,
    pub log_level: String,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
}

impl Default for MetaphorModuleConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://root:password@localhost:5432/metaphor_db".to_string()),
            service_name: "metaphor".to_string(),
            grpc_port: std::env::var("GRPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50051),
            http_port: std::env::var("HTTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cache_url: std::env::var("CACHE_URL").ok(),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            enable_metrics: std::env::var("ENABLE_METRICS").unwrap_or_else(|_| "true".to_string()) == "true",
            enable_tracing: std::env::var("ENABLE_TRACING").unwrap_or_else(|_| "true".to_string()) == "true",
        }
    }
}

/// Builder pattern for module configuration
pub struct MetaphorModuleBuilder {
    config: MetaphorModuleConfig,
}

impl MetaphorModuleBuilder {
    pub fn new() -> Self {
        Self {
            config: MetaphorModuleConfig::default(),
        }
    }

    pub fn with_database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    pub fn with_ports(mut self, grpc_port: u16, http_port: u16) -> Self {
        self.config.grpc_port = grpc_port;
        self.config.http_port = http_port;
        self
    }

    pub fn with_cache_url(mut self, cache_url: impl Into<String>) -> Self {
        self.config.cache_url = Some(cache_url.into());
        self
    }

    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.config.log_level = level.into();
        self
    }

    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    pub fn enable_tracing(mut self, enable: bool) -> Self {
        self.config.enable_tracing = enable;
        self
    }

    pub fn build(self) -> Result<MetaphorModule, anyhow::Error> {
        MetaphorModule::new(self.config)
    }
}

/// Main bounded context module
pub struct MetaphorModule {
    pub config: MetaphorModuleConfig,
    database_pool: Option<sqlx::postgres::PgPool>,
    cache_client: Option<redis::Client>,
}

impl MetaphorModule {
    pub fn new(config: MetaphorModuleConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            config,
            database_pool: None,
            cache_client: None,
        })
    }

    /// Initialize the bounded context with all dependencies
    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        // Initialize logging
        self.init_logging()?;

        // Initialize database connection
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(&self.config.database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations/postgres").run(&pool).await?;

        self.database_pool = Some(pool);

        // Initialize cache if URL is provided
        if let Some(cache_url) = &self.config.cache_url {
            let client = redis::Client::open(cache_url)?;
            self.cache_client = Some(client);
        }

        // Initialize metrics if enabled
        if self.config.enable_metrics {
            self.init_metrics();
        }

        tracing::info!("MetaphorModule initialized successfully");
        Ok(())
    }

    /// Configure Actix-web routes
    pub fn configure_routes(&self, cfg: &mut actix_web::web::ServiceConfig) {
        // Import and use presentation layer routes
        cfg.configure(self::presentation::http::configure_routes);
    }

    /// Initialize gRPC services
    pub fn configure_grpc_services(
        &self,
        server: &mut tonic::transport::Server,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use self::presentation::grpc::MetaphorGrpcService;
        use self::domain::repository::metaphor_repository_server::MetaphorRepositoryServer;

        let pool = self.database_pool.as_ref()
            .ok_or("Database not initialized")?;

        let repository = self::infrastructure::persistence::postgres::MetaphorRepository::new(pool);
        let grpc_service = MetaphorGrpcService::new(repository);

        *server = server.add_service(
            MetaphorRepositoryServer::new(grpc_service)
        );

        Ok(())
    }

    /// Get database connection pool
    pub fn database_pool(&self) -> Option<&sqlx::postgres::PgPool> {
        self.database_pool.as_ref()
    }

    /// Get cache client
    pub fn cache_client(&self) -> Option<&redis::Client> {
        self.cache_client.as_ref()
    }

    /// Get module configuration
    pub fn config(&self) -> &MetaphorModuleConfig {
        &self.config
    }

    /// Health check for the module
    pub async fn health_check(&self) -> ModuleHealth {
        let mut health = ModuleHealth {
            service_name: self.config.service_name.clone(),
            status: "healthy".to_string(),
            components: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        // Check database
        if let Some(pool) = &self.database_pool {
            match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => {
                    health.components.insert("database".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    health.components.insert("database".to_string(), format!("unhealthy: {}", e));
                    health.status = "degraded".to_string();
                }
            }
        } else {
            health.components.insert("database".to_string(), "not_initialized".to_string());
            health.status = "unhealthy".to_string();
        }

        // Check cache
        if let Some(client) = &self.cache_client {
            let mut conn = client.get_connection().await.unwrap();
            match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                Ok(_) => {
                    health.components.insert("cache".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    health.components.insert("cache".to_string(), format!("unhealthy: {}", e));
                    health.status = "degraded".to_string();
                }
            }
        }

        health
    }

    fn init_logging(&self) -> Result<(), anyhow::Error> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&self.config.log_level));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();

        Ok(())
    }

    fn init_metrics(&self) {
        metrics::describe_histogram!(
            "metaphor_request_duration_seconds",
            "Request duration in seconds",
            "endpoint",
            "method",
            "status"
        );

        metrics::describe_counter!(
            "metaphor_requests_total",
            "Total number of requests",
            "endpoint",
            "method",
            "status"
        );
    }
}

/// Convenience function for creating module
pub fn builder() -> MetaphorModuleBuilder {
    MetaphorModuleBuilder::new()
}

/// Health status structure
#[derive(Debug, serde::Serialize)]
pub struct ModuleHealth {
    pub service_name: String,
    pub status: String,
    pub components: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Drop for MetaphorModule {
    fn drop(&mut self) {
        tracing::info!("MetaphorModule shutting down");
    }
}