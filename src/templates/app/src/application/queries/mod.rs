//! Application Queries
//!
//! Query objects that represent read operations
//! in the application layer following CQRS pattern.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Query types for classification and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    UserQuery,
    SystemQuery,
    ConfigurationQuery,
    AuditQuery,
    ModuleQuery,
}

/// Base query trait
pub trait Query {
    /// Get the query ID
    fn id(&self) -> Uuid;

    /// Get the query timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the query type
    fn query_type(&self) -> &'static str;
}

/// User queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserQuery {
    /// Get user by ID
    GetUser {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
    },

    /// Get user by email
    GetUserByEmail {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        email: String,
    },

    /// List users with pagination
    ListUsers {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        page: u32,
        limit: u32,
        sort_by: Option<String>,
        sort_order: Option<String>,
        filters: serde_json::Value,
    },

    /// Search users
    SearchUsers {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        query: String,
        page: u32,
        limit: u32,
        filters: serde_json::Value,
    },

    /// Get user roles
    GetUserRoles {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
    },

    /// Get user permissions
    GetUserPermissions {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
    },

    /// Get user sessions
    GetUserSessions {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        active_only: bool,
    },

    /// Check if email exists
    CheckEmailExists {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        email: String,
    },

    /// Check if username exists
    CheckUsernameExists {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        username: String,
    },
}

impl Query for UserQuery {
    fn id(&self) -> Uuid {
        match self {
            UserQuery::GetUser { id, .. }
            | UserQuery::GetUserByEmail { id, .. }
            | UserQuery::ListUsers { id, .. }
            | UserQuery::SearchUsers { id, .. }
            | UserQuery::GetUserRoles { id, .. }
            | UserQuery::GetUserPermissions { id, .. }
            | UserQuery::GetUserSessions { id, .. }
            | UserQuery::CheckEmailExists { id, .. }
            | UserQuery::CheckUsernameExists { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            UserQuery::GetUser { timestamp, .. }
            | UserQuery::GetUserByEmail { timestamp, .. }
            | UserQuery::ListUsers { timestamp, .. }
            | UserQuery::SearchUsers { timestamp, .. }
            | UserQuery::GetUserRoles { timestamp, .. }
            | UserQuery::GetUserPermissions { timestamp, .. }
            | UserQuery::GetUserSessions { timestamp, .. }
            | UserQuery::CheckEmailExists { timestamp, .. }
            | UserQuery::CheckUsernameExists { timestamp, .. } => *timestamp,
        }
    }

    fn query_type(&self) -> &'static str {
        match self {
            UserQuery::GetUser { .. } => "user.get",
            UserQuery::GetUserByEmail { .. } => "user.get_by_email",
            UserQuery::ListUsers { .. } => "user.list",
            UserQuery::SearchUsers { .. } => "user.search",
            UserQuery::GetUserRoles { .. } => "user.get_roles",
            UserQuery::GetUserPermissions { .. } => "user.get_permissions",
            UserQuery::GetUserSessions { .. } => "user.get_sessions",
            UserQuery::CheckEmailExists { .. } => "user.check_email_exists",
            UserQuery::CheckUsernameExists { .. } => "user.check_username_exists",
        }
    }
}

/// System configuration queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemQuery {
    /// Get configuration by key
    GetConfiguration {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        key: String,
    },

    /// List all configurations
    ListConfigurations {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        category: Option<String>,
    },

    /// Get configuration by category
    GetConfigurationByCategory {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        category: String,
    },

    /// Get system health status
    GetSystemHealth {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Get module status
    GetModuleStatus {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        module_name: Option<String>,
    },

    /// Get system metrics
    GetSystemMetrics {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        metric_types: Option<Vec<String>>,
    },
}

impl Query for SystemQuery {
    fn id(&self) -> Uuid {
        match self {
            SystemQuery::GetConfiguration { id, .. }
            | SystemQuery::ListConfigurations { id, .. }
            | SystemQuery::GetConfigurationByCategory { id, .. }
            | SystemQuery::GetSystemHealth { id, .. }
            | SystemQuery::GetModuleStatus { id, .. }
            | SystemQuery::GetSystemMetrics { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SystemQuery::GetConfiguration { timestamp, .. }
            | SystemQuery::ListConfigurations { timestamp, .. }
            | SystemQuery::GetConfigurationByCategory { timestamp, .. }
            | SystemQuery::GetSystemHealth { timestamp, .. }
            | SystemQuery::GetModuleStatus { timestamp, .. }
            | SystemQuery::GetSystemMetrics { timestamp, .. } => *timestamp,
        }
    }

    fn query_type(&self) -> &'static str {
        match self {
            SystemQuery::GetConfiguration { .. } => "system.get_configuration",
            SystemQuery::ListConfigurations { .. } => "system.list_configurations",
            SystemQuery::GetConfigurationByCategory { .. } => "system.get_configuration_by_category",
            SystemQuery::GetSystemHealth { .. } => "system.get_health",
            SystemQuery::GetModuleStatus { .. } => "system.get_module_status",
            SystemQuery::GetSystemMetrics { .. } => "system.get_metrics",
        }
    }
}

/// Audit and logging queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditQuery {
    /// Get audit log by ID
    GetAuditLog {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        log_id: Uuid,
    },

    /// Get audit logs by user
    GetAuditLogsByUser {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        user_id: Uuid,
        page: u32,
        limit: u32,
    },

    /// Get audit logs by date range
    GetAuditLogsByDateRange {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        page: u32,
        limit: u32,
    },

    /// Get audit logs by resource
    GetAuditLogsByResource {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        resource_type: String,
        resource_id: Option<String>,
        page: u32,
        limit: u32,
    },

    /// Search audit logs
    SearchAuditLogs {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        search_criteria: serde_json::Value,
        page: u32,
        limit: u32,
    },

    /// Get audit statistics
    GetAuditStatistics {
        id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
        period: Option<String>, // "day", "week", "month"
    },
}

impl Query for AuditQuery {
    fn id(&self) -> Uuid {
        match self {
            AuditQuery::GetAuditLog { id, .. }
            | AuditQuery::GetAuditLogsByUser { id, .. }
            | AuditQuery::GetAuditLogsByDateRange { id, .. }
            | AuditQuery::GetAuditLogsByResource { id, .. }
            | AuditQuery::SearchAuditLogs { id, .. }
            | AuditQuery::GetAuditStatistics { id, .. } => *id,
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            AuditQuery::GetAuditLog { timestamp, .. }
            | AuditQuery::GetAuditLogsByUser { timestamp, .. }
            | AuditQuery::GetAuditLogsByDateRange { timestamp, .. }
            | AuditQuery::GetAuditLogsByResource { timestamp, .. }
            | AuditQuery::SearchAuditLogs { timestamp, .. }
            | AuditQuery::GetAuditStatistics { timestamp, .. } => *timestamp,
        }
    }

    fn query_type(&self) -> &'static str {
        match self {
            AuditQuery::GetAuditLog { .. } => "audit.get_log",
            AuditQuery::GetAuditLogsByUser { .. } => "audit.get_logs_by_user",
            AuditQuery::GetAuditLogsByDateRange { .. } => "audit.get_logs_by_date_range",
            AuditQuery::GetAuditLogsByResource { .. } => "audit.get_logs_by_resource",
            AuditQuery::SearchAuditLogs { .. } => "audit.search_logs",
            AuditQuery::GetAuditStatistics { .. } => "audit.get_statistics",
        }
    }
}

/// Base query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub query_type: String,
    pub data: T,
    pub execution_time_ms: u64,
}

/// Paginated query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedQueryResult<T> {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub query_type: String,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    pub execution_time_ms: u64,
}

/// Pagination information for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Query validation trait
pub trait ValidatableQuery {
    /// Validate the query
    fn validate(&self) -> crate::shared::error::AppResult<()>;
}

/// Query validator
pub struct QueryValidator;

impl QueryValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for QueryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryValidator {
    pub fn validate_query<Q: ValidatableQuery>(&self, query: &Q) -> crate::shared::error::AppResult<()> {
        query.validate()
    }
}

/// Query cache for performance optimization
pub struct QueryCache {
    // In a real implementation, this would cache query results
}

impl QueryCache {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get<T>(&self, query_id: &str) -> Option<QueryResult<T>> {
        // In a real implementation, this would check cache
        None
    }

    pub fn set<T>(&self, query_id: &str, result: QueryResult<T>) {
        // In a real implementation, this would store in cache
    }

    pub fn invalidate(&self, query_id: &str) {
        // In a real implementation, this would invalidate cache entry
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_query_properties() {
        let query = UserQuery::GetUser {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: Uuid::new_v4(),
        };

        assert_eq!(query.query_type(), "user.get");
        assert_eq!(query.user_id, Uuid::new_v4());
    }

    #[test]
    fn test_list_users_query() {
        let query = UserQuery::ListUsers {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            page: 1,
            limit: 20,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
            filters: serde_json::json!({"active": true}),
        };

        assert_eq!(query.query_type(), "user.list");
        assert_eq!(query.page, 1);
        assert_eq!(query.limit, 20);
    }

    #[test]
    fn test_system_query_properties() {
        let query = SystemQuery::GetConfiguration {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            key: "app.name".to_string(),
        };

        assert_eq!(query.query_type(), "system.get_configuration");
        assert_eq!(query.key, "app.name");
    }

    #[test]
    fn test_audit_query_properties() {
        let query = AuditQuery::GetAuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            log_id: Uuid::new_v4(),
        };

        assert_eq!(query.query_type(), "audit.get_log");
        assert_eq!(query.log_id, Uuid::new_v4());
    }

    #[test]
    fn test_query_result() {
        let result: QueryResult<String> = QueryResult {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            query_type: "test".to_string(),
            data: "result".to_string(),
            execution_time_ms: 45,
        };

        assert_eq!(result.data, "result");
        assert_eq!(result.execution_time_ms, 45);
    }

    #[test]
    fn test_paginated_result() {
        let pagination = PaginationInfo {
            page: 1,
            limit: 20,
            total: 100,
            total_pages: 5,
            has_next: true,
            has_prev: false,
        };

        let result: PaginatedQueryResult<String> = PaginatedQueryResult {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            query_type: "test".to_string(),
            data: vec!["item1".to_string(), "item2".to_string()],
            pagination,
            execution_time_ms: 120,
        };

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.pagination.page, 1);
        assert_eq!(result.pagination.total_pages, 5);
        assert!(result.pagination.has_next);
        assert!(!result.pagination.has_prev);
    }
}