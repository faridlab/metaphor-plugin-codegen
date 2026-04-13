//! Domain Services
//!
//! These services contain business logic that doesn't naturally fit within
//! a single entity or value object and often coordinates between multiple
/// bounded contexts (modules).

pub mod permission_service;
pub mod user_session_service;
pub mod audit_service;
pub mod module_health_service;

// Re-export services
pub use permission_service::*;
pub use user_session_service::*;
pub use audit_service::*;
pub use module_health_service::*;