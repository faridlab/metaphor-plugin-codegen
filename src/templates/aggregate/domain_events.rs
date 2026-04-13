//! {{PascalCaseEntity}} Domain Events Implementation
//!
//! Domain events represent something that happened in the domain
//! that domain experts care about.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// {{PascalCaseEntity}} Domain Events
///
/// Union type representing all possible {{PascalCaseEntity}} domain events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum {{PascalCaseEntity}}Event {
    /// {{PascalCaseEntity}} was created
    Created({{PascalCaseEntity}}CreatedEvent),

    /// {{PascalCaseEntity}} was updated
    Updated({{PascalCaseEntity}}UpdatedEvent),

    /// {{PascalCaseEntity}} was deleted
    Deleted({{PascalCaseEntity}}DeletedEvent),

    /// {{PascalCaseEntity}} was restored
    Restored({{PascalCaseEntity}}RestoredEvent),

    // TODO: Add custom domain events here
    // Example:
    // /// {{PascalCaseEntity}} status changed
    // StatusChanged({{PascalCaseEntity}}StatusChangedEvent),
    //
    // /// Business rule was violated
    // BusinessRuleViolation({{PascalCaseEntity}}BusinessRuleViolationEvent),
}

/// {{PascalCaseEntity}} Created Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}}CreatedEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Aggregate that generated this event
    pub aggregate_id: Uuid,

    /// When the event occurred
    pub occurred_at: DateTime<Utc>,

    // TODO: Add event-specific data here
    // Example:
    // /// Who created the aggregate
    // pub created_by: String,
    //
    // /// Name of the aggregate
    // pub name: String,
    //
    // /// Additional metadata
    // pub metadata: std::collections::HashMap<String, String>,
}

impl {{PascalCaseEntity}}CreatedEvent {
    /// Create a new {{PascalCaseEntity}} created event
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            // TODO: Initialize event-specific fields
        }
    }
}

/// {{PascalCaseEntity}} Updated Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}}UpdatedEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Aggregate that generated this event
    pub aggregate_id: Uuid,

    /// When the event occurred
    pub occurred_at: DateTime<Utc>,

    // TODO: Add changed fields data here
    // Example:
    // /// Who updated the aggregate
    // pub updated_by: String,
    //
    // /// Fields that were changed
    // pub changed_fields: Vec<String>,
    //
    // /// Old values
    // pub old_values: std::collections::HashMap<String, String>,
    //
    // /// New values
    // pub new_values: std::collections::HashMap<String, String>,
}

impl {{PascalCaseEntity}}UpdatedEvent {
    /// Create a new {{PascalCaseEntity}} updated event
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            // TODO: Initialize event-specific fields
        }
    }
}

/// {{PascalCaseEntity}} Deleted Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}}DeletedEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Aggregate that generated this event
    pub aggregate_id: Uuid,

    /// When the event occurred
    pub occurred_at: DateTime<Utc>,

    // TODO: Add deletion-specific data here
    // Example:
    // /// Who deleted the aggregate
    // pub deleted_by: String,
    //
    // /// Reason for deletion
    // pub reason: Option<String>,
}

impl {{PascalCaseEntity}}DeletedEvent {
    /// Create a new {{PascalCaseEntity}} deleted event
    pub fn new(aggregate_id: Uuid, reason: Option<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            // TODO: Initialize event-specific fields
            // deleted_by: ...,
            // reason,
        }
    }
}

/// {{PascalCaseEntity}} Restored Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}}RestoredEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Aggregate that generated this event
    pub aggregate_id: Uuid,

    /// When the event occurred
    pub occurred_at: DateTime<Utc>,

    // TODO: Add restoration-specific data here
    // Example:
    // /// Who restored the aggregate
    // pub restored_by: String,
}

impl {{PascalCaseEntity}}RestoredEvent {
    /// Create a new {{PascalCaseEntity}} restored event
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            aggregate_id,
            occurred_at: Utc::now(),
            // TODO: Initialize event-specific fields
        }
    }
}

// TODO: Add custom domain event implementations

// Example: Status Changed Event
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct {{PascalCaseEntity}}StatusChangedEvent {
//     pub event_id: Uuid,
//     pub aggregate_id: Uuid,
//     pub occurred_at: DateTime<Utc>,
//
//     /// Old status
//     pub old_status: String,
//
//     /// New status
//     pub new_status: String,
//
//     /// Who changed the status
//     pub changed_by: String,
// }
//
// impl {{PascalCaseEntity}}StatusChangedEvent {
//     pub fn new(aggregate_id: Uuid, old_status: String, new_status: String, changed_by: String) -> Self {
//         Self {
//             event_id: Uuid::new_v4(),
//             aggregate_id,
//             occurred_at: Utc::now(),
//             old_status,
//             new_status,
//             changed_by,
//         }
//     }
// }

// Example: Business Rule Violation Event
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct {{PascalCaseEntity}}BusinessRuleViolationEvent {
//     pub event_id: Uuid,
//     pub aggregate_id: Uuid,
//     pub occurred_at: DateTime<Utc>,
//
//     /// Name of the rule that was violated
//     pub rule_name: String,
//
//     /// Description of the rule
//     pub rule_description: String,
//
//     /// Details about the violation
//     pub violation_details: String,
//
//     /// Who attempted the operation
//     pub attempted_by: String,
// }
//
// impl {{PascalCaseEntity}}BusinessRuleViolationEvent {
//     pub fn new(
//         aggregate_id: Uuid,
//         rule_name: String,
//         rule_description: String,
//         violation_details: String,
//         attempted_by: String,
//     ) -> Self {
//         Self {
//             event_id: Uuid::new_v4(),
//             aggregate_id,
//             occurred_at: Utc::now(),
//             rule_name,
//             rule_description,
//             violation_details,
//             attempted_by,
//         }
//     }
// }

/// Event metadata trait for all {{PascalCaseEntity}} events
pub trait {{PascalCaseEntity}}EventMetadata {
    /// Get the event ID
    fn event_id(&self) -> Uuid;

    /// Get the aggregate ID
    fn aggregate_id(&self) -> Uuid;

    /// Get when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Get the event type name
    fn event_type(&self) -> &'static str;
}

impl {{PascalCaseEntity}}EventMetadata for {{PascalCaseEntity}}Event {
    fn event_id(&self) -> Uuid {
        match self {
            Self::Created(e) => e.event_id,
            Self::Updated(e) => e.event_id,
            Self::Deleted(e) => e.event_id,
            Self::Restored(e) => e.event_id,
            // TODO: Add custom events
            // Self::StatusChanged(e) => e.event_id,
            // Self::BusinessRuleViolation(e) => e.event_id,
        }
    }

    fn aggregate_id(&self) -> Uuid {
        match self {
            Self::Created(e) => e.aggregate_id,
            Self::Updated(e) => e.aggregate_id,
            Self::Deleted(e) => e.aggregate_id,
            Self::Restored(e) => e.aggregate_id,
            // TODO: Add custom events
            // Self::StatusChanged(e) => e.aggregate_id,
            // Self::BusinessRuleViolation(e) => e.aggregate_id,
        }
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            Self::Created(e) => e.occurred_at,
            Self::Updated(e) => e.occurred_at,
            Self::Deleted(e) => e.occurred_at,
            Self::Restored(e) => e.occurred_at,
            // TODO: Add custom events
            // Self::StatusChanged(e) => e.occurred_at,
            // Self::BusinessRuleViolation(e) => e.occurred_at,
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::Created(_) => "{{PascalCaseEntity}}Created",
            Self::Updated(_) => "{{PascalCaseEntity}}Updated",
            Self::Deleted(_) => "{{PascalCaseEntity}}Deleted",
            Self::Restored(_) => "{{PascalCaseEntity}}Restored",
            // TODO: Add custom events
            // Self::StatusChanged(_) => "{{PascalCaseEntity}}StatusChanged",
            // Self::BusinessRuleViolation(_) => "{{PascalCaseEntity}}BusinessRuleViolation",
        }
    }
}
