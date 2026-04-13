//! Application Domain Layer
//!
//! This layer contains business logic that is specific to the Metaphor application
//! but crosses multiple bounded contexts (modules). It includes:
//!
//! - Cross-cutting domain entities
//! - Application-wide business rules
//! - Domain services that coordinate between modules
//! - Cross-module events and specifications

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod repositories;
pub mod events;

// Re-export commonly used domain types
pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use repositories::*;
pub use events::*;