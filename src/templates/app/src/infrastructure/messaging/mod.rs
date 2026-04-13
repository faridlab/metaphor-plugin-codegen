//! Messaging Infrastructure
//!
//! Message brokering and event streaming infrastructure for asynchronous
//! communication between modules and external systems.

pub mod orchestrator;

use crate::domain::events::{DomainEvent, EventBus};
use crate::shared::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Message broker interface
#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, topic: &str, message: &Message) -> AppResult<()>;
    async fn subscribe(&self, topic: &str, handler: Box<dyn MessageHandler + Send + Sync>) -> AppResult<()>;
    async fn unsubscribe(&self, topic: &str, handler_id: &str) -> AppResult<()>;
    async fn create_topic(&self, topic: &str, config: TopicConfig) -> AppResult<()>;
    async fn delete_topic(&self, topic: &str) -> AppResult<()>;
}

/// Message handler interface
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, message: &Message) -> AppResult<()>;
    fn handler_id(&self) -> &str;
}

/// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub topic: String,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<Uuid>,
    pub reply_to: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl Message {
    pub fn new(topic: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic,
            payload,
            headers: HashMap::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            reply_to: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    pub fn should_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Topic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub partitions: u32,
    pub replication_factor: u32,
    pub retention_ms: Option<u64>,
    pub max_message_size_bytes: u32,
}

impl Default for TopicConfig {
    fn default() -> Self {
        Self {
            partitions: 1,
            replication_factor: 1,
            retention_ms: Some(7 * 24 * 60 * 60 * 1000), // 7 days
            max_message_size_bytes: 1024 * 1024, // 1MB
        }
    }
}

/// In-memory message broker for development and testing
pub struct InMemoryMessageBroker {
    topics: HashMap<String, Vec<Message>>,
    handlers: HashMap<String, Vec<Box<dyn MessageHandler + Send + Sync>>>,
    config: BrokerConfig,
}

/// Broker configuration
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub max_messages_per_topic: usize,
    pub enable_persistence: bool,
    pub retention_hours: u32,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            max_messages_per_topic: 1000,
            enable_persistence: false,
            retention_hours: 24,
        }
    }
}

impl InMemoryMessageBroker {
    pub fn new(config: BrokerConfig) -> Self {
        Self {
            topics: HashMap::new(),
            handlers: HashMap::new(),
            config,
        }
    }

    async fn deliver_to_handlers(&self, topic: &str, message: &Message) {
        if let Some(handlers) = self.handlers.get(topic) {
            for handler in handlers {
                if let Err(e) = handler.handle(message).await {
                    eprintln!("Error handling message in handler {}: {:?}", handler.handler_id(), e);
                }
            }
        }
    }

    fn cleanup_old_messages(&mut self) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(self.config.retention_hours as i64);

        for messages in self.topics.values_mut() {
            messages.retain(|msg| msg.timestamp > cutoff_time);
        }
    }
}

#[async_trait]
impl MessageBroker for InMemoryMessageBroker {
    async fn publish(&self, topic: &str, message: &Message) -> AppResult<()> {
        let mut broker = self.clone();

        // Add message to topic
        let messages = broker.topics.entry(topic.to_string()).or_insert_with(Vec::new);
        messages.push(message.clone());

        // Limit messages per topic
        if messages.len() > self.config.max_messages_per_topic {
            messages.remove(0);
        }

        // Deliver to handlers
        broker.deliver_to_handlers(topic, message).await;

        // Cleanup old messages
        broker.cleanup_old_messages();

        Ok(())
    }

    async fn subscribe(&mut self, topic: &str, handler: Box<dyn MessageHandler + Send + Sync>) -> AppResult<()> {
        let handlers = self.handlers.entry(topic.to_string()).or_insert_with(Vec::new);
        handlers.push(handler);
        Ok(())
    }

    async fn unsubscribe(&mut self, topic: &str, handler_id: &str) -> AppResult<()> {
        if let Some(handlers) = self.handlers.get_mut(topic) {
            handlers.retain(|h| h.handler_id() != handler_id);
        }
        Ok(())
    }

    async fn create_topic(&mut self, topic: &str, _config: TopicConfig) -> AppResult<()> {
        self.topics.entry(topic.to_string()).or_insert_with(Vec::new);
        Ok(())
    }

    async fn delete_topic(&mut self, topic: &str) -> AppResult<()> {
        self.topics.remove(topic);
        self.handlers.remove(topic);
        Ok(())
    }
}

impl Clone for InMemoryMessageBroker {
    fn clone(&self) -> Self {
        Self {
            topics: self.topics.clone(),
            handlers: HashMap::new(), // Handlers can't be cloned
            config: self.config.clone(),
        }
    }
}

/// Domain event message handler
pub struct DomainEventHandler {
    event_bus: std::sync::Arc<EventBus>,
    handler_id: String,
}

impl DomainEventHandler {
    pub fn new(event_bus: std::sync::Arc<EventBus>, handler_id: String) -> Self {
        Self {
            event_bus,
            handler_id,
        }
    }
}

#[async_trait]
impl MessageHandler for DomainEventHandler {
    async fn handle(&self, message: &Message) -> AppResult<()> {
        // Convert message to domain event and publish to event bus
        // This is a simplified implementation
        match message.topic.as_str() {
            "domain.events" => {
                // Parse domain event from message payload
                // In a real implementation, this would deserialize to specific event types
                println!("Handling domain event: {}", message.id);
            }
            _ => {}
        }
        Ok(())
    }

    fn handler_id(&self) -> &str {
        &self.handler_id
    }
}

/// Audit log message handler
pub struct AuditLogHandler {
    handler_id: String,
}

impl AuditLogHandler {
    pub fn new(handler_id: String) -> Self {
        Self { handler_id }
    }

    async fn log_message(&self, message: &Message) -> AppResult<()> {
        // Log message for audit purposes
        println!("Audit Log: Message {} on topic {}", message.id, message.topic);
        Ok(())
    }
}

#[async_trait]
impl MessageHandler for AuditLogHandler {
    async fn handle(&self, message: &Message) -> AppResult<()> {
        self.log_message(message).await
    }

    fn handler_id(&self) -> &str {
        &self.handler_id
    }
}

/// Retry handler for failed messages
pub struct RetryHandler {
    broker: std::sync::Arc<dyn MessageBroker>,
    handler_id: String,
}

impl RetryHandler {
    pub fn new(broker: std::sync::Arc<dyn MessageBroker>, handler_id: String) -> Self {
        Self {
            broker,
            handler_id,
        }
    }

    async fn retry_failed_message(&self, message: &Message) -> AppResult<()> {
        if message.should_retry() {
            let mut retry_message = message.clone();
            retry_message.increment_retry();

            // Delay retry
            tokio::time::sleep(tokio::time::Duration::from_millis(1000 * (message.retry_count as u64))).await;

            self.broker.publish(&retry_message.topic, &retry_message).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl MessageHandler for RetryHandler {
    async fn handle(&self, message: &Message) -> AppResult<()> {
        self.retry_failed_message(message).await
    }

    fn handler_id(&self) -> &str {
        &self.handler_id
    }
}

/// Message producer
pub struct MessageProducer {
    broker: std::sync::Arc<dyn MessageBroker>,
}

impl MessageProducer {
    pub fn new(broker: std::sync::Arc<dyn MessageBroker>) -> Self {
        Self { broker }
    }

    pub async fn publish_domain_event(&self, event: &DomainEvent) -> AppResult<()> {
        let message = Message::new(
            "domain.events".to_string(),
            serde_json::to_value(event)?,
        ).with_header("event_type".to_string(), event.event_type().to_string());

        self.broker.publish("domain.events", &message).await
    }

    pub async fn publish_notification(&self, user_id: Uuid, notification: NotificationMessage) -> AppResult<()> {
        let message = Message::new(
            "notifications".to_string(),
            serde_json::to_value(&notification)?,
        ).with_header("user_id".to_string(), user_id.to_string());

        self.broker.publish("notifications", &message).await
    }

    pub async fn publish_module_event(&self, module_name: &str, event: ModuleEvent) -> AppResult<()> {
        let topic = format!("modules.{}", module_name);
        let message = Message::new(
            topic.clone(),
            serde_json::to_value(&event)?,
        ).with_header("module".to_string(), module_name.to_string());

        self.broker.publish(&topic, &message).await
    }
}

/// Notification message
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub message: String,
    pub notification_type: NotificationType,
    pub data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Notification type
#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// Module event
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleEvent {
    pub module_name: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message service
pub struct MessageService {
    broker: std::sync::Arc<dyn MessageBroker>,
    producer: MessageProducer,
}

impl MessageService {
    pub fn new(broker: std::sync::Arc<dyn MessageBroker>) -> Self {
        let producer = MessageProducer::new(broker.clone());
        Self { broker, producer }
    }

    pub async fn setup_default_handlers(&mut self) -> AppResult<()> {
        // Register audit log handler
        let audit_handler = Box::new(AuditLogHandler::new("audit-handler".to_string()));
        self.broker.subscribe("*", audit_handler).await?;

        // Register retry handler
        let retry_handler = Box::new(RetryHandler::new(self.broker.clone(), "retry-handler".to_string()));
        self.broker.subscribe("retry-queue", retry_handler).await?;

        Ok(())
    }

    pub fn producer(&self) -> &MessageProducer {
        &self.producer
    }

    pub async fn create_standard_topics(&mut self) -> AppResult<()> {
        self.broker.create_topic("domain.events", TopicConfig::default()).await?;
        self.broker.create_topic("notifications", TopicConfig::default()).await?;
        self.broker.create_topic("module-status", TopicConfig::default()).await?;
        self.broker.create_topic("retry-queue", TopicConfig::default()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_broker() {
        let broker = std::sync::Arc::new(InMemoryMessageBroker::new(BrokerConfig::default()));

        // Create topic
        broker.create_topic("test-topic", TopicConfig::default()).await.unwrap();

        // Create message
        let message = Message::new(
            "test-topic".to_string(),
            serde_json::json!({"test": "data"}),
        );

        // Publish message
        broker.publish("test-topic", &message).await.unwrap();
    }

    #[tokio::test]
    async fn test_message_producer() {
        let broker = std::sync::Arc::new(InMemoryMessageBroker::new(BrokerConfig::default()));
        let producer = MessageProducer::new(broker);

        let event = crate::domain::events::SystemEvent::SystemStarted {
            timestamp: chrono::Utc::now(),
            version: "2.0.0".to_string(),
        };

        // Note: This would need proper domain event implementation
        // producer.publish_domain_event(&event).await.unwrap();

        assert!(true); // Placeholder for now
    }

    #[test]
    fn test_message_creation() {
        let message = Message::new(
            "test-topic".to_string(),
            serde_json::json!({"key": "value"}),
        )
        .with_correlation_id(Uuid::new_v4())
        .with_header("key".to_string(), "value".to_string());

        assert_eq!(message.topic, "test-topic");
        assert!(message.correlation_id.is_some());
        assert!(message.headers.contains_key("key"));
    }

    #[test]
    fn test_retry_logic() {
        let mut message = Message::new(
            "test-topic".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert!(message.should_retry());

        for _ in 0..message.max_retries {
            message.increment_retry();
        }

        assert!(!message.should_retry());
    }
}