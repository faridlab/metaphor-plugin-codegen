//! HTTP Presentation Layer
//!
//! Contains HTTP controllers, middleware, and route handlers that
//! translate HTTP requests into application layer calls and format
//! responses for HTTP clients.

pub mod controllers;
pub mod middleware;
pub mod routes;

// Re-export HTTP components
pub use controllers::*;
pub use middleware::*;
pub use routes::*;