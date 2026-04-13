//! Domain Repository Interfaces
//!
//! Repository interfaces that define contracts for data access
//! across the application domain.

use uuid::Uuid;
use async_trait::async_trait;
use crate::domain::entities::{SystemUser, UserSession, AuditLog, ModuleConfiguration};

/// Generic repository interface for domain entities
#[async_trait]
pub trait Repository<T, ID> {
    /// Find an entity by its ID
    async fn find_by_id(&self, id: ID) -> crate::shared::error::AppResult<Option<T>>;

    /// Save an entity (create or update)
    async fn save(&self, entity: &T) -> crate::shared::error::AppResult<T>;

    /// Delete an entity by its ID
    async fn delete(&self, id: ID) -> crate::shared::error::AppResult<bool>;

    /// List entities with pagination
    async fn list(&self, limit: u32, offset: u32) -> crate::shared::error::AppResult<Vec<T>>;

    /// Count total entities
    async fn count(&self) -> crate::shared::error::AppResult<u64>;
}

/// User session repository interface
#[async_trait]
pub trait UserSessionRepository: Repository<crate::domain::entities::UserSession, String> {
    /// Find active session by token hash
    async fn find_by_token_hash(&self, token_hash: &str) -> crate::shared::error::AppResult<Option<crate::domain::entities::UserSession>>;

    /// Find sessions by user ID
    async fn find_by_user_id(&self, user_id: Uuid) -> crate::shared::error::AppResult<Vec<crate::domain::entities::UserSession>>;

    /// Revoke all sessions for a user
    async fn revoke_all_for_user(&self, user_id: Uuid) -> crate::shared::error::AppResult<u32>;

    /// Cleanup expired sessions
    async fn cleanup_expired(&self) -> crate::shared::error::AppResult<u32>;
}

/// System configuration repository interface
#[async_trait]
pub trait SystemConfigurationRepository: Repository<crate::domain::entities::SystemConfiguration, String> {
    /// Find configuration by key
    async fn find_by_key(&self, key: &str) -> crate::shared::error::AppResult<Option<crate::domain::entities::SystemConfiguration>>;

    /// Find configurations by category
    async fn find_by_category(&self, category: &crate::domain::value_objects::ConfigurationCategory) -> crate::shared::error::AppResult<Vec<crate::domain::entities::SystemConfiguration>>;

    /// Update configuration value
    async fn update_value(&self, key: &str, value: serde_json::Value, updated_by: Uuid) -> crate::shared::error::AppResult<crate::domain::entities::SystemConfiguration>;
}

/// Audit log repository interface
#[async_trait]
pub trait AuditLogRepository: Repository<crate::domain::entities::AuditLog, Uuid> {
    /// Create audit log entry
    async fn create(&self, audit_log: crate::domain::entities::AuditLog) -> crate::shared::error::AppResult<crate::domain::entities::AuditLog>;

    /// Find audit logs by user ID
    async fn find_by_user_id(&self, user_id: Uuid, limit: u32) -> crate::shared::error::AppResult<Vec<crate::domain::entities::AuditLog>>;

    /// Find audit logs by resource
    async fn find_by_resource(&self, resource_type: &str, resource_id: Option<&str>, limit: u32) -> crate::shared::error::AppResult<Vec<crate::domain::entities::AuditLog>>;

    /// Find audit logs by date range
    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        limit: u32,
    ) -> crate::shared::error::AppResult<Vec<crate::domain::entities::AuditLog>>;
}

/// Repository interface for SystemUser entity
#[async_trait]
pub trait SystemUserRepository {
    async fn save(&self, user: &SystemUser) -> crate::shared::error::AppResult<SystemUser>;
    async fn find_by_id(&self, id: &Uuid) -> crate::shared::error::AppResult<Option<SystemUser>>;
    async fn find_by_email(&self, email: &str) -> crate::shared::error::AppResult<Option<SystemUser>>;
    async fn find_all(&self, limit: Option<u32>, offset: Option<u32>) -> crate::shared::error::AppResult<Vec<SystemUser>>;
    async fn delete(&self, id: &Uuid) -> crate::shared::error::AppResult<bool>;
}


/// Repository interface for AuditLog entity
#[async_trait]
pub trait AuditLogRepository {
    async fn save(&self, audit_log: &AuditLog) -> crate::shared::error::AppResult<AuditLog>;
    async fn find_by_user_id(&self, user_id: &Uuid, page: u32, limit: u32) -> crate::shared::error::AppResult<Vec<AuditLog>>;
    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        page: u32,
        limit: u32,
    ) -> crate::shared::error::AppResult<Vec<AuditLog>>;
    async fn find_by_resource(
        &self,
        resource_type: &str,
        resource_id: Option<&str>,
        page: u32,
        limit: u32,
    ) -> crate::shared::error::AppResult<Vec<AuditLog>>;
}

/// Repository interface for ModuleConfiguration entity
#[async_trait]
pub trait ModuleConfigurationRepository {
    async fn save(&self, config: &ModuleConfiguration) -> crate::shared::error::AppResult<ModuleConfiguration>;
    async fn find_by_id(&self, id: &Uuid) -> crate::shared::error::AppResult<Option<ModuleConfiguration>>;
    async fn find_by_module_and_key(&self, module_name: &str, config_key: &str) -> crate::shared::error::AppResult<Option<ModuleConfiguration>>;
    async fn find_by_module(&self, module_name: &str) -> crate::shared::error::AppResult<Vec<ModuleConfiguration>>;
    async fn delete(&self, id: &Uuid) -> crate::shared::error::AppResult<bool>;
}