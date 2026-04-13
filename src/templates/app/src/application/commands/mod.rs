//! Application Commands
//!
//! Command objects that represent write operations
//! in the application layer following CQRS pattern.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command types for classification and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    UserCommand,
    SystemCommand,
    ConfigurationCommand,
    AuditCommand,
    ModuleCommand,
}

/// Base command trait
pub trait Command {
    /// Get the command ID
    fn id(&self) -> Uuid;

    /// Get the command timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the command type
    fn command_type(&self) -> &'static str;
}

/// User management commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserCommand {
    /// Create a new user
    CreateUser {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        email: String,
        password: String,
        username: String,
        first_name: String,
        last_name: String,
        roles: Vec<String>,
    },

    /// Update user information
    UpdateUser {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        updates: UserUpdateData,
    },

    /// Delete a user
    DeleteUser {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        reason: String,
    },

    /// Change user roles
    ChangeUserRoles {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        roles: Vec<String>,
    },

    /// Reset user password
    ResetPassword {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        new_password: String,
        reset_token: String,
    },
}

/// User update data for UpdateUser command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

impl Command for UserCommand {
    fn id(&self) -> Uuid {
        match self {
            UserCommand::CreateUser { id, .. }
            | UserCommand::UpdateUser { id, .. }
            | UserCommand::DeleteUser { id, .. }
            | UserCommand::ChangeUserRoles { id, .. }
            | UserCommand::ResetPassword { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            UserCommand::CreateUser { timestamp, .. }
            | UserCommand::UpdateUser { timestamp, .. }
            | UserCommand::DeleteUser { timestamp, .. }
            | UserCommand::ChangeUserRoles { timestamp, .. }
            | UserCommand::ResetPassword { timestamp, .. } => *timestamp,
        }
    }

    fn command_type(&self) -> &'static str {
        match self {
            UserCommand::CreateUser { .. } => "user.create",
            UserCommand::UpdateUser { .. } => "user.update",
            UserCommand::DeleteUser { .. } => "user.delete",
            UserCommand::ChangeUserRoles { .. } => "user.change_roles",
            UserCommand::ResetPassword { .. } => "user.reset_password",
        }
    }
}

/// System configuration commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    /// Update system configuration
    UpdateConfiguration {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        key: String,
        value: serde_json::Value,
        updated_by: Uuid,
    },

    /// Create system backup
    CreateBackup {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        backup_name: String,
        description: Option<String>,
        created_by: Option<Uuid>,
    },

    /// Restore system from backup
    RestoreBackup {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        backup_name: String,
        restored_by: Option<Uuid>,
    },

    /// Enable/disable module
    ToggleModule {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        module_name: String,
        enabled: bool,
        reason: Option<String>,
    },
}

impl Command for SystemCommand {
    fn id(&self) -> Uuid {
        match self {
            SystemCommand::UpdateConfiguration { id, .. }
            | SystemCommand::CreateBackup { id, .. }
            | SystemCommand::RestoreBackup { id, .. }
            | SystemCommand::ToggleModule { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SystemCommand::UpdateConfiguration { timestamp, .. }
            | SystemCommand::CreateBackup { timestamp, .. }
            | SystemCommand::RestoreBackup { timestamp, .. }
            | SystemCommand::ToggleModule { timestamp, .. } => *timestamp,
        }
    }

    fn command_type(&self) -> &'static str {
        match self {
            SystemCommand::UpdateConfiguration { .. } => "system.update_configuration",
            SystemCommand::CreateBackup { .. } => "system.create_backup",
            SystemCommand::RestoreBackup { .. } => "system.restore_backup",
            SystemCommand::ToggleModule { .. } => "system.toggle_module",
        }
    }
}

/// Session management commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCommand {
    /// Create new user session
    CreateSession {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        token_hash: String,
        device_info: Option<DeviceInfo>,
        permissions: Vec<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
    },

    /// Revoke user session
    RevokeSession {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        session_id: Uuid,
        reason: String,
    },

    /// Revoke all user sessions
    RevokeAllUserSessions {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        reason: String,
    },

    /// Extend session expiration
    ExtendSession {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        session_id: Uuid,
        new_expires_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Device information for session commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub device_name: Option<String>,
    pub operating_system: Option<String>,
    pub browser: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Command for SessionCommand {
    fn id(&self) -> Uuid {
        match self {
            SessionCommand::CreateSession { id, .. }
            | SessionCommand::RevokeSession { id, .. }
            | SessionCommand::RevokeAllUserSessions { id, .. }
            | SessionCommand::ExtendSession { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SessionCommand::CreateSession { timestamp, .. }
            | SessionCommand::RevokeSession { timestamp, .. }
            | SessionCommand::RevokeAllUserSessions { timestamp, .. }
            | SessionCommand::ExtendSession { timestamp, .. } => *timestamp,
        }
    }

    fn command_type(&self) -> &'static str {
        match self {
            SessionCommand::CreateSession { .. } => "session.create",
            SessionCommand::RevokeSession { .. } => "session.revoke",
            SessionCommand::RevokeAllUserSessions { .. } => "session.revoke_all",
            SessionCommand::ExtendSession { .. } => "session.extend",
        }
    }
}

/// Audit commands for logging and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditCommand {
    /// Log user action
    LogUserAction {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        action: String,
        resource_type: String,
        resource_id: Option<String>,
        details: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    },

    /// Log system action
    LogSystemAction {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        action: String,
        resource_type: String,
        resource_id: Option<String>,
        details: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
    },
}

impl Command for AuditCommand {
    fn id(&self) -> Uuid {
        match self {
            AuditCommand::LogUserAction { id, .. }
            | AuditCommand::LogSystemAction { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            AuditCommand::LogUserAction { timestamp, .. }
            | AuditCommand::LogSystemAction { timestamp, .. } => *timestamp,
        }
    }

    fn command_type(&self) -> &'static str {
        match self {
            AuditCommand::LogUserAction { .. } => "audit.log_user_action",
            AuditCommand::LogSystemAction { .. } => "audit.log_system_action",
        }
    }
}

/// Command validation trait
pub trait ValidatableCommand {
    /// Validate the command
    fn validate(&self) -> crate::shared::error::AppResult<()>;
}

/// Command validator
pub struct CommandValidator;

impl CommandValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandValidator {
    pub fn validate_command<C: ValidatableCommand>(&self, command: &C) -> crate::shared::error::AppResult<()> {
        command.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_command_properties() {
        let command = UserCommand::CreateUser {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            roles: vec!["user".to_string()],
        };

        assert_eq!(command.command_type(), "user.create");
        assert_eq!(command.email, "test@example.com");
    }

    #[test]
    fn test_system_command_properties() {
        let command = SystemCommand::UpdateConfiguration {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            key: "app.name".to_string(),
            value: serde_json::json!("Metaphor"),
            updated_by: Uuid::new_v4(),
        };

        assert_eq!(command.command_type(), "system.update_configuration");
        assert_eq!(command.key, "app.name");
    }

    #[test]
    fn test_session_command_properties() {
        let command = SessionCommand::CreateSession {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            device_info: None,
            permissions: vec!["read".to_string()],
            expires_at: Utc::now() + chrono::Duration::hours(24),
        };

        assert_eq!(command.command_type(), "session.create");
        assert_eq!(command.user_id, Uuid::new_v4());
    }
}