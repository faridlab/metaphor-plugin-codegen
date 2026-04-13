//! {{PascalCaseEntity}} Aggregate Root Implementation
//!
//! This is the Rust implementation of the {{PascalCaseEntity}} aggregate root.
//! It enforces business invariants and maintains consistency.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::events::{{MODULE_NAME}}_events::*;
use crate::domain::value_objects::*;

/// {{PascalCaseEntity}} Aggregate Root
///
/// This aggregate root represents the {{MODULE_NAME}} bounded context.
/// It maintains consistency and enforces business invariants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {{PascalCaseEntity}} {
    /// Unique identifier for the aggregate
    pub id: Uuid,

  COMMON_FIELDS_PLACEHOLDER

    // TODO: Add aggregate-specific fields here
    // Example:
    // pub name: String,
    // pub status: {{PascalCaseEntity}}Status,
    // pub quantity: i32,
    // pub metadata: HashMap<String, String>,

    // Domain events (not persisted)
    #[serde(skip)]
    pub pending_events: Vec<{{PascalCaseEntity}}Event>,
}

impl {{PascalCaseEntity}} {
    /// Create a new {{PascalCaseEntity}} aggregate
    pub fn create(
        // TODO: Add creation parameters here
        // name: String,
        // initial_quantity: i32,
    ) -> Result<Self, {{PascalCaseEntity}}Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // TODO: Validate creation data
        // if name.is_empty() {
        //     return Err({{PascalCaseEntity}}Error::InvalidName("Name cannot be empty".to_string()));
        // }
        // if initial_quantity < 0 {
        //     return Err({{PascalCaseEntity}}Error::InvalidQuantity("Quantity cannot be negative".to_string()));
        // }

        let mut aggregate = Self {
            id,
            {% if with_common_fields %}
            created_at: now,
            updated_at: now,
            deleted_at: None,
            {% endif %}
            // TODO: Initialize fields here
            // name,
            // status: {{PascalCaseEntity}}Status::Active,
            // quantity: initial_quantity,
            // metadata: HashMap::new(),
            pending_events: Vec::new(),
        };

        // TODO: Generate and add domain event
        // aggregate.add_domain_event({{PascalCaseEntity}}Event::Created {
        //     event_id: Uuid::new_v4(),
        //     aggregate_id: id,
        //     occurred_at: now,
        //     name: name.clone(),
        // });

        Ok(aggregate)
    }

    /// Update {{PascalCaseEntity}}
    pub fn update(
        &mut self,
        // TODO: Add update parameters here
        // new_name: Option<String>,
        // new_quantity: Option<i32>,
    ) -> Result<(), {{PascalCaseEntity}}Error> {
        // TODO: Validate update data and business rules
        // if let Some(name) = &new_name {
        //     if name.is_empty() {
        //         return Err({{PascalCaseEntity}}Error::InvalidName("Name cannot be empty".to_string()));
        //     }
        // }

        // TODO: Apply updates and check invariants
        // if let Some(quantity) = new_quantity {
        //     if quantity < 0 {
        //         return Err({{PascalCaseEntity}}Error::InvalidQuantity("Quantity cannot be negative".to_string()));
        //     }
        //     self.quantity = quantity;
        // }

        // if let Some(name) = new_name {
        //     let old_name = self.name.clone();
        //     self.name = name;
        //
        //     // Generate domain event for name change
        //     self.add_domain_event({{PascalCaseEntity}}Event::NameChanged {
        //         event_id: Uuid::new_v4(),
        //         aggregate_id: self.id,
        //         occurred_at: Utc::now(),
        //         old_name,
        //         new_name: self.name.clone(),
        //     });
        // }

        {% if with_common_fields %}
        self.updated_at = Utc::now();
        {% endif %}

        Ok(())
    }

    /// Soft delete {{PascalCaseEntity}}
    pub fn delete(&mut self, reason: Option<String>) -> Result<(), {{PascalCaseEntity}}Error> {
        // TODO: Check business rules for deletion
        // if self.status == {{PascalCaseEntity}}Status::Locked {
        //     return Err({{PascalCaseEntity}}Error::CannotDeleteLockedAggregate);
        // }

        {% if with_common_fields %}
        self.deleted_at = Some(Utc::now());
        {% endif %}

        // Generate domain event
        self.add_domain_event({{PascalCaseEntity}}Event::Deleted {
            event_id: Uuid::new_v4(),
            aggregate_id: self.id,
            occurred_at: Utc::now(),
            reason,
        });

        Ok(())
    }

    /// Restore soft-deleted {{PascalCaseEntity}}
    pub fn restore(&mut self) -> Result<(), {{PascalCaseEntity}}Error> {
        // TODO: Check business rules for restoration
        // if !self.can_be_restored() {
        //     return Err({{PascalCaseEntity}}Error::CannotRestoreAggregate);
        // }

        {% if with_common_fields %}
        self.deleted_at = None;
        self.updated_at = Utc::now();
        {% endif %}

        // Generate domain event
        self.add_domain_event({{PascalCaseEntity}}Event::Restored {
            event_id: Uuid::new_v4(),
            aggregate_id: self.id,
            occurred_at: Utc::now(),
        });

        Ok(())
    }

    /// Add a domain event to the pending events list
    pub fn add_domain_event(&mut self, event: {{PascalCaseEntity}}Event) {
        self.pending_events.push(event);
    }

    /// Get pending domain events and clear them
    pub fn take_domain_events(&mut self) -> Vec<{{PascalCaseEntity}}Event> {
        std::mem::take(&mut self.pending_events)
    }

    /// Check if aggregate is deleted
    pub fn is_deleted(&self) -> bool {
        {% if with_common_fields %}
        self.deleted_at.is_some()
        {% else %}
        false // TODO: Implement deletion logic if not using common fields
        {% endif %}
    }

    // TODO: Add business methods that enforce invariants
    // Example:
    // pub fn change_status(&mut self, new_status: {{PascalCaseEntity}}Status) -> Result<(), {{PascalCaseEntity}}Error> {
    //     if !self.can_change_status_to(&new_status) {
    //         return Err({{PascalCaseEntity}}Error::InvalidStatusTransition);
    //     }
    //
    //     let old_status = self.status;
    //     self.status = new_status;
    //     self.updated_at = Utc::now();
    //
    //     self.add_domain_event({{PascalCaseEntity}}Event::StatusChanged {
    //         event_id: Uuid::new_v4(),
    //         aggregate_id: self.id,
    //         occurred_at: Utc::now(),
    //         old_status,
    //         new_status,
    //     });
    //
    //     Ok(())
    // }
    //
    // pub fn can_change_status_to(&self, new_status: &{{PascalCaseEntity}}Status) -> bool {
    //     // Implement business rules for status transitions
    //     match (&self.status, new_status) {
    //         ({{PascalCaseEntity}}Status::Draft, {{PascalCaseEntity}}Status::Active) => true,
    //         ({{PascalCaseEntity}}Status::Active, {{PascalCaseEntity}}Status::Suspended) => true,
    //         ({{PascalCaseEntity}}Status::Suspended, {{PascalCaseEntity}}Status::Active) => true,
    //         ({{PascalCaseEntity}}Status::Active, {{PascalCaseEntity}}Status::Closed) => true,
    //         ({{PascalCaseEntity}}Status::Suspended, {{PascalCaseEntity}}Status::Closed) => true,
    //         _ => false,
    //     }
    // }
}

// TODO: Implement validation traits
// impl Validate for {{PascalCaseEntity}} {
//     fn validate(&self) -> Result<(), ValidationError> {
//         if self.name.is_empty() {
//             return Err(ValidationError::new("name", "Name cannot be empty"));
//         }
//         if self.quantity < 0 {
//             return Err(ValidationError::new("quantity", "Quantity cannot be negative"));
//         }
//         Ok(())
//     }
// }

/// {{PascalCaseEntity}} aggregate errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum {{PascalCaseEntity}}Error {
    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    #[error("Cannot delete locked aggregate")]
    CannotDeleteLockedAggregate,

    #[error("Cannot restore aggregate")]
    CannotRestoreAggregate,

    #[error("Invalid status transition")]
    InvalidStatusTransition,

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// {{PascalCaseEntity}} status enum (example)
// TODO: Customize this for your domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum {{PascalCaseEntity}}Status {
    Draft,
    Active,
    Suspended,
    Closed,
}