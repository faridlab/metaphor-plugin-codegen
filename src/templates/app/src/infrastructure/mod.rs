//! Infrastructure Layer
//!
//! This layer contains all the technical implementations that support
//! the application layer. This includes database connections,
//! external service clients, messaging systems, and other
//! infrastructure concerns.

pub mod database;
pub mod external;
pub mod messaging;
pub mod health;

// Re-export commonly used infrastructure types
pub use database::*;
pub use external::*;
pub use messaging::*;
pub use health::*;