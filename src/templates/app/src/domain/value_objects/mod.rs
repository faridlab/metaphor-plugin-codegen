//! Application Domain Value Objects
//!
//! These are value objects that are used across the application and don't have
//! their own identity. They are immutable and defined by their attributes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Email address value object with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub value: String,
    pub verified: bool,
    pub primary: bool,
}

impl Email {
    pub fn new(email: &str) -> Result<Self, crate::shared::error::AppError> {
        if Self::is_valid_email(email) {
            Ok(Self {
                value: email.to_lowercase(),
                verified: false,
                primary: false,
            })
        } else {
            Err(crate::shared::error::AppError::validation(format!(
                "Invalid email address: {}",
                email
            )))
        }
    }

    pub fn verified(email: &str) -> Result<Self, crate::shared::error::AppError> {
        let mut email = Self::new(email)?;
        email.verified = true;
        Ok(email)
    }

    pub fn primary(email: &str) -> Result<Self, crate::shared::error::AppError> {
        let mut email = Self::new(email)?;
        email.primary = true;
        Ok(email)
    }

    fn is_valid_email(email: &str) -> bool {
        // Simple email validation - in production use proper regex
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// Role identifier with hierarchy support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub level: u8,
    pub module: Option<String>,
}

impl Role {
    pub fn new(name: String, level: u8) -> Self {
        Self {
            name,
            level,
            module: None,
        }
    }

    pub fn module_role(name: String, level: u8, module: String) -> Self {
        Self {
            name,
            level,
            module: Some(module),
        }
    }

    pub fn is_system_role(&self) -> bool {
        self.module.is_none()
    }

    pub fn is_module_role(&self, module_name: &str) -> bool {
        self.module.as_ref().map_or(false, |m| m == module_name)
    }

    pub fn has_higher_privilege_than(&self, other: &Role) -> bool {
        self.level > other.level
    }
}

impl PartialEq for Role {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.module == other.module
    }
}

/// Permission with resource-level granularity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub action: String,
    pub resource: String,
    pub scope: PermissionScope,
}

impl Permission {
    pub fn new(action: String, resource: String, scope: PermissionScope) -> Self {
        Self {
            action,
            resource,
            scope,
        }
    }

    pub fn global(action: String, resource: String) -> Self {
        Self::new(action, resource, PermissionScope::Global)
    }

    pub fn module(action: String, resource: String, module: String) -> Self {
        Self::new(action, resource, PermissionScope::Module(module))
    }

    pub fn resource(action: String, resource: String, resource_id: String) -> Self {
        Self::new(action, resource, PermissionScope::Resource(resource_id))
    }
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        self.action == other.action
            && self.resource == other.resource
            && self.scope == other.scope
    }
}

/// Permission scope defines where a permission applies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    /// Applies globally across the entire application
    Global,
    /// Applies within a specific module
    Module(String),
    /// Applies to a specific resource instance
    Resource(String),
}

impl PartialEq for PermissionScope {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PermissionScope::Global, PermissionScope::Global) => true,
            (PermissionScope::Module(a), PermissionScope::Module(b)) => a == b,
            (PermissionScope::Resource(a), PermissionScope::Resource(b)) => a == b,
            _ => false,
        }
    }
}

/// File metadata value object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub content_type: String,
    pub size: u64,
    pub hash: String,
    pub uploaded_at: DateTime<Utc>,
    pub module: Option<String>,
}

impl FileMetadata {
    pub fn new(
        name: String,
        content_type: String,
        size: u64,
        hash: String,
        module: Option<String>,
    ) -> Self {
        Self {
            name,
            content_type,
            size,
            hash,
            uploaded_at: Utc::now(),
            module,
        }
    }

    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    pub fn is_document(&self) -> bool {
        matches!(
            self.content_type.as_str(),
            "application/pdf" | "text/plain" | "application/msword"
        )
    }

    pub fn human_readable_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Address value object with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub coordinates: Option<Coordinates>,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        postal_code: String,
        country: String,
    ) -> Self {
        Self {
            street,
            city,
            state: None,
            postal_code,
            country,
            coordinates: None,
        }
    }

    pub fn with_state(mut self, state: String) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.coordinates = Some(Coordinates { latitude, longitude });
        self
    }

    pub fn full_address(&self) -> String {
        let mut parts = vec![&self.street, &self.city];
        if let Some(ref state) = self.state {
            parts.push(state);
        }
        parts.push(&self.postal_code);
        parts.push(&self.country);
        parts.join(", ")
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.street == other.street
            && self.city == other.city
            && self.state == other.state
            && self.postal_code == other.postal_code
            && self.country == other.country
    }
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }

    pub fn is_valid(&self) -> bool {
        self.latitude >= -90.0
            && self.latitude <= 90.0
            && self.longitude >= -180.0
            && self.longitude <= 180.0
    }
}

impl PartialEq for Coordinates {
    fn eq(&self, other: &Self) -> bool {
        (self.latitude - other.latitude).abs() < f64::EPSILON
            && (self.longitude - other.longitude).abs() < f64::EPSILON
    }
}

/// Search criteria for complex queries across modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCriteria {
    pub query: Option<String>,
    pub filters: HashMap<String, serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
    pub page: u32,
    pub limit: u32,
}

impl SearchCriteria {
    pub fn new() -> Self {
        Self {
            query: None,
            filters: HashMap::new(),
            sort_by: None,
            sort_order: SortOrder::Ascending,
            page: 1,
            limit: 20,
        }
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.filters.insert(key, value);
        self
    }

    pub fn with_sort(mut self, sort_by: String, sort_order: SortOrder) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_order = sort_order;
        self
    }

    pub fn with_pagination(mut self, page: u32, limit: u32) -> Self {
        self.page = page.max(1);
        self.limit = limit.clamp(1, 100);
        self
    }

    pub fn get_offset(&self) -> u32 {
        (self.page - 1) * self.limit
    }
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort order enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Ascending
    }
}

/// Module health status value object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub module_name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metrics: HashMap<String, f64>,
}

impl ModuleHealth {
    pub fn healthy(module_name: String) -> Self {
        Self {
            module_name,
            status: HealthStatus::Healthy,
            last_check: Utc::now(),
            response_time_ms: None,
            error_message: None,
            metrics: HashMap::new(),
        }
    }

    pub fn unhealthy(module_name: String, error: String) -> Self {
        Self {
            module_name,
            status: HealthStatus::Unhealthy,
            last_check: Utc::now(),
            response_time_ms: None,
            error_message: Some(error),
            metrics: HashMap::new(),
        }
    }

    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = Some(response_time_ms);
        self
    }

    pub fn with_metric(mut self, key: String, value: f64) -> Self {
        self.metrics.insert(key, value);
        self
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl PartialEq for HealthStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HealthStatus::Healthy, HealthStatus::Healthy) => true,
            (HealthStatus::Degraded, HealthStatus::Degraded) => true,
            (HealthStatus::Unhealthy, HealthStatus::Unhealthy) => true,
            _ => false,
        }
    }
}