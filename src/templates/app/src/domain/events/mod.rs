//! Domain Events
//!
//! Domain events that represent important business events
//! occurring in the application domain layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base domain event trait
pub trait DomainEvent {
    /// Get the event ID
    fn id(&self) -> Uuid;

    /// Get the event timestamp
    fn timestamp(&self) -> DateTime<Utc>;

    /// Get the event type
    fn event_type(&self) -> &'static str;

    /// Get the aggregate ID that generated this event
    fn aggregate_id(&self) -> Option<String>;
}

/// User-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    /// User account created
    UserCreated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        email: String,
        roles: Vec<String>,
    },

    /// User account updated
    UserUpdated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        changes: Vec<String>,
    },

    /// User account deleted
    UserDeleted {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        reason: String,
    },

    /// User roles changed
    UserRolesChanged {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        old_roles: Vec<String>,
        new_roles: Vec<String>,
    },

    /// User session created
    UserSessionCreated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        session_id: Uuid,
        device_info: Option<String>,
    },

    /// User session revoked
    UserSessionRevoked {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        session_id: Uuid,
        reason: String,
    },
}

/// System-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// System configuration changed
    ConfigurationChanged {
        id: Uuid,
        timestamp: DateTime<Utc>,
        key: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        changed_by: Uuid,
    },

    /// System module status changed
    ModuleStatusChanged {
        id: Uuid,
        timestamp: DateTime<Utc>,
        module_name: String,
        old_status: String,
        new_status: String,
    },

    /// System backup created
    BackupCreated {
        id: Uuid,
        timestamp: DateTime<Utc>,
        backup_name: String,
        size: u64,
        created_by: Option<Uuid>,
    },

    /// System backup restored
    BackupRestored {
        id: Uuid,
        timestamp: DateTime<Utc>,
        backup_name: String,
        restored_by: Option<Uuid>,
    },
}

/// Security-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Failed authentication attempt
    AuthenticationFailed {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Option<Uuid>,
        email: Option<String>,
        ip_address: Option<String>,
        reason: String,
    },

    /// Permission changed
    PermissionChanged {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Uuid,
        target_user_id: Uuid,
        change_type: String,
        permissions: Vec<String>,
    },

    /// Suspicious activity detected
    SuspiciousActivity {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Option<Uuid>,
        activity_type: String,
        details: serde_json::Value,
        ip_address: Option<String>,
    },
}

impl DomainEvent for UserEvent {
    fn id(&self) -> Uuid {
        match self {
            UserEvent::UserCreated { id, .. }
            | UserEvent::UserUpdated { id, .. }
            | UserEvent::UserDeleted { id, .. }
            | UserEvent::UserRolesChanged { id, .. }
            | UserEvent::UserSessionCreated { id, .. }
            | UserEvent::UserSessionRevoked { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            UserEvent::UserCreated { timestamp, .. }
            | UserEvent::UserUpdated { timestamp, .. }
            | UserEvent::UserDeleted { timestamp, .. }
            | UserEvent::UserRolesChanged { timestamp, .. }
            | UserEvent::UserSessionCreated { timestamp, .. }
            | UserEvent::UserSessionRevoked { timestamp, .. } => *timestamp,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            UserEvent::UserCreated { .. } => "user.created",
            UserEvent::UserUpdated { .. } => "user.updated",
            UserEvent::UserDeleted { .. } => "user.deleted",
            UserEvent::UserRolesChanged { .. } => "user.roles_changed",
            UserEvent::UserSessionCreated { .. } => "user.session_created",
            UserEvent::UserSessionRevoked { .. } => "user.session_revoked",
        }
    }

    fn aggregate_id(&self) -> Option<String> {
        match self {
            UserEvent::UserCreated { user_id, .. }
            | UserEvent::UserUpdated { user_id, .. }
            | UserEvent::UserDeleted { user_id, .. }
            | UserEvent::UserRolesChanged { user_id, .. }
            | UserEvent::UserSessionCreated { user_id, .. }
            | UserEvent::UserSessionRevoked { user_id, .. } => Some(user_id.to_string()),
        }
    }
}

impl DomainEvent for SystemEvent {
    fn id(&self) -> Uuid {
        match self {
            SystemEvent::ConfigurationChanged { id, .. }
            | SystemEvent::ModuleStatusChanged { id, .. }
            | SystemEvent::BackupCreated { id, .. }
            | SystemEvent::BackupRestored { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SystemEvent::ConfigurationChanged { timestamp, .. }
            | SystemEvent::ModuleStatusChanged { timestamp, .. }
            | SystemEvent::BackupCreated { timestamp, .. }
            | SystemEvent::BackupRestored { timestamp, .. } => *timestamp,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            SystemEvent::ConfigurationChanged { .. } => "system.configuration_changed",
            SystemEvent::ModuleStatusChanged { .. } => "system.module_status_changed",
            SystemEvent::BackupCreated { .. } => "system.backup_created",
            SystemEvent::BackupRestored { .. } => "system.backup_restored",
        }
    }

    fn aggregate_id(&self) -> Option<String> {
        match self {
            SystemEvent::ConfigurationChanged { key, .. } => Some(format!("system:{}", key)),
            SystemEvent::ModuleStatusChanged { module_name, .. } => Some(format!("module:{}", module_name)),
            SystemEvent::BackupCreated { backup_name, .. }
            | SystemEvent::BackupRestored { backup_name, .. } => Some(format!("backup:{}", backup_name)),
        }
    }
}

impl DomainEvent for SecurityEvent {
    fn id(&self) -> Uuid {
        match self {
            SecurityEvent::AuthenticationFailed { id, .. }
            | SecurityEvent::PermissionChanged { id, .. }
            | SecurityEvent::SuspiciousActivity { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SecurityEvent::AuthenticationFailed { timestamp, .. }
            | SecurityEvent::PermissionChanged { timestamp, .. }
            | SecurityEvent::SuspiciousActivity { timestamp, .. } => *timestamp,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            SecurityEvent::AuthenticationFailed { .. } => "security.authentication_failed",
            SecurityEvent::PermissionChanged { .. } => "security.permission_changed",
            SecurityEvent::SuspiciousActivity { .. } => "security.suspicious_activity",
        }
    }

    fn aggregate_id(&self) -> Option<String> {
        match self {
            SecurityEvent::AuthenticationFailed { user_id, .. }
            | SecurityEvent::PermissionChanged { user_id, .. }
            | SecurityEvent::SuspiciousActivity { user_id, .. } => {
                user_id.map(|id| format!("security:{}", id))
            }
        }
    }
}

/// Event bus interface for publishing domain events
#[async_trait]
pub trait EventBus {
    /// Publish a domain event
    async fn publish<E: DomainEvent + Send + Sync>(&self, event: &E) -> crate::shared::error::AppResult<()>;

    /// Publish multiple domain events
    async fn publish_batch<E: DomainEvent + Send + Sync>(&self, events: &[E]) -> crate::shared::error::AppResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

/// In-memory event bus implementation for testing
pub struct InMemoryEventBus {
    // In a real implementation, this would have subscribers and message passing
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish<E: DomainEvent + Send + Sync>(&self, event: &E) -> crate::shared::error::AppResult<()> {
        tracing::info!(
            "📢 Publishing event: {} (ID: {})",
            event.event_type(),
            event.id()
        );
        // In a real implementation, this would send the event to subscribers
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_event_properties() {
        let event = UserEvent::UserCreated {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
        };

        assert_eq!(event.event_type(), "user.created");
        assert_eq!(event.aggregate_id(), Some(event.user_id.to_string()));
    }

    #[test]
    fn test_system_event_properties() {
        let event = SystemEvent::ConfigurationChanged {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            key: "app.name".to_string(),
            old_value: None,
            new_value: serde_json::json!("Metaphor"),
            changed_by: Uuid::new_v4(),
        };

        assert_eq!(event.event_type(), "system.configuration_changed");
        assert_eq!(event.aggregate_id(), Some("system:app.name".to_string()));
    }
}