//! Protocol buffer commands (Priority 1.2 from BACKFRAME_TODO.md)
//!
//! Implements:
//! - `metaphor proto generate` - Generate Rust-native types from proto files
//! - `metaphor proto lint` - Lint and validate proto files

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::*;
use std::fs;
use std::path::Path;

#[derive(Subcommand)]
pub enum ProtoAction {
    /// Generate Rust-native types from proto files
    Generate {
        module: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Lint and validate proto files
    Lint {
        #[arg(long)]
        fix: bool,
    },
}

/// Protocol buffer command handler
pub async fn handle_command(action: &ProtoAction) -> Result<()> {
    match action {
        ProtoAction::Generate { module, force, dry_run } => {
            generate_rust_native(module, *force, *dry_run).await
        }
        ProtoAction::Lint { fix } => {
            lint_proto_files(*fix).await
        }
    }
}

/// Discover all modules that contain proto files
fn discover_modules_with_protos(modules_root: &Path) -> Result<Vec<ModuleProtoInfo>> {
    let mut modules = Vec::new();

    for entry in fs::read_dir(modules_root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this module has proto/domain directory
            let proto_domain_dir = path.join("proto/domain");

            if proto_domain_dir.exists() {
                let mut proto_files = Vec::new();
                collect_proto_files(&proto_domain_dir, &mut proto_files)?;

                if !proto_files.is_empty() {
                    modules.push(ModuleProtoInfo {
                        name: entry.file_name().to_string_lossy().to_string(),
                        module_path: path.clone(),
                        proto_domain_dir,
                        proto_files,
                        generated_dir: path.join("src/generated"),
                    });
                }
            }
        }
    }

    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(modules)
}

/// Information about a module's proto files
struct ModuleProtoInfo {
    name: String,
    #[allow(dead_code)]
    module_path: PathBuf,
    #[allow(dead_code)]
    proto_domain_dir: PathBuf,
    proto_files: Vec<PathBuf>,
    #[allow(dead_code)]
    generated_dir: PathBuf,
}

/// Recursively collect all proto files in a directory
fn collect_proto_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_proto_files(&path, files)?;
        } else if let Some(ext) = path.extension() {
            if ext == "proto" {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Lint and validate proto files with module support
async fn lint_proto_files(_fix: bool) -> Result<()> {
    println!("🔍 {} proto files...", "Linting".bright_yellow());

    let modules_root = Path::new("libs/modules");

    if !modules_root.exists() {
        println!("❌ No modules directory found at: {}", modules_root.display());
        println!("   Expected structure: libs/modules/{{module}}/proto/domain/");
        return Ok(());
    }

    let modules_with_protos = discover_modules_with_protos(modules_root)?;

    if modules_with_protos.is_empty() {
        println!("⚠️  No modules with .proto files found");
        return Ok(());
    }

    for module in &modules_with_protos {
        println!("  📦 Linting module: {}", module.name.bright_cyan());

        for proto_file in &module.proto_files {
            lint_single_proto_file(proto_file, &module.name)?;
        }
    }

    println!("✅ Proto files linted successfully");
    Ok(())
}

/// Enhanced lint for a single proto file with module context
fn lint_single_proto_file(proto_file: &Path, module_name: &str) -> Result<()> {
    let content = fs::read_to_string(proto_file)?;
    let mut issues_found = Vec::new();
    let mut suggestions = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Check package declaration
        if line.starts_with("package") {
            if !line.contains(&format!("{}.domain", module_name)) {
                suggestions.push(format!("Line {}: Package should be {}.domain", line_num + 1, module_name));
            }
        }

        // Check for required imports
        if line.contains("google.protobuf.Timestamp") && !content.contains("import \"google/protobuf/timestamp.proto\"") {
            suggestions.push("Missing import: google/protobuf/timestamp.proto".to_string());
        }

        if line.contains("buf.validate") && !content.contains("import \"buf/validate/validate.proto\"") {
            suggestions.push("Missing import: buf/validate/validate.proto".to_string());
        }

        // Check for common linting issues
        if !line.starts_with("syntax") &&
           !line.starts_with("package") &&
           !line.starts_with("import") &&
           !line.contains("message") &&
           !line.contains("service") &&
           !line.contains("enum") &&
           !line.starts_with("//") &&
           !line.starts_with("}") &&
           !line.contains(" = ") && // Field numbers
           !line.contains("(") && // Options
           !line.starts_with("  ") && // Field definitions
           !line.starts_with("\t") {
            issues_found.push(format!("Line {}: Suspicious content: {}", line_num + 1, line));
        }
    }

    let file_name = proto_file.file_name().unwrap().to_string_lossy();

    if suggestions.is_empty() && issues_found.is_empty() {
        println!("    ✅ {}", file_name);
    } else {
        println!("    ⚠️  {} - {} potential issues", file_name, issues_found.len() + suggestions.len());

        // Show suggestions first
        for suggestion in suggestions.iter().take(3) {
            println!("    💡 {}", suggestion);
        }

        // Then show issues
        for issue in issues_found.iter().take(2) {
            println!("    ⚠️  {}", issue);
        }

        if issues_found.len() + suggestions.len() > 5 {
            println!("    ... and {} more", (issues_found.len() + suggestions.len()) - 5);
        }
    }

    Ok(())
}

// Add missing imports
use std::path::PathBuf;

// ============================================================
// RUST-NATIVE CODE GENERATION
// ============================================================

/// Generate Rust-native types from proto files (no tonic-build dependency)
async fn generate_rust_native(module: &str, force: bool, dry_run: bool) -> Result<()> {
    println!("🦀 {} Rust-native types for module: {}", "Generating".bright_green(), module.bright_cyan());

    let module_path = Path::new("libs/modules").join(module);
    let proto_domain_dir = module_path.join("proto/domain");
    let generated_file = module_path.join("src/generated/mod.rs");

    if !module_path.exists() {
        return Err(anyhow::anyhow!("Module '{}' not found at: {}", module, module_path.display()));
    }

    if !proto_domain_dir.exists() {
        return Err(anyhow::anyhow!("Proto directory not found at: {}", proto_domain_dir.display()));
    }

    if generated_file.exists() && !force {
        println!("⚠️  Generated file already exists: {}", generated_file.display());
        println!("   Use --force to overwrite");
        return Ok(());
    }

    // Collect all proto files
    let mut proto_files = Vec::new();
    collect_proto_files(&proto_domain_dir, &mut proto_files)?;

    if proto_files.is_empty() {
        println!("⚠️  No .proto files found in {}", proto_domain_dir.display());
        return Ok(());
    }

    println!("📄 Found {} proto files:", proto_files.len());
    for file in &proto_files {
        println!("   - {}", file.file_name().unwrap().to_string_lossy());
    }

    // Parse all proto files
    let mut all_parsed = ParsedModule::new(module.to_string());
    for proto_file in &proto_files {
        let parsed = parse_proto_file(proto_file)?;
        all_parsed.merge(parsed);
    }

    println!("\n📊 Parsed:");
    println!("   - {} messages", all_parsed.messages.len());
    println!("   - {} enums", all_parsed.enums.len());
    println!("   - {} services", all_parsed.services.len());

    // Generate Rust code
    let rust_code = generate_rust_code(&all_parsed)?;

    if dry_run {
        println!("\n📝 {} (dry-run mode):", "Generated code preview".bright_yellow());
        println!("{}", "─".repeat(60));
        // Show first 100 lines
        for (i, line) in rust_code.lines().enumerate() {
            if i >= 100 {
                println!("... ({} more lines)", rust_code.lines().count() - 100);
                break;
            }
            println!("{}", line);
        }
        println!("{}", "─".repeat(60));
        return Ok(());
    }

    // Create generated directory if needed
    let generated_dir = module_path.join("src/generated");
    fs::create_dir_all(&generated_dir)?;

    // Write the generated code
    fs::write(&generated_file, &rust_code)?;

    println!("\n✅ {} generated: {}", "Successfully".bright_green(), generated_file.display());
    println!("   {} lines of Rust code", rust_code.lines().count());

    Ok(())
}

// ============================================================
// PROTO PARSING STRUCTURES
// ============================================================

#[derive(Debug, Default)]
struct ParsedModule {
    name: String,
    messages: Vec<ProtoMessage>,
    enums: Vec<ProtoEnum>,
    services: Vec<ProtoService>,
}

impl ParsedModule {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    fn merge(&mut self, other: ParsedProto) {
        // Deduplicate messages by name - keep first occurrence
        for msg in other.messages {
            if !self.messages.iter().any(|m| m.name == msg.name) {
                self.messages.push(msg);
            }
        }

        // Deduplicate enums by name - keep first occurrence
        for e in other.enums {
            if !self.enums.iter().any(|existing| existing.name == e.name) {
                self.enums.push(e);
            }
        }

        // Deduplicate services by name - keep first occurrence
        for svc in other.services {
            if !self.services.iter().any(|s| s.name == svc.name) {
                self.services.push(svc);
            }
        }
    }
}

#[derive(Debug, Default)]
struct ParsedProto {
    package: String,
    messages: Vec<ProtoMessage>,
    enums: Vec<ProtoEnum>,
    services: Vec<ProtoService>,
}

#[derive(Debug, Clone)]
struct ProtoMessage {
    name: String,
    fields: Vec<ProtoField>,
    nested_messages: Vec<ProtoMessage>,
    nested_enums: Vec<ProtoEnum>,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProtoField {
    name: String,
    proto_type: String,
    field_number: u32,
    is_repeated: bool,
    is_optional: bool,
    is_map: bool,
    map_key_type: Option<String>,
    map_value_type: Option<String>,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProtoEnum {
    name: String,
    values: Vec<ProtoEnumValue>,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProtoEnumValue {
    name: String,
    number: i32,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProtoService {
    name: String,
    methods: Vec<ProtoMethod>,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProtoMethod {
    name: String,
    input_type: String,
    output_type: String,
    comments: Vec<String>,
}

// ============================================================
// PROTO PARSING FUNCTIONS
// ============================================================

fn parse_proto_file(path: &Path) -> Result<ParsedProto> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read proto file: {}", path.display()))?;

    let mut parsed = ParsedProto::default();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut pending_comments: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();

        // Collect comments
        if line.starts_with("//") {
            pending_comments.push(line.trim_start_matches("//").trim().to_string());
            i += 1;
            continue;
        }

        // Skip empty lines (but keep comments)
        if line.is_empty() {
            if !pending_comments.is_empty() && i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if !next_line.starts_with("//") && !next_line.starts_with("message")
                   && !next_line.starts_with("enum") && !next_line.starts_with("service") {
                    pending_comments.clear();
                }
            }
            i += 1;
            continue;
        }

        // Parse package
        if line.starts_with("package") {
            parsed.package = line
                .trim_start_matches("package")
                .trim()
                .trim_end_matches(';')
                .trim()
                .to_string();
            pending_comments.clear();
            i += 1;
            continue;
        }

        // Skip imports, syntax, options
        if line.starts_with("syntax") || line.starts_with("import") || line.starts_with("option") {
            pending_comments.clear();
            i += 1;
            continue;
        }

        // Parse message
        if line.starts_with("message") {
            let (message, end_line) = parse_message_block(&lines, i, std::mem::take(&mut pending_comments))?;
            parsed.messages.push(message);
            i = end_line + 1;
            continue;
        }

        // Parse enum
        if line.starts_with("enum") {
            let (proto_enum, end_line) = parse_enum_block(&lines, i, std::mem::take(&mut pending_comments))?;
            parsed.enums.push(proto_enum);
            i = end_line + 1;
            continue;
        }

        // Parse service
        if line.starts_with("service") {
            let (service, end_line) = parse_service_block(&lines, i, std::mem::take(&mut pending_comments))?;
            parsed.services.push(service);
            i = end_line + 1;
            continue;
        }

        pending_comments.clear();
        i += 1;
    }

    Ok(parsed)
}

fn parse_message_block(lines: &[&str], start: usize, comments: Vec<String>) -> Result<(ProtoMessage, usize)> {
    let first_line = lines[start].trim();

    // Extract message name - handle both "message Foo {" and "message Foo {}" cases
    let after_message = first_line.trim_start_matches("message").trim();
    let name = if let Some(brace_pos) = after_message.find('{') {
        after_message[..brace_pos].trim().to_string()
    } else {
        after_message.to_string()
    };

    let mut message = ProtoMessage {
        name,
        fields: Vec::new(),
        nested_messages: Vec::new(),
        nested_enums: Vec::new(),
        comments,
    };

    // Handle empty message on single line: message Foo {}
    if first_line.contains("{}") {
        return Ok((message, start));
    }

    let mut i = start + 1;
    let mut brace_count = 1;
    let mut pending_comments: Vec<String> = Vec::new();

    // Handle case where opening brace is on next line
    if !first_line.contains('{') {
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "{" {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    while i < lines.len() && brace_count > 0 {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Collect comments
        if line.starts_with("//") {
            pending_comments.push(line.trim_start_matches("//").trim().to_string());
            i += 1;
            continue;
        }

        // Handle closing brace
        if line == "}" {
            brace_count -= 1;
            if brace_count == 0 {
                return Ok((message, i));
            }
            i += 1;
            continue;
        }

        // Handle opening brace
        if line.contains('{') {
            brace_count += line.matches('{').count() as i32;
        }
        if line.contains('}') {
            brace_count -= line.matches('}').count() as i32;
        }

        // Parse nested message
        if line.starts_with("message") {
            let (nested_msg, end_line) = parse_message_block(lines, i, std::mem::take(&mut pending_comments))?;
            message.nested_messages.push(nested_msg);
            i = end_line + 1;
            continue;
        }

        // Parse nested enum
        if line.starts_with("enum") {
            let (nested_enum, end_line) = parse_enum_block(lines, i, std::mem::take(&mut pending_comments))?;
            message.nested_enums.push(nested_enum);
            i = end_line + 1;
            continue;
        }

        // Skip reserved, option (not optional!), oneof for now
        if line.starts_with("reserved") || (line.starts_with("option ") || line.starts_with("option(")) || line.starts_with("oneof") {
            // Skip oneof blocks
            if line.starts_with("oneof") && line.contains('{') {
                let mut oneof_brace = 1;
                i += 1;
                while i < lines.len() && oneof_brace > 0 {
                    let l = lines[i].trim();
                    if l.contains('{') { oneof_brace += 1; }
                    if l.contains('}') { oneof_brace -= 1; }
                    i += 1;
                }
                continue;
            }
            pending_comments.clear();
            i += 1;
            continue;
        }

        // Parse field - handle multi-line field definitions (with validation annotations)
        // Skip lines that look like validation annotation continuations
        if line.starts_with("(") || line.starts_with("]") {
            i += 1;
            continue;
        }

        if line.contains('=') {
            // Collect full field definition (may span multiple lines)
            let mut field_line = line.to_string();

            // If line ends with '[' (has validation annotations on following lines)
            if line.ends_with('[') {
                i += 1;
                while i < lines.len() {
                    let next = lines[i].trim();
                    if next.starts_with("//") {
                        i += 1;
                        continue;
                    }
                    // Append to field definition
                    field_line.push(' ');
                    field_line.push_str(next);

                    // Check if field is complete (ends with ];)
                    if next.ends_with("];") || next == "];" {
                        break;
                    }
                    i += 1;
                }
            }

            if let Some(field) = parse_field(&field_line, std::mem::take(&mut pending_comments)) {
                message.fields.push(field);
            }
        }

        i += 1;
    }

    Ok((message, i.saturating_sub(1)))
}

fn parse_enum_block(lines: &[&str], start: usize, comments: Vec<String>) -> Result<(ProtoEnum, usize)> {
    let first_line = lines[start].trim();

    let name = first_line
        .trim_start_matches("enum")
        .trim()
        .trim_end_matches('{')
        .trim()
        .to_string();

    let mut proto_enum = ProtoEnum {
        name,
        values: Vec::new(),
        comments,
    };

    let mut i = start + 1;
    let mut pending_comments: Vec<String> = Vec::new();

    // Handle case where opening brace is on next line
    if !first_line.contains('{') {
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "{" {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        if line.starts_with("//") {
            pending_comments.push(line.trim_start_matches("//").trim().to_string());
            i += 1;
            continue;
        }

        if line == "}" {
            return Ok((proto_enum, i));
        }

        // Skip option lines
        if line.starts_with("option") {
            i += 1;
            continue;
        }

        // Parse enum value: NAME = NUMBER;
        if let Some(eq_pos) = line.find('=') {
            let name = line[..eq_pos].trim().to_string();
            let rest = &line[eq_pos + 1..];

            // Extract number (before any ; or [)
            let number_str = rest
                .split(|c| c == ';' || c == '[')
                .next()
                .unwrap_or("0")
                .trim();

            let number: i32 = number_str.parse().unwrap_or(0);

            proto_enum.values.push(ProtoEnumValue {
                name,
                number,
                comments: std::mem::take(&mut pending_comments),
            });
        }

        i += 1;
    }

    Ok((proto_enum, i.saturating_sub(1)))
}

fn parse_service_block(lines: &[&str], start: usize, comments: Vec<String>) -> Result<(ProtoService, usize)> {
    let first_line = lines[start].trim();

    let name = first_line
        .trim_start_matches("service")
        .trim()
        .trim_end_matches('{')
        .trim()
        .to_string();

    let mut service = ProtoService {
        name,
        methods: Vec::new(),
        comments,
    };

    let mut i = start + 1;
    let mut pending_comments: Vec<String> = Vec::new();

    // Handle case where opening brace is on next line
    if !first_line.contains('{') {
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "{" {
                i += 1;
                break;
            }
            i += 1;
        }
    }

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        if line.starts_with("//") {
            pending_comments.push(line.trim_start_matches("//").trim().to_string());
            i += 1;
            continue;
        }

        if line == "}" {
            return Ok((service, i));
        }

        // Parse rpc method
        if line.starts_with("rpc") {
            if let Some(method) = parse_rpc_method(line, std::mem::take(&mut pending_comments)) {
                service.methods.push(method);
            }
        }

        i += 1;
    }

    Ok((service, i.saturating_sub(1)))
}

fn parse_field(line: &str, comments: Vec<String>) -> Option<ProtoField> {
    let line = line.trim();

    // Skip if no field number assignment
    if !line.contains('=') {
        return None;
    }

    // Check for map type: map<key_type, value_type>
    if line.starts_with("map<") {
        return parse_map_field(line, comments);
    }

    let is_repeated = line.starts_with("repeated ");
    let is_optional = line.starts_with("optional ");

    let line = line
        .trim_start_matches("repeated ")
        .trim_start_matches("optional ")
        .trim();

    // Split into type+name and rest
    let eq_pos = line.find('=')?;
    let before_eq = line[..eq_pos].trim();
    let after_eq = &line[eq_pos + 1..];

    // Parse field number
    let field_number_str = after_eq
        .split(|c| c == ';' || c == '[')
        .next()?
        .trim();
    let field_number: u32 = field_number_str.parse().ok()?;

    // Split type and name
    let parts: Vec<&str> = before_eq.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let proto_type = parts[..parts.len() - 1].join(" ");
    let name = parts[parts.len() - 1].to_string();

    Some(ProtoField {
        name,
        proto_type,
        field_number,
        is_repeated,
        is_optional,
        is_map: false,
        map_key_type: None,
        map_value_type: None,
        comments,
    })
}

fn parse_map_field(line: &str, comments: Vec<String>) -> Option<ProtoField> {
    // map<key_type, value_type> name = number;
    let start = line.find('<')? + 1;
    let end = line.find('>')?;
    let map_content = &line[start..end];

    let parts: Vec<&str> = map_content.split(',').collect();
    if parts.len() != 2 {
        return None;
    }

    let key_type = parts[0].trim().to_string();
    let value_type = parts[1].trim().to_string();

    let after_map = &line[end + 1..];
    let eq_pos = after_map.find('=')?;
    let name = after_map[..eq_pos].trim().to_string();

    let field_number_str = after_map[eq_pos + 1..]
        .split(|c| c == ';' || c == '[')
        .next()?
        .trim();
    let field_number: u32 = field_number_str.parse().ok()?;

    Some(ProtoField {
        name,
        proto_type: format!("map<{}, {}>", key_type, value_type),
        field_number,
        is_repeated: false,
        is_optional: false,
        is_map: true,
        map_key_type: Some(key_type),
        map_value_type: Some(value_type),
        comments,
    })
}

fn parse_rpc_method(line: &str, comments: Vec<String>) -> Option<ProtoMethod> {
    // rpc MethodName(InputType) returns (OutputType);
    let line = line.trim_start_matches("rpc").trim();

    let paren_start = line.find('(')?;
    let name = line[..paren_start].trim().to_string();

    let input_start = paren_start + 1;
    let input_end = line.find(')')?;
    let input_type = line[input_start..input_end].trim().to_string();

    let returns_pos = line.find("returns")?;
    let output_start = line[returns_pos..].find('(')? + returns_pos + 1;
    let output_end = line[output_start..].find(')')? + output_start;
    let output_type = line[output_start..output_end].trim().to_string();

    Some(ProtoMethod {
        name,
        input_type,
        output_type,
        comments,
    })
}

// ============================================================
// RUST CODE GENERATION
// ============================================================

fn generate_rust_code(parsed: &ParsedModule) -> Result<String> {
    let mut code = String::new();

    // Header
    code.push_str(&format!(r#"//! Generated Rust types for {} module
//!
//! This file is auto-generated by `metaphor proto generate`.
//! DO NOT EDIT MANUALLY - changes will be overwritten.
//!
//! Generated from proto files in: libs/modules/{}/proto/domain/

#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::large_enum_variant)]
#![allow(dead_code)]

use serde::{{Deserialize, Serialize}};
use chrono::{{DateTime, Utc}};

"#, parsed.name, parsed.name));

    // Generate enums first (they may be referenced by messages)
    code.push_str("// ============================================================\n");
    code.push_str("// ENUMS\n");
    code.push_str("// ============================================================\n\n");

    for proto_enum in &parsed.enums {
        code.push_str(&generate_enum(proto_enum));
        code.push_str("\n");
    }

    // Generate messages
    code.push_str("// ============================================================\n");
    code.push_str("// MESSAGES (Structs)\n");
    code.push_str("// ============================================================\n\n");

    for message in &parsed.messages {
        code.push_str(&generate_message(message, &parsed.messages));
        code.push_str("\n");
    }

    // Generate module structure for DDD (entity, value_object, prelude)
    code.push_str("// ============================================================\n");
    code.push_str("// MODULE RE-EXPORTS\n");
    code.push_str("// ============================================================\n\n");

    // Entity module - re-exports all types
    code.push_str("pub mod entity {\n");
    code.push_str("    //! Domain entities - all generated types\n");
    code.push_str("    pub use super::*;\n");
    code.push_str("}\n\n");

    // Value object module - re-exports all types (includes value objects like Email, Password, Phone)
    code.push_str("pub mod value_object {\n");
    code.push_str("    //! Value objects - immutable domain objects\n");
    code.push_str("    pub use super::*;\n");
    code.push_str("}\n\n");

    // Prelude module - convenient re-exports of common types
    code.push_str("pub mod prelude {\n");
    code.push_str("    //! Prelude - commonly used types for easy imports\n");
    code.push_str("    pub use super::*;\n");
    code.push_str("}\n\n");

    // Generate type aliases for common naming conventions
    code.push_str("// ============================================================\n");
    code.push_str("// TYPE ALIASES\n");
    code.push_str("// ============================================================\n\n");

    // Check for common patterns and create aliases
    let alias_pairs = [
        ("PhoneNumber", "Phone"),
        ("EmailAddress", "Email"),
    ];

    for (from, to) in alias_pairs {
        if parsed.messages.iter().any(|m| m.name == from) && !parsed.messages.iter().any(|m| m.name == to) {
            code.push_str(&format!("/// Type alias: {} -> {}\n", to, from));
            code.push_str(&format!("pub type {} = {};\n\n", to, from));
        }
    }

    // Generate services if any
    if !parsed.services.is_empty() {
        code.push_str("// ============================================================\n");
        code.push_str("// SERVICES (Traits)\n");
        code.push_str("// ============================================================\n\n");

        for service in &parsed.services {
            code.push_str(&generate_service_trait(service));
            code.push_str("\n");
        }
    }

    Ok(code)
}

fn generate_enum(proto_enum: &ProtoEnum) -> String {
    let mut code = String::new();

    // Doc comments
    if !proto_enum.comments.is_empty() {
        for comment in &proto_enum.comments {
            code.push_str(&format!("/// {}\n", comment));
        }
    }

    // Derive macros
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    code.push_str("#[serde(rename_all = \"SCREAMING_SNAKE_CASE\")]\n");
    code.push_str("#[repr(i32)]\n");
    code.push_str(&format!("pub enum {} {{\n", proto_enum.name));

    for value in &proto_enum.values {
        if !value.comments.is_empty() {
            for comment in &value.comments {
                code.push_str(&format!("    /// {}\n", comment));
            }
        }
        code.push_str(&format!("    {} = {},\n", to_pascal_case(&value.name), value.number));
    }

    code.push_str("}\n\n");

    // Default implementation
    if let Some(first_value) = proto_enum.values.first() {
        code.push_str(&format!("impl Default for {} {{\n", proto_enum.name));
        code.push_str(&format!("    fn default() -> Self {{\n"));
        code.push_str(&format!("        Self::{}\n", to_pascal_case(&first_value.name)));
        code.push_str("    }\n");
        code.push_str("}\n\n");
    }

    // From<i32> implementation
    code.push_str(&format!("impl From<i32> for {} {{\n", proto_enum.name));
    code.push_str("    fn from(value: i32) -> Self {\n");
    code.push_str("        match value {\n");
    for value in &proto_enum.values {
        code.push_str(&format!("            {} => Self::{},\n", value.number, to_pascal_case(&value.name)));
    }
    if let Some(first_value) = proto_enum.values.first() {
        code.push_str(&format!("            _ => Self::{},\n", to_pascal_case(&first_value.name)));
    }
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}

fn generate_message(message: &ProtoMessage, all_messages: &[ProtoMessage]) -> String {
    let mut code = String::new();

    // Doc comments
    if !message.comments.is_empty() {
        for comment in &message.comments {
            code.push_str(&format!("/// {}\n", comment));
        }
    }

    // Derive macros
    code.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    code.push_str(&format!("pub struct {} {{\n", message.name));

    for field in &message.fields {
        if !field.comments.is_empty() {
            for comment in &field.comments {
                code.push_str(&format!("    /// {}\n", comment));
            }
        }

        let rust_type = proto_type_to_rust(&field.proto_type, field.is_optional, field.is_repeated, field.is_map, &field.map_key_type, &field.map_value_type);
        let field_name = to_snake_case(&field.name);

        // Add serde rename if needed
        if field_name != field.name {
            code.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
        }

        // Handle optional serialization
        if field.is_optional {
            code.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
        }

        code.push_str(&format!("    pub {}: {},\n", field_name, rust_type));
    }

    code.push_str("}\n\n");

    // Default implementation
    code.push_str(&format!("impl Default for {} {{\n", message.name));
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        Self {\n");
    for field in &message.fields {
        let field_name = to_snake_case(&field.name);
        let default_value = get_default_value(&field.proto_type, field.is_optional, field.is_repeated, field.is_map);
        code.push_str(&format!("            {}: {},\n", field_name, default_value));
    }
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    // Generate nested types
    for nested_enum in &message.nested_enums {
        code.push_str("\n");
        code.push_str(&generate_enum(nested_enum));
    }

    for nested_msg in &message.nested_messages {
        code.push_str("\n");
        code.push_str(&generate_message(nested_msg, all_messages));
    }

    code
}

fn generate_service_trait(service: &ProtoService) -> String {
    let mut code = String::new();

    // Doc comments
    if !service.comments.is_empty() {
        for comment in &service.comments {
            code.push_str(&format!("/// {}\n", comment));
        }
    }

    code.push_str("#[async_trait::async_trait]\n");
    code.push_str(&format!("pub trait {} {{\n", service.name));

    for method in &service.methods {
        if !method.comments.is_empty() {
            for comment in &method.comments {
                code.push_str(&format!("    /// {}\n", comment));
            }
        }
        let method_name = to_snake_case(&method.name);
        code.push_str(&format!(
            "    async fn {}(&self, request: {}) -> Result<{}, Box<dyn std::error::Error + Send + Sync>>;\n",
            method_name,
            method.input_type,
            method.output_type
        ));
    }

    code.push_str("}\n");

    code
}

fn proto_type_to_rust(
    proto_type: &str,
    is_optional: bool,
    is_repeated: bool,
    is_map: bool,
    map_key_type: &Option<String>,
    map_value_type: &Option<String>,
) -> String {
    if is_map {
        let key = map_key_type.as_ref().map(|k| proto_scalar_to_rust(k)).unwrap_or_else(|| "String".to_string());
        let value = map_value_type.as_ref().map(|v| proto_scalar_to_rust(v)).unwrap_or_else(|| "String".to_string());
        return format!("std::collections::HashMap<{}, {}>", key, value);
    }

    let base_type = proto_scalar_to_rust(proto_type);

    if is_repeated {
        format!("Vec<{}>", base_type)
    } else if is_optional {
        format!("Option<{}>", base_type)
    } else {
        base_type
    }
}

fn proto_scalar_to_rust(proto_type: &str) -> String {
    match proto_type {
        "string" => "String".to_string(),
        "bytes" => "Vec<u8>".to_string(),
        "bool" => "bool".to_string(),
        "int32" | "sint32" | "sfixed32" => "i32".to_string(),
        "int64" | "sint64" | "sfixed64" => "i64".to_string(),
        "uint32" | "fixed32" => "u32".to_string(),
        "uint64" | "fixed64" => "u64".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "google.protobuf.Timestamp" => "DateTime<Utc>".to_string(),
        "google.protobuf.Duration" => "std::time::Duration".to_string(),
        "google.protobuf.Any" => "serde_json::Value".to_string(),
        "google.protobuf.Value" => "serde_json::Value".to_string(),
        "google.protobuf.Struct" => "std::collections::HashMap<String, serde_json::Value>".to_string(),
        "google.protobuf.Empty" => "()".to_string(),
        other => {
            // Handle qualified names (e.g., sapiens.domain.entity.UserStatus)
            if other.contains('.') {
                other.split('.').last().unwrap_or(other).to_string()
            } else {
                other.to_string()
            }
        }
    }
}

fn get_default_value(proto_type: &str, is_optional: bool, is_repeated: bool, is_map: bool) -> String {
    if is_optional {
        return "None".to_string();
    }
    if is_repeated {
        return "Vec::new()".to_string();
    }
    if is_map {
        return "std::collections::HashMap::new()".to_string();
    }

    match proto_type {
        "string" => "String::new()".to_string(),
        "bytes" => "Vec::new()".to_string(),
        "bool" => "false".to_string(),
        "int32" | "sint32" | "sfixed32" => "0".to_string(),
        "int64" | "sint64" | "sfixed64" => "0".to_string(),
        "uint32" | "fixed32" => "0".to_string(),
        "uint64" | "fixed64" => "0".to_string(),
        "float" => "0.0".to_string(),
        "double" => "0.0".to_string(),
        "google.protobuf.Timestamp" => "Utc::now()".to_string(),
        _ => "Default::default()".to_string(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    escape_rust_keyword(&result)
}

/// Escape Rust keywords by prefixing with r#
fn escape_rust_keyword(name: &str) -> String {
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn",
        "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
        "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type",
        "unsafe", "use", "where", "while", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "try", "typeof", "unsized",
        "virtual", "yield",
    ];

    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else {
        name.to_string()
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap());
        }
    }

    result
}