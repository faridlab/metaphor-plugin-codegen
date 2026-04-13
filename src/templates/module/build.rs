use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");

    // Find the module name from Cargo.toml
    let module_name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "module".to_string());
    let module_snake_case = module_name.replace("-", "_");

    // Set output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated_dir = out_dir.join("generated");

    // Create generated directory
    fs::create_dir_all(&generated_dir)?;

    // Proto files to compile
    let proto_files = vec![
        "proto/domain/entity/metaphor.proto",
        "proto/domain/value_object/common.proto",
        "proto/domain/repository/metaphor_repository.proto",
        "proto/domain/usecase/commands.proto",
        "proto/domain/usecase/queries.proto",
        "proto/domain/service/metaphor_service.proto",
        "proto/domain/event/metaphor_events.proto",
        "proto/domain/specification/rules.proto",
    ];

    // Convert relative proto paths to absolute paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_files: Vec<String> = proto_files
        .iter()
        .map(|file| manifest_dir.join(file).to_string_lossy().to_string())
        .collect();

    // Configure tonic-build
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&generated_dir)
        .type_attribute(".", "#[allow(clippy::derive_partial_eq_without_eq)]")
        .field_attribute("id.", "#[serde(skip_serializing_if = \"Option::is_none\")]")
        .field_attribute("created_at.", "#[serde(skip_serializing_if = \"Option::is_none\")]")
        .field_attribute("updated_at.", "#[serde(skip_serializing_if = \"Option::is_none\")]")
        .field_attribute("deleted_at.", "#[serde(skip_serializing_if = \"Option::is_none\")]")
        .compile(
            &proto_files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &["proto"],
        )?;

    // Generate mod.rs file for the generated code
    let mod_content = generate_mod_file(&module_snake_case);
    fs::write(generated_dir.join("mod.rs"), mod_content)?;

    // Generate lib.rs file that re-exports generated types
    let lib_content = generate_lib_file(&module_snake_case);
    fs::write(generated_dir.join("lib.rs"), lib_content)?;

    println!("Proto compilation completed successfully for module: {}", module_name);
    Ok(())
}

fn generate_mod_file(module_name: &str) -> String {
    format!(r#"// Generated protobuf types for {module_name} module
// This file is auto-generated. DO NOT EDIT.

pub mod entity;
pub mod value_object;
pub mod repository;
pub mod usecase;
pub mod service;
pub mod event;
pub mod specification;

// Re-export commonly used types
pub use entity::*;
pub use value_object::*;
pub use repository::*;
pub use usecase::*;
pub use service::*;
pub use event::*;
pub use specification::*;
"#)
}

fn generate_lib_file(module_name: &str) -> String {
    format!(r#"// Generated types for {module_name} module
// This file is auto-generated. DO NOT EDIT.

// Include generated modules
pub mod entity;
pub mod value_object;
pub mod repository;
pub mod usecase;
pub mod service;
pub mod event;
pub mod specification;

// Include all proto generated code
include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));
"#)
}