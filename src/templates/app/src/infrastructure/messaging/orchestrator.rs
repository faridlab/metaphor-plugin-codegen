//! Event Orchestrator - Central coordination for cross-module integration events
//!
//! The Event Orchestrator provides a centralized hub for managing integration events
//! across all bounded contexts (modules) in the application.
//!
//! # Architecture
//!
//! ```text
//!                    ┌────────────────────────────┐
//!                    │    Event Orchestrator      │
//!                    │  (Application Bootstrap)   │
//!                    └────────────────────────────┘
//!                              │
//!              ┌───────────────┼───────────────┐
//!              │               │               │
//!              ▼               ▼               ▼
//!     ┌────────────┐  ┌────────────┐  ┌────────────┐
//!     │  Sapiens   │  │  Postman   │  │  Bucket  │
//!     │  Module    │  │  Module    │  │  Module    │
//!     └────────────┘  └────────────┘  └────────────┘
//!              │               │               │
//!              └───────────────┼───────────────┘
//!                              │
//!                    ┌────────────────────┐
//!                    │ IntegrationEventBus │
//!                    │   (Shared Bus)      │
//!                    └────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use metaphor::infrastructure::messaging::orchestrator::EventOrchestrator;
//!
//! // In main.rs during application bootstrap
//! let orchestrator = EventOrchestrator::new();
//!
//! // Get the shared integration bus to pass to modules
//! let integration_bus = orchestrator.integration_bus();
//!
//! // Build modules with the shared bus
//! let sapiens = SapiensModule::builder()
//!     .with_database(pool)
//!     .with_integration_bus(integration_bus.clone())
//!     .build()
//!     .await?;
//!
//! // Register cross-module handlers
//! orchestrator.register_handlers().await;
//! ```

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug};

use metaphor_messaging::{
    IntegrationEventBus,
    IntegrationBusConfig,
    IntegrationEventHandler,
    IntegrationEventEnvelope,
    EventError,
};

/// Central event orchestrator for the application
///
/// Manages the lifecycle of the shared integration event bus and
/// coordinates cross-module event handling.
pub struct EventOrchestrator {
    /// The shared integration event bus
    integration_bus: Arc<IntegrationEventBus>,
    /// Configuration for the orchestrator
    config: OrchestratorConfig,
}

/// Configuration for the event orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum events to keep in history
    pub max_history_size: usize,
    /// Whether to enable the dead letter queue
    pub enable_dead_letter_queue: bool,
    /// Whether to log all events
    pub log_all_events: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100_000,
            enable_dead_letter_queue: true,
            log_all_events: true,
        }
    }
}

impl EventOrchestrator {
    /// Create a new event orchestrator with default configuration
    pub fn new() -> Self {
        Self::with_config(OrchestratorConfig::default())
    }

    /// Create a new event orchestrator with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        let bus_config = IntegrationBusConfig {
            buffer_size: 10_000,
            persist_events: true,
            max_history_size: config.max_history_size,
            enable_dead_letter_queue: config.enable_dead_letter_queue,
        };

        let bus = IntegrationEventBus::with_config(bus_config);

        Self {
            integration_bus: Arc::new(bus),
            config,
        }
    }

    /// Get the shared integration event bus
    ///
    /// Pass this to modules during initialization so they can
    /// publish and subscribe to integration events.
    pub fn integration_bus(&self) -> Arc<IntegrationEventBus> {
        Arc::clone(&self.integration_bus)
    }

    /// Register all cross-module event handlers
    ///
    /// This should be called after all modules are initialized.
    /// It sets up handlers that need to react to events from other modules.
    pub async fn register_handlers(&self) {
        info!("📡 Registering integration event handlers...");

        // Register the global logging handler if enabled
        if self.config.log_all_events {
            let logging_handler = IntegrationLoggingHandler::new();
            self.integration_bus
                .register_handler(Arc::new(logging_handler))
                .await;
            info!("  ✓ Registered global event logging handler");
        }

        // Note: Additional handlers can be registered here as modules are added

        info!("📡 Integration event handlers registered");
    }

    /// Get the current health status of the event system
    pub async fn health_status(&self) -> EventSystemHealth {
        let history = self.integration_bus.history().await;
        let dead_letters = self.integration_bus.dead_letters().await;

        EventSystemHealth {
            status: if dead_letters.is_empty() { "healthy" } else { "degraded" }.to_string(),
            total_events_processed: history.len(),
            dead_letter_count: dead_letters.len(),
            last_event_at: history.last().map(|e| e.published_at),
        }
    }

    /// Get statistics about the integration event system
    pub async fn statistics(&self) -> EventStatistics {
        let history = self.integration_bus.history().await;
        let dead_letters = self.integration_bus.dead_letters().await;

        let mut events_by_context: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for event in &history {
            *events_by_context.entry(event.source_context.clone()).or_default() += 1;
        }

        let mut events_by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for event in &history {
            *events_by_type.entry(event.event_type.clone()).or_default() += 1;
        }

        EventStatistics {
            total_events: history.len(),
            dead_letter_count: dead_letters.len(),
            events_by_context,
            events_by_type,
        }
    }
}

impl Default for EventOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Health Status Types
// ============================================================

/// Health status of the event system
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventSystemHealth {
    pub status: String,
    pub total_events_processed: usize,
    pub dead_letter_count: usize,
    pub last_event_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Statistics about the event system
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventStatistics {
    pub total_events: usize,
    pub dead_letter_count: usize,
    pub events_by_context: std::collections::HashMap<String, usize>,
    pub events_by_type: std::collections::HashMap<String, usize>,
}

// ============================================================
// Built-in Handlers
// ============================================================

/// Logging handler for all integration events
///
/// Logs every integration event for debugging and audit purposes.
pub struct IntegrationLoggingHandler;

impl IntegrationLoggingHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IntegrationLoggingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationEventHandler for IntegrationLoggingHandler {
    async fn handle(&self, envelope: IntegrationEventEnvelope) -> Result<(), EventError> {
        info!(
            event_type = %envelope.event_type,
            source_context = %envelope.source_context,
            aggregate_id = %envelope.aggregate_id,
            event_id = %envelope.id,
            correlation_id = ?envelope.correlation_id,
            "📨 Integration event received"
        );
        Ok(())
    }

    fn event_patterns(&self) -> Vec<&'static str> {
        vec!["*"] // Subscribe to all events
    }

    fn name(&self) -> &'static str {
        "IntegrationLoggingHandler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = EventOrchestrator::new();
        let bus = orchestrator.integration_bus();

        assert!(Arc::strong_count(&bus) >= 1);
    }

    #[tokio::test]
    async fn test_orchestrator_health() {
        let orchestrator = EventOrchestrator::new();
        let health = orchestrator.health_status().await;

        assert_eq!(health.status, "healthy");
        assert_eq!(health.total_events_processed, 0);
        assert_eq!(health.dead_letter_count, 0);
    }

    #[tokio::test]
    async fn test_orchestrator_statistics() {
        let orchestrator = EventOrchestrator::new();
        let stats = orchestrator.statistics().await;

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.dead_letter_count, 0);
        assert!(stats.events_by_context.is_empty());
    }

    #[tokio::test]
    async fn test_register_handlers() {
        let orchestrator = EventOrchestrator::new();
        orchestrator.register_handlers().await;
    }

    #[tokio::test]
    async fn test_logging_handler() {
        let handler = IntegrationLoggingHandler::new();

        assert_eq!(handler.name(), "IntegrationLoggingHandler");
        assert_eq!(handler.event_patterns(), vec!["*"]);
    }
}
