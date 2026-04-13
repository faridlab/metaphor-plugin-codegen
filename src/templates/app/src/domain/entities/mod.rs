//! Application Domain Entities
//!
//! These are domain entities that exist at the application level and
//! cross multiple bounded contexts (modules).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Application-level system user that represents the application itself
/// for system operations, background jobs, and service-to-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUser {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<SystemPermission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SystemUser {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            permissions: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_permission(&mut self, permission: SystemPermission) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
            self.updated_at = Utc::now();
        }
    }

    pub fn has_permission(&self, permission: &SystemPermission) -> bool {
        self.permissions.contains(permission)
    }
}

/// System-wide permissions for application-level operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemPermission {
    /// Can manage system configuration
    ManageSystem,
    /// Can manage all modules
    ManageModules,
    /// Can view system health and metrics
    ViewSystemHealth,
    /// Can perform system backups and restores
    ManageBackups,
    /// Can manage all users across modules
    ManageAllUsers,
    /// Can send system-wide notifications
    SendNotifications,
    /// Can access system logs
    AccessSystemLogs,
    /// Can perform system maintenance
    SystemMaintenance,
}

/// Application-wide configuration entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: ConfigurationCategory,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl SystemConfiguration {
    pub fn new(
        key: String,
        value: serde_json::Value,
        category: ConfigurationCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key,
            value,
            description: None,
            category,
            is_encrypted: false,
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
}

/// Configuration categories for system-wide settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationCategory {
    /// Server and networking configuration
    Server,
    /// Database and connection settings
    Database,
    /// Security and authentication settings
    Security,
    /// Email and notification settings
    Notifications,
    /// File storage settings
    Storage,
    /// External service integrations
    Integrations,
    /// Performance and monitoring settings
    Performance,
    /// Feature flags and toggles
    Features,
}

/// Module configuration entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfiguration {
    pub id: Uuid,
    pub module_name: String,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub is_sensitive: bool,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<Uuid>,
}

/// Cross-module session entity for managing user sessions across all modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_info: Option<DeviceInfo>,
    pub permissions: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub is_active: bool,
}

impl UserSession {
    pub fn new(user_id: Uuid, token_hash: String, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            device_info: None,
            permissions: vec![],
            expires_at,
            created_at: now,
            last_accessed_at: now,
            is_active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn refresh_expiration(&mut self, new_expires_at: DateTime<Utc>) {
        self.expires_at = new_expires_at;
        self.last_accessed_at = Utc::now();
    }

    pub fn revoke(&mut self) {
        self.is_active = false;
    }
}

/// Device information for tracking user sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub device_name: Option<String>,
    pub operating_system: Option<String>,
    pub browser: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Api,
    Unknown,
}

/// Cross-module audit log entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub module: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        action: AuditAction,
        resource_type: String,
        resource_id: Option<String>,
        module: Option<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            action,
            resource_type,
            resource_id,
            module,
            details,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_metadata(mut self, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }
}

/// Audit actions for system-wide logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Read,
    Login,
    Logout,
    Export,
    Import,
    Backup,
    Restore,
    ConfigurationChange,
    PermissionChange,
    SystemStart,
    SystemStop,
}

impl SystemConfiguration {
    pub fn get_string_value(&self) -> Option<String> {
        self.value.as_str().map(|s| s.to_string())
    }

    pub fn get_bool_value(&self) -> Option<bool> {
        self.value.as_bool()
    }

    pub fn get_int_value(&self) -> Option<i64> {
        self.value.as_i64()
    }

    pub fn update_value(&mut self, new_value: serde_json::Value, updated_by: Uuid) {
        self.value = new_value;
        self.updated_at = Utc::now();
        self.updated_by = Some(updated_by);
    }
}