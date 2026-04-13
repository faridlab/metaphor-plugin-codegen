//! Use Cases
//!
//! These are the application's use cases that orchestrate business logic
//! across multiple modules. Each use case represents a specific
//! application-level scenario.

pub mod user_onboarding;
pub mod system_configuration;
pub mod cross_module_search;
pub mod backup_restore;

// Re-export use cases
pub use user_onboarding::*;
pub use system_configuration::*;
pub use cross_module_search::*;
pub use backup_restore::*;