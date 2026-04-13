// Metaphor Domain Events
// Domain events for the Metaphor aggregate

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::value_objects::{MetaphorId, MetaphorStatus, Metadata};

// Base Domain Event Trait
pub trait DomainEvent {
    fn event_id(&self) -> &str;
    fn aggregate_id(&self) -> &MetaphorId;
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn version(&self) -> i64;
}

// MetaphorCreated Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorCreated {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub name: String,
    pub description: String,
    pub status: MetaphorStatus,
    pub tags: Vec<String>,
    pub metadata: Metadata,
    pub created_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

impl MetaphorCreated {
    pub fn new(
        metaphor_id: MetaphorId,
        name: String,
        description: String,
        status: MetaphorStatus,
        tags: Vec<String>,
        metadata: Metadata,
        created_by: String,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            name,
            description,
            status,
            tags,
            metadata,
            created_by,
            occurred_at: Utc::now(),
            version: 1,
        }
    }
}

impl DomainEvent for MetaphorCreated {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorCreated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

// MetaphorUpdated Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorUpdated {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub changes: HashMap<String, String>,
    pub previous_version: i64,
    pub new_version: i64,
    pub updated_by: String,
    pub occurred_at: DateTime<Utc>,
}

impl MetaphorUpdated {
    pub fn new(
        metaphor_id: MetaphorId,
        changes: HashMap<String, String>,
        previous_version: i64,
        new_version: i64,
        updated_by: String,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            changes,
            previous_version,
            new_version,
            updated_by,
            occurred_at: Utc::now(),
        }
    }

    pub fn add_change(&mut self, field: String, old_value: String, new_value: String) {
        self.changes.insert(field, format!("{} -> {}", old_value, new_value));
    }

    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

impl DomainEvent for MetaphorUpdated {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorUpdated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.new_version
    }
}

// MetaphorStatusChanged Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorStatusChanged {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub previous_status: MetaphorStatus,
    pub new_status: MetaphorStatus,
    pub reason: String,
    pub changed_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

impl MetaphorStatusChanged {
    pub fn new(
        metaphor_id: MetaphorId,
        previous_status: MetaphorStatus,
        new_status: MetaphorStatus,
        reason: String,
        changed_by: String,
        version: i64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            previous_status,
            new_status,
            reason,
            changed_by,
            occurred_at: Utc::now(),
            version,
        }
    }

    pub fn is_activation(&self) -> bool {
        self.new_status.is_active() && !self.previous_status.is_active()
    }

    pub fn is_deactivation(&self) -> bool {
        !self.new_status.is_active() && self.previous_status.is_active()
    }

    pub fn is_suspension(&self) -> bool {
        self.new_status.is_suspended() && !self.previous_status.is_suspended()
    }

    pub fn is_archival(&self) -> bool {
        self.new_status.is_archived() && !self.previous_status.is_archived()
    }
}

impl DomainEvent for MetaphorStatusChanged {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorStatusChanged"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

// MetaphorTagsChanged Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorTagsChanged {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub added_tags: Vec<String>,
    pub removed_tags: Vec<String>,
    pub changed_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

impl MetaphorTagsChanged {
    pub fn new(
        metaphor_id: MetaphorId,
        added_tags: Vec<String>,
        removed_tags: Vec<String>,
        changed_by: String,
        version: i64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            added_tags,
            removed_tags,
            changed_by,
            occurred_at: Utc::now(),
            version,
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.added_tags.is_empty() || !self.removed_tags.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added_tags.len() + self.removed_tags.len()
    }
}

impl DomainEvent for MetaphorTagsChanged {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorTagsChanged"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

// MetaphorMetadataChanged Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorMetadataChanged {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub previous_metadata: Metadata,
    pub new_metadata: Metadata,
    pub changed_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

impl MetaphorMetadataChanged {
    pub fn new(
        metaphor_id: MetaphorId,
        previous_metadata: Metadata,
        new_metadata: Metadata,
        changed_by: String,
        version: i64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            previous_metadata,
            new_metadata,
            changed_by,
            occurred_at: Utc::now(),
            version,
        }
    }

    pub fn has_changes(&self) -> bool {
        self.previous_metadata != self.new_metadata
    }

    pub fn get_added_keys(&self) -> Vec<String> {
        self.new_metadata
            .keys()
            .filter(|&k| !self.previous_metadata.contains_key(k))
            .cloned()
            .collect()
    }

    pub fn get_removed_keys(&self) -> Vec<String> {
        self.previous_metadata
            .keys()
            .filter(|&k| !self.new_metadata.contains_key(k))
            .cloned()
            .collect()
    }

    pub fn get_modified_keys(&self) -> Vec<String> {
        self.new_metadata
            .keys()
            .filter(|&k| {
                self.previous_metadata.contains_key(k)
                    && self.previous_metadata.get(k) != self.new_metadata.get(k)
            })
            .cloned()
            .collect()
    }
}

impl DomainEvent for MetaphorMetadataChanged {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorMetadataChanged"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

// MetaphorDeleted Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaphorDeleted {
    pub event_id: String,
    pub metaphor_id: MetaphorId,
    pub hard_delete: bool,
    pub reason: String,
    pub deleted_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

impl MetaphorDeleted {
    pub fn new(
        metaphor_id: MetaphorId,
        hard_delete: bool,
        reason: String,
        deleted_by: String,
        version: i64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            metaphor_id,
            hard_delete,
            reason,
            deleted_by,
            occurred_at: Utc::now(),
            version,
        }
    }

    pub fn is_soft_delete(&self) -> bool {
        !self.hard_delete
    }

    pub fn is_hard_delete(&self) -> bool {
        self.hard_delete
    }
}

impl DomainEvent for MetaphorDeleted {
    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn aggregate_id(&self) -> &MetaphorId {
        &self.metaphor_id
    }

    fn event_type(&self) -> &'static str {
        "MetaphorDeleted"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

// Event Store for managing domain events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventStore {
    pub events: Vec<Box<dyn DomainEvent>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event<E: DomainEvent + 'static>(&mut self, event: E) {
        self.events.push(Box::new(event));
    }

    pub fn get_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.events
    }

    pub fn get_events_by_type(&self, event_type: &str) -> Vec<&Box<dyn DomainEvent>> {
        self.events
            .iter()
            .filter(|e| e.event_type() == event_type)
            .collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn get_latest_version(&self) -> i64 {
        self.events
            .iter()
            .map(|e| e.version())
            .max()
            .unwrap_or(0)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{MetaphorName, MetaphorVersion};

    #[test]
    fn test_metaphor_created_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let metadata = Metadata::from_map([("key".to_string(), "value".to_string())].into()).unwrap();

        let event = MetaphorCreated::new(
            metaphor_id.clone(),
            "Test Metaphor".to_string(),
            "Test Description".to_string(),
            MetaphorStatus::Active,
            vec!["test".to_string()],
            metadata,
            "test_user".to_string(),
        );

        assert_eq!(event.event_type(), "MetaphorCreated");
        assert_eq!(event.aggregate_id(), &metaphor_id);
        assert_eq!(event.name, "Test Metaphor");
        assert!(event.occurred_at <= Utc::now());
    }

    #[test]
    fn test_metaphor_updated_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let mut changes = HashMap::new();
        changes.insert("name".to_string(), "Old -> New".to_string());

        let event = MetaphorUpdated::new(
            metaphor_id.clone(),
            changes,
            1,
            2,
            "test_user".to_string(),
        );

        assert_eq!(event.event_type(), "MetaphorUpdated");
        assert_eq!(event.aggregate_id(), &metaphor_id);
        assert!(event.has_changes());
        assert_eq!(event.new_version, 2);
    }

    #[test]
    fn test_metaphor_status_changed_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let event = MetaphorStatusChanged::new(
            metaphor_id.clone(),
            MetaphorStatus::Inactive,
            MetaphorStatus::Active,
            "User activation".to_string(),
            "admin".to_string(),
            2,
        );

        assert_eq!(event.event_type(), "MetaphorStatusChanged");
        assert_eq!(event.aggregate_id(), &metaphor_id);
        assert!(event.is_activation());
        assert!(!event.is_deactivation());
    }

    #[test]
    fn test_metaphor_tags_changed_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let event = MetaphorTagsChanged::new(
            metaphor_id.clone(),
            vec!["new_tag".to_string()],
            vec!["old_tag".to_string()],
            "user".to_string(),
            3,
        );

        assert_eq!(event.event_type(), "MetaphorTagsChanged");
        assert_eq!(event.aggregate_id(), &metaphor_id);
        assert!(event.has_changes());
        assert_eq!(event.total_changes(), 2);
    }

    #[test]
    fn test_metaphor_metadata_changed_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let old_metadata = Metadata::from_map([("old".to_string(), "value".to_string())].into()).unwrap();
        let new_metadata = Metadata::from_map([("new".to_string(), "value".to_string())].into()).unwrap();

        let event = MetaphorMetadataChanged::new(
            metaphor_id.clone(),
            old_metadata.clone(),
            new_metadata.clone(),
            "user".to_string(),
            4,
        );

        assert_eq!(event.event_type(), "MetaphorMetadataChanged");
        assert_eq!(event.aggregate_id(), &metaphor_id);
        assert!(event.has_changes());
    }

    #[test]
    fn test_metaphor_deleted_event() {
        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let soft_event = MetaphorDeleted::new(
            metaphor_id.clone(),
            false,
            "Soft delete".to_string(),
            "user".to_string(),
            5,
        );

        assert!(soft_event.is_soft_delete());
        assert!(!soft_event.is_hard_delete());

        let hard_event = MetaphorDeleted::new(
            metaphor_id,
            true,
            "Hard delete".to_string(),
            "admin".to_string(),
            6,
        );

        assert!(!hard_event.is_soft_delete());
        assert!(hard_event.is_hard_delete());
    }

    #[test]
    fn test_event_store() {
        let mut store = EventStore::new();
        assert!(store.is_empty());

        let metaphor_id = MetaphorId::new("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let event = MetaphorCreated::new(
            metaphor_id,
            "Test".to_string(),
            "Desc".to_string(),
            MetaphorStatus::Active,
            vec![],
            Metadata::new(),
            "user".to_string(),
        );

        store.add_event(event);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get_latest_version(), 1);

        let events = store.get_events_by_type("MetaphorCreated");
        assert_eq!(events.len(), 1);

        store.clear();
        assert!(store.is_empty());
    }
}