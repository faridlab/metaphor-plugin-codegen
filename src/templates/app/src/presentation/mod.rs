//! Presentation Layer
//!
//! This layer contains all the interface adapters that handle
//! communication with external systems. This includes HTTP controllers,
//! gRPC services, CLI commands, and other presentation concerns.

pub mod http;
pub mod grpc;
pub mod cli;

// Re-export presentation components
pub use http::*;
pub use grpc::*;
pub use cli::*;