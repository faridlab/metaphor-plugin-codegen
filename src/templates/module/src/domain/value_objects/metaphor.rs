// Metaphor Value Objects
// Shared value objects for Metaphor domain

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// MetaphorId Value Object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetaphorId(String);

impl MetaphorId {
    pub fn new(id: &str) -> Result<Self, MetaphorIdError> {
        if id.is_empty() {
            return Err(MetaphorIdError::Empty);
        }

        // Validate UUID format (basic validation)
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() != 5 {
            return Err(MetaphorIdError::InvalidFormat);
        }

        Ok(MetaphorId(id.to_string()))
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Default for MetaphorId {
    fn default() -> Self {
        Self::generate()
    }
}

impl fmt::Display for MetaphorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MetaphorId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MetaphorId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetaphorIdError {
    #[error("Metaphor ID cannot be empty")]
    Empty,
    #[error("Invalid Metaphor ID format")]
    InvalidFormat,
}

// MetaphorName Value Object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetaphorName(String);

impl MetaphorName {
    pub fn new(name: &str) -> Result<Self, MetaphorNameError> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(MetaphorNameError::Empty);
        }

        if trimmed.len() > 100 {
            return Err(MetaphorNameError::TooLong);
        }

        // Validate allowed characters: alphanumeric, spaces, hyphens, underscores
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_') {
            return Err(MetaphorNameError::InvalidCharacters);
        }

        Ok(MetaphorName(trimmed.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for MetaphorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MetaphorName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MetaphorName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetaphorNameError {
    #[error("Metaphor name cannot be empty")]
    Empty,
    #[error("Metaphor name cannot exceed 100 characters")]
    TooLong,
    #[error("Metaphor name contains invalid characters")]
    InvalidCharacters,
}

// MetaphorStatus Value Object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetaphorStatus {
    Active,
    Inactive,
    Suspended,
    Archived,
}

impl MetaphorStatus {
    pub fn value(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Inactive => "INACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Archived => "ARCHIVED",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self, Self::Inactive)
    }

    pub fn is_suspended(&self) -> bool {
        matches!(self, Self::Suspended)
    }

    pub fn is_archived(&self) -> bool {
        matches!(self, Self::Archived)
    }

    pub fn can_transition_to(&self, target: &MetaphorStatus) -> bool {
        use MetaphorStatus::*;

        match (self, target) {
            // From any state to same state
            (s, t) if s == t => true,

            // From Active
            (Active, Inactive) => true,
            (Active, Suspended) => true,
            (Active, Archived) => true,

            // From Inactive
            (Inactive, Active) => true,
            (Inactive, Suspended) => true,
            (Inactive, Archived) => true,

            // From Suspended
            (Suspended, Active) => true,
            (Suspended, Inactive) => true,
            (Suspended, Archived) => true,

            // From Archived (can only transition back to Inactive)
            (Archived, Inactive) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    pub fn all_statuses() -> Vec<&'static str> {
        vec!["ACTIVE", "INACTIVE", "SUSPENDED", "ARCHIVED"]
    }
}

impl Default for MetaphorStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl fmt::Display for MetaphorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<&str> for MetaphorStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Self::Active,
            "INACTIVE" => Self::Inactive,
            "SUSPENDED" => Self::Suspended,
            "ARCHIVED" => Self::Archived,
            _ => Self::Active, // Default fallback
        }
    }
}

impl From<String> for MetaphorStatus {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

// MetaphorTimestamp Value Object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaphorTimestamp(DateTime<Utc>);

impl MetaphorTimestamp {
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self(timestamp)
    }

    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_timestamp(timestamp: i64) -> Option<Self> {
        DateTime::from_timestamp(timestamp, 0).map(Self)
    }

    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn iso8601(&self) -> String {
        self.0.to_rfc3339()
    }

    pub fn is_future(&self) -> bool {
        self.0 > Utc::now()
    }

    pub fn is_past(&self) -> bool {
        self.0 < Utc::now()
    }

    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + chrono::Duration::days(days))
    }

    pub fn add_hours(&self, hours: i64) -> Self {
        Self(self.0 + chrono::Duration::hours(hours))
    }
}

impl fmt::Display for MetaphorTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso8601())
    }
}

impl Default for MetaphorTimestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl From<DateTime<Utc>> for MetaphorTimestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<i64> for MetaphorTimestamp {
    fn from(timestamp: i64) -> Self {
        Self(DateTime::from_timestamp(timestamp, 0).unwrap_or_default())
    }
}

// MetaphorVersion Value Object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaphorVersion(i64);

impl MetaphorVersion {
    pub fn new(version: i64) -> Result<Self, MetaphorVersionError> {
        if version < 0 {
            return Err(MetaphorVersionError::Negative);
        }

        Ok(MetaphorVersion(version))
    }

    pub fn initial() -> Self {
        Self(1)
    }

    pub fn value(&self) -> i64 {
        self.0
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn is_first(&self) -> bool {
        self.0 == 1
    }

    pub fn_greater_than(&self, other: &MetaphorVersion) -> bool {
        self.0 > other.0
    }
}

impl fmt::Display for MetaphorTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for MetaphorVersion {
    fn default() -> Self {
        Self::initial()
    }
}

impl From<i64> for MetaphorVersion {
    fn from(version: i64) -> Self {
        Self(version.max(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetaphorVersionError {
    #[error("Metaphor version cannot be negative")]
    Negative,
}

// Metadata Value Object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    data: std::collections::HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn from_map(data: std::collections::HashMap<String, String>) -> Result<Self, MetadataError> {
        if data.is_empty() {
            return Err(MetadataError::Empty);
        }

        // Validate key-value pairs
        for (key, value) in &data {
            if key.is_empty() {
                return Err(MetadataError::EmptyKey);
            }
            if key.len() > 50 {
                return Err(MetadataError::KeyTooLong);
            }
            if value.len() > 500 {
                return Err(MetadataError::ValueTooLong);
            }
        }

        Ok(Self { data })
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: std::collections::HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, key: String, value: String) -> Result<(), MetadataError> {
        if key.is_empty() {
            return Err(MetadataError::EmptyKey);
        }
        if key.len() > 50 {
            return Err(MetadataError::KeyTooLong);
        }
        if value.len() > 500 {
            return Err(MetadataError::ValueTooLong);
        }

        self.data.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &String> {
        self.data.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn to_map(&self) -> std::collections::HashMap<String, String> {
        self.data.clone()
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("Metadata cannot be empty")]
    Empty,
    #[error("Metadata key cannot be empty")]
    EmptyKey,
    #[error("Metadata key cannot exceed 50 characters")]
    KeyTooLong,
    #[error("Metadata value cannot exceed 500 characters")]
    ValueTooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metaphor_id() {
        // Valid UUID
        let id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();
        assert_eq!(id.value(), "123e4567-e89b-12d3-a456-426614174000");

        // Invalid UUID format
        assert!(matches!(
            MetaphorId::new("invalid-uuid"),
            Err(MetaphorIdError::InvalidFormat)
        ));

        // Empty ID
        assert!(matches!(
            MetaphorId::new(""),
            Err(MetaphorIdError::Empty)
        ));
    }

    #[test]
    fn test_metaphor_name() {
        // Valid names
        let name = MetaphorName::new("Test Metaphor").unwrap();
        assert_eq!(name.value(), "Test Metaphor");
        assert_eq!(name.length(), 13);

        let name = MetaphorName::new("metaphor-test_123").unwrap();
        assert_eq!(name.value(), "metaphor-test_123");

        // Empty name
        assert!(matches!(
            MetaphorName::new(""),
            Err(MetaphorNameError::Empty)
        ));

        // Too long name
        let long_name = "a".repeat(101);
        assert!(matches!(
            MetaphorName::new(&long_name),
            Err(MetaphorNameError::TooLong)
        ));

        // Invalid characters
        assert!(matches!(
            MetaphorName::new("test@metaphor"),
            Err(MetaphorNameError::InvalidCharacters)
        ));
    }

    #[test]
    fn test_metaphor_status() {
        let status = MetaphorStatus::Active;
        assert!(status.is_active());
        assert!(!status.is_inactive());

        // Test transitions
        assert!(status.can_transition_to(&MetaphorStatus::Inactive));
        assert!(status.can_transition_to(&MetaphorStatus::Suspended));
        assert!(status.can_transition_to(&MetaphorStatus::Archived));
        assert!(!MetaphorStatus::Archived.can_transition_to(&MetaphorStatus::Active));
    }

    #[test]
    fn test_metaphor_timestamp() {
        let now = MetaphorTimestamp::now();
        assert!(!now.is_future());
        assert!(!now.is_past());

        let future = now.add_days(1);
        assert!(future.is_future());

        let past = now.add_hours(-1);
        assert!(past.is_past());
    }

    #[test]
    fn test_metaphor_version() {
        let version = MetaphorVersion::initial();
        assert!(version.is_first());
        assert_eq!(version.value(), 1);

        let next = version.next();
        assert_eq!(next.value(), 2);
        assert!(next._greater_than(&version));
    }

    #[test]
    fn test_metadata() {
        let mut metadata = Metadata::new();
        assert!(metadata.is_empty());

        metadata.insert("key".to_string(), "value".to_string()).unwrap();
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata.get("key"), Some(&"value".to_string()));

        // Test invalid key
        assert!(matches!(
            metadata.insert("".to_string(), "value".to_string()),
            Err(MetadataError::EmptyKey)
        ));

        // Test invalid value
        let long_value = "a".repeat(501);
        assert!(matches!(
            metadata.insert("key".to_string(), long_value),
            Err(MetadataError::ValueTooLong)
        ));
    }
}