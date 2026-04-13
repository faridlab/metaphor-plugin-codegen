//! Metaphor Codegen Plugin — scaffolding and code generation commands.
//!
//! This plugin provides: make, module, apps, proto, migration, seed.

pub mod commands;
pub mod app_generator;
pub mod templates;
pub mod utils;

pub use anyhow::Result;
