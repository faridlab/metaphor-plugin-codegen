//! Audit Service
//!
//! Provides auditing capabilities for tracking important events across all modules

use crate::domain::entities::{AuditLog, AuditAction};
use crate::shared::error::{AppError, AppResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Domain service for audit logging
pub struct AuditService {
    // Dependencies would be injected here (e.g., audit repository)
}

impl AuditService {
    pub fn new() -> Self {
        Self {}
    }

    /// Log a user action
    pub fn log_user_action(
        &self,
        user_id: Uuid,
        action: AuditAction,
        resource_type: String,
        resource_id: Option<String>,
        module: Option<String>,
        details: serde_json::Value,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let mut audit_log = AuditLog::new(action, resource_type, resource_id, module, details)
            .with_user(user_id)
            .with_metadata(metadata.ip_address, metadata.user_agent);

        // Additional validation could be added here
        self.validate_audit_log(&audit_log)?;

        Ok(audit_log)
    }

    /// Log a system action (no user)
    pub fn log_system_action(
        &self,
        action: AuditAction,
        resource_type: String,
        resource_id: Option<String>,
        module: Option<String>,
        details: serde_json::Value,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let audit_log = AuditLog::new(action, resource_type, resource_id, module, details)
            .with_metadata(metadata.ip_address, metadata.user_agent);

        self.validate_audit_log(&audit_log)?;

        Ok(audit_log)
    }

    /// Log user authentication
    pub fn log_authentication(
        &self,
        user_id: Uuid,
        action: AuthAction,
        details: serde_json::Value,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let audit_action = match action {
            AuthAction::Login => AuditAction::Login,
            AuthAction::Logout => AuditAction::Logout,
            AuthAction::LoginFailed => AuditAction::Login, // We'll add details to distinguish
        };

        let enhanced_details = json!({
            "auth_action": action,
            "details": details
        });

        self.log_user_action(
            user_id,
            audit_action,
            "authentication".to_string(),
            None,
            Some("security".to_string()),
            enhanced_details,
            metadata,
        )
    }

    /// Log data access
    pub fn log_data_access(
        &self,
        user_id: Option<Uuid>,
        resource_type: String,
        resource_id: Option<String>,
        access_type: DataAccessType,
        module: Option<String>,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let action = match access_type {
            DataAccessType::Read => AuditAction::Read,
            DataAccessType::Export => AuditAction::Export,
            DataAccessType::Import => AuditAction::Import,
        };

        let details = json!({
            "access_type": access_type,
            "resource_type": resource_type,
            "resource_id": resource_id
        });

        let mut audit_log = AuditLog::new(action, resource_type, resource_id, module, details)
            .with_metadata(metadata.ip_address, metadata.user_agent);

        if let Some(uid) = user_id {
            audit_log = audit_log.with_user(uid);
        }

        self.validate_audit_log(&audit_log)?;

        Ok(audit_log)
    }

    /// Log configuration changes
    pub fn log_configuration_change(
        &self,
        user_id: Uuid,
        config_key: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        module: Option<String>,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let details = json!({
            "config_key": config_key,
            "old_value": old_value,
            "new_value": new_value
        });

        self.log_user_action(
            user_id,
            AuditAction::ConfigurationChange,
            "configuration".to_string(),
            None,
            module,
            details,
            metadata,
        )
    }

    /// Log permission changes
    pub fn log_permission_change(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
        change_type: PermissionChangeType,
        permissions: Vec<String>,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let details = json!({
            "target_user_id": target_user_id,
            "change_type": change_type,
            "permissions": permissions
        });

        self.log_user_action(
            user_id,
            AuditAction::PermissionChange,
            "user_permissions".to_string(),
            Some(target_user_id.to_string()),
            Some("security".to_string()),
            details,
            metadata,
        )
    }

    /// Log backup operations
    pub fn log_backup_operation(
        &self,
        user_id: Option<Uuid>,
        operation: BackupOperation,
        backup_name: String,
        details: serde_json::Value,
        metadata: AuditMetadata,
    ) -> AppResult<AuditLog> {
        let action = match operation {
            BackupOperation::Create => AuditAction::Backup,
            BackupOperation::Restore => AuditAction::Restore,
        };

        let enhanced_details = json!({
            "operation": operation,
            "backup_name": backup_name,
            "details": details
        });

        let mut audit_log = AuditLog::new(
            action,
            "backup".to_string(),
            Some(backup_name),
            Some("system".to_string()),
            enhanced_details,
        )
        .with_metadata(metadata.ip_address, metadata.user_agent);

        if let Some(uid) = user_id {
            audit_log = audit_log.with_user(uid);
        }

        self.validate_audit_log(&audit_log)?;

        Ok(audit_log)
    }

    /// Get audit trail for a resource
    pub fn get_resource_audit_trail(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: Option<u32>,
    ) -> AppResult<Vec<AuditLog>> {
        // This would typically query the audit repository
        // For now, returning empty result
        Ok(Vec::new())
    }

    /// Get audit trail for a user
    pub fn get_user_audit_trail(
        &self,
        user_id: &Uuid,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<u32>,
    ) -> AppResult<Vec<AuditLog>> {
        // This would typically query the audit repository
        // For now, returning empty result
        Ok(Vec::new())
    }

    /// Validate audit log data
    fn validate_audit_log(&self, audit_log: &AuditLog) -> AppResult<()> {
        // Validate resource type is not empty
        if audit_log.resource_type.is_empty() {
            return Err(AppError::validation("Resource type cannot be empty"));
        }

        // Validate action is appropriate for resource type
        self.validate_action_resource_combination(&audit_log.action, &audit_log.resource_type)?;

        // Validate details size to prevent oversized logs
        let details_str = serde_json::to_string(&audit_log.details).unwrap_or_default();
        if details_str.len() > 10000 {
            return Err(AppError::validation("Audit log details are too large"));
        }

        Ok(())
    }

    /// Validate that action is appropriate for resource type
    fn validate_action_resource_combination(
        &self,
        action: &AuditAction,
        resource_type: &str,
    ) -> AppResult<()> {
        match action {
            AuditAction::Login | AuditAction::Logout => {
                if resource_type != "authentication" {
                    return Err(AppError::validation(format!(
                        "Auth actions only valid for authentication resource, got: {}",
                        resource_type
                    )));
                }
            }
            AuditAction::SystemStart | AuditAction::SystemStop => {
                if resource_type != "system" {
                    return Err(AppError::validation(format!(
                        "System actions only valid for system resource, got: {}",
                        resource_type
                    )));
                }
            }
            _ => {
                // Most actions are valid for any resource type
            }
        }

        Ok(())
    }

    /// Check if audit logging is enabled for a specific action
    pub fn is_audit_enabled(&self, action: &AuditAction, module: &str) -> bool {
        // In a real implementation, this would check configuration
        // For now, enabling all auditing
        true
    }

    /// Sensitive data filter for audit logs
    pub fn filter_sensitive_data(&self, details: &mut serde_json::Value) {
        // Remove or mask sensitive fields
        if let Some(obj) = details.as_object_mut() {
            let sensitive_fields = vec![
                "password", "token", "secret", "key", "credit_card", "ssn",
            ];

            for field in sensitive_fields {
                if obj.contains_key(field) {
                    obj.insert(field.to_string(), json!("[REDACTED]"));
                }
            }
        }
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for audit events
#[derive(Debug, Clone)]
pub struct AuditMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditMetadata {
    pub fn new(ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            ip_address,
            user_agent,
        }
    }

    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            });

        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            ip_address,
            user_agent,
        }
    }
}

/// Authentication actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthAction {
    Login,
    Logout,
    LoginFailed,
}

/// Data access types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataAccessType {
    Read,
    Export,
    Import,
}

/// Permission change types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionChangeType {
    Grant,
    Revoke,
    Update,
}

/// Backup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupOperation {
    Create,
    Restore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_audit_log_creation() {
        let service = AuditService::new();
        let user_id = Uuid::new_v4();
        let metadata = AuditMetadata::new(
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        let result = service.log_user_action(
            user_id,
            AuditAction::Create,
            "user".to_string(),
            Some("123".to_string()),
            Some("sapiens".to_string()),
            json!({"email": "test@example.com"}),
            metadata,
        );

        assert!(result.is_ok());
        let audit_log = result.unwrap();
        assert_eq!(audit_log.user_id, Some(user_id));
        assert_eq!(audit_log.action, AuditAction::Create);
        assert_eq!(audit_log.resource_type, "user");
        assert_eq!(audit_log.resource_id, Some("123".to_string()));
    }

    #[test]
    fn test_authentication_logging() {
        let service = AuditService::new();
        let user_id = Uuid::new_v4();
        let metadata = AuditMetadata::new(None, None);

        let result = service.log_authentication(
            user_id,
            AuthAction::Login,
            json!({"method": "password"}),
            metadata,
        );

        assert!(result.is_ok());
        let audit_log = result.unwrap();
        assert_eq!(audit_log.action, AuditAction::Login);
        assert_eq!(audit_log.resource_type, "authentication");
    }

    #[test]
    fn test_configuration_change_logging() {
        let service = AuditService::new();
        let user_id = Uuid::new_v4();
        let metadata = AuditMetadata::new(None, None);

        let result = service.log_configuration_change(
            user_id,
            "max_connections".to_string(),
            Some(json!(20)),
            json!(30),
            Some("database".to_string()),
            metadata,
        );

        assert!(result.is_ok());
        let audit_log = result.unwrap();
        assert_eq!(audit_log.action, AuditAction::ConfigurationChange);
        assert_eq!(audit_log.resource_type, "configuration");
    }

    #[test]
    fn test_audit_metadata_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", "Mozilla/5.0".parse().unwrap());
        headers.insert("x-forwarded-for", "203.0.113.1, 192.168.1.1".parse().unwrap());

        let metadata = AuditMetadata::from_headers(&headers);
        assert_eq!(metadata.ip_address, Some("203.0.113.1".to_string()));
        assert_eq!(metadata.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_sensitive_data_filtering() {
        let service = AuditService::new();
        let mut details = json!({
            "username": "testuser",
            "password": "secret123",
            "email": "test@example.com",
            "token": "abc123"
        });

        service.filter_sensitive_data(&mut details);

        assert_eq!(details["username"], "testuser");
        assert_eq!(details["email"], "test@example.com");
        assert_eq!(details["password"], json!("[REDACTED]"));
        assert_eq!(details["token"], json!("[REDACTED]"));
    }

    #[test]
    fn test_action_validation() {
        let service = AuditService::new();
        let user_id = Uuid::new_v4();
        let metadata = AuditMetadata::new(None, None);

        // Valid case
        let valid_result = service.log_user_action(
            user_id,
            AuditAction::Login,
            "authentication".to_string(),
            None,
            None,
            json!({}),
            metadata.clone(),
        );
        assert!(valid_result.is_ok());

        // Invalid case - login action with wrong resource type
        let invalid_result = service.log_user_action(
            user_id,
            AuditAction::Login,
            "user".to_string(),
            None,
            None,
            json!({}),
            metadata,
        );
        assert!(invalid_result.is_err());
    }
}