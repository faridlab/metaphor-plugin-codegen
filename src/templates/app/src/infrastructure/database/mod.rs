//! Database Infrastructure
//!
//! Contains database connection management, migrations, and repository
//! implementations for the application-level domain entities.

pub mod connection;
pub mod crud;
pub mod memory_repository;
pub mod migrations;
pub mod repositories;

// Re-export database components
pub use connection::*;
pub use crud::*;
pub use memory_repository::*;