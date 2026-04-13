//! Laravel-inspired "make" commands for quick scaffolding
//!
//! This module provides quick scaffolding commands similar to Laravel's artisan make:*
//! commands. Each command generates boilerplate code following framework conventions.
//!
//! # Commands
//!
//! - `metaphor make:module` - Create a new module (delegates to module create)
//! - `metaphor make:entity` - Create a new entity (delegates to entity create)
//! - `metaphor make:command` - Create a CQRS command
//! - `metaphor make:query` - Create a CQRS query
//! - `metaphor make:repository` - Create a repository
//! - `metaphor make:handler` - Create an HTTP handler
//! - `metaphor make:service` - Create a domain service
//! - `metaphor make:event` - Create a domain event
//! - `metaphor make:test` - Create a test file (delegates to test generate)
//! - `metaphor make:migration` - Create a database migration
//! - `metaphor make:value-object` - Create a value object
//! - `metaphor make:spec` - Create a specification

use anyhow::Result;
use clap::Subcommand;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Make command actions (Laravel-style scaffolding)
#[derive(Subcommand, Clone, Debug)]
pub enum MakeAction {
    /// Create a new module with bounded context structure
    #[command(name = "module")]
    Module {
        /// Module name (lowercase, e.g., "payments", "analytics")
        name: String,

        /// Module description
        #[arg(long)]
        description: Option<String>,
    },

    /// Create a new entity with proto definition
    #[command(name = "entity")]
    Entity {
        /// Entity name (PascalCase, e.g., "User", "Payment")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Include soft delete support
        #[arg(long)]
        soft_delete: bool,

        /// Include versioning for optimistic locking
        #[arg(long)]
        versioned: bool,
    },

    /// Create a CQRS command
    #[command(name = "command")]
    Command {
        /// Command name (e.g., "CreateUser", "UpdatePayment")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Entity this command operates on
        #[arg(short, long)]
        entity: String,
    },

    /// Create a CQRS query
    #[command(name = "query")]
    Query {
        /// Query name (e.g., "GetUser", "ListPayments")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Entity this query returns
        #[arg(short, long)]
        entity: String,
    },

    /// Create a repository interface and implementation
    #[command(name = "repository")]
    Repository {
        /// Entity name
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Database type (postgres, mongodb)
        #[arg(long, default_value = "postgres")]
        database: String,
    },

    /// Create an HTTP handler
    #[command(name = "handler")]
    Handler {
        /// Handler name (e.g., "user", "payment")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Generate all CRUD handlers
        #[arg(long)]
        crud: bool,
    },

    /// Create a domain service
    #[command(name = "service")]
    Service {
        /// Service name (e.g., "PaymentProcessor", "EmailSender")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,
    },

    /// Create a domain event
    #[command(name = "event")]
    Event {
        /// Event name (e.g., "UserCreated", "PaymentProcessed")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Entity this event relates to
        #[arg(short, long)]
        entity: Option<String>,
    },

    /// Create a test file
    #[command(name = "test")]
    Test {
        /// Test name or entity name
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Test type (unit, integration, e2e)
        #[arg(long, default_value = "unit")]
        r#type: String,
    },

    /// Create a database migration
    #[command(name = "migration")]
    Migration {
        /// Migration name (e.g., "create_users_table", "add_email_to_users")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,

        /// Create table migration
        #[arg(long)]
        create: Option<String>,

        /// Alter table migration
        #[arg(long)]
        table: Option<String>,
    },

    /// Create a value object
    #[command(name = "value-object")]
    ValueObject {
        /// Value object name (e.g., "Email", "Money", "Address")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,
    },

    /// Create a specification (business rule)
    #[command(name = "spec")]
    Specification {
        /// Specification name (e.g., "UserIsActive", "PaymentIsValid")
        name: String,

        /// Target module
        #[arg(short, long)]
        module: String,
    },
}

/// Handle make commands
pub async fn handle_command(action: &MakeAction) -> Result<()> {
    match action {
        MakeAction::Module { name, description } => make_module(name, description.as_deref()).await,

        MakeAction::Entity {
            name,
            module,
            soft_delete,
            versioned,
        } => make_entity(name, module, *soft_delete, *versioned).await,

        MakeAction::Command {
            name,
            module,
            entity,
        } => make_command(name, module, entity).await,

        MakeAction::Query {
            name,
            module,
            entity,
        } => make_query(name, module, entity).await,

        MakeAction::Repository {
            name,
            module,
            database,
        } => make_repository(name, module, database).await,

        MakeAction::Handler { name, module, crud } => make_handler(name, module, *crud).await,

        MakeAction::Service { name, module } => make_service(name, module).await,

        MakeAction::Event {
            name,
            module,
            entity,
        } => make_event(name, module, entity.as_deref()).await,

        MakeAction::Test {
            name,
            module,
            r#type,
        } => make_test(name, module, r#type).await,

        MakeAction::Migration {
            name,
            module,
            create,
            table,
        } => make_migration(name, module, create.as_deref(), table.as_deref()).await,

        MakeAction::ValueObject { name, module } => make_value_object(name, module).await,

        MakeAction::Specification { name, module } => make_specification(name, module).await,
    }
}

// ============================================================================
// Template Processing Utilities
// ============================================================================

/// Get the make templates directory path
fn get_make_template_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("crates")
        .join("metaphor-cli")
        .join("src")
        .join("templates")
        .join("make")
}

/// Load a template file and return its content
fn load_template(template_type: &str, template_name: &str) -> Result<String> {
    let template_path = get_make_template_dir()
        .join(template_type)
        .join(template_name);

    if !template_path.exists() {
        anyhow::bail!(
            "Template not found: {:?}. Please ensure templates are installed.",
            template_path
        );
    }

    Ok(fs::read_to_string(&template_path)?)
}

/// Replace placeholders in template content with actual values
fn process_template(template: &str, replacements: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (placeholder, value) in replacements {
        result = result.replace(placeholder, value);
    }
    result
}

/// Write processed content to file and update mod.rs
fn write_generated_file(
    output_path: &PathBuf,
    content: &str,
    update_mod: bool,
    mod_name: &str,
) -> Result<()> {
    // Create parent directories
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write file
    fs::write(output_path, content)?;

    // Update mod.rs if requested
    if update_mod {
        if let Some(parent) = output_path.parent() {
            update_mod_file(parent, mod_name)?;
        }
    }

    Ok(())
}

/// Update mod.rs to include new module
fn update_mod_file(dir: &std::path::Path, module_name: &str) -> Result<()> {
    let mod_path = dir.join("mod.rs");

    let mut content = if mod_path.exists() {
        fs::read_to_string(&mod_path)?
    } else {
        "//! Module exports\n\n".to_string()
    };

    let module_decl = format!("pub mod {};", module_name);

    if !content.contains(&module_decl) {
        content.push_str(&module_decl);
        content.push('\n');
        fs::write(&mod_path, content)?;
    }

    Ok(())
}

// ============================================================================
// Case Conversion Utilities
// ============================================================================

/// Convert string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '_' || c == '-')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

// ============================================================================
// Delegating Commands
// ============================================================================

/// Create a new module (delegates to module create)
async fn make_module(name: &str, description: Option<&str>) -> Result<()> {
    println!(
        "{}",
        format!("📦 Creating module: {}", name).bright_cyan().bold()
    );
    println!();

    let module_action = crate::commands::module::ModuleAction::Create {
        name: name.to_string(),
        author: "Metaphor Developer".to_string(),
        description: description.map(String::from),
    };

    crate::commands::module::handle_command(&module_action).await
}

/// Create a new entity (schema-first approach)
///
/// NOTE: This command is deprecated. Use schema files instead:
/// 1. Create schema file: libs/modules/{module}/schema/models/{entity}.model.yaml
/// 2. Run: metaphor schema generate {module} --target proto,rust
async fn make_entity(name: &str, module: &str, _soft_delete: bool, _versioned: bool) -> Result<()> {
    println!(
        "{}",
        format!("🏗️ Creating entity: {} in module {}", name, module)
            .bright_cyan()
            .bold()
    );
    println!();
    println!(
        "  {} This command now uses schema-first approach.",
        "ℹ️".bright_blue(),
    );
    println!();
    println!("  📝 To create an entity, follow these steps:");
    println!();
    println!("  1. Create schema file:");
    println!("     libs/modules/{}/schema/models/{}.model.yaml", module, name.to_lowercase());
    println!();
    println!("  2. Define your entity in the schema file:");
    println!("     models:");
    println!("       - name: {}", name);
    println!("         collection: {}s", name.to_lowercase());
    println!("         fields:");
    println!("           id:");
    println!("             type: uuid");
    println!("             attributes: [\"@id\", \"@default(uuid)\"]");
    println!("           # Add your fields here...");
    println!();
    println!("  3. Generate code from schema:");
    println!("     metaphor schema generate {} --target proto,rust,sql", module);
    println!();
    println!(
        "  {} Schema-first ensures consistency across proto, Rust, and SQL.",
        "💡".bright_yellow()
    );

    Ok(())
}

/// Create a test file (delegates to metaphor-dev test generate)
async fn make_test(name: &str, module: &str, test_type: &str) -> Result<()> {
    println!("Test generation is handled by the metaphor-dev plugin.");
    println!("Run: {} {} {} --module {}",
        "metaphor-dev".bright_cyan(),
        "test".bright_cyan(),
        "generate".bright_cyan(),
        module.bright_yellow(),
    );
    println!("  --entity {} --{}", name, test_type);
    Ok(())
}

// ============================================================================
// Template-Based Commands
// ============================================================================

/// Create a CQRS command
async fn make_command(name: &str, module: &str, entity: &str) -> Result<()> {
    println!(
        "{}",
        format!("📝 Creating command: {} for entity {}", name, entity)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let command_snake = to_snake_case(name);
    let command_pascal = to_pascal_case(name);
    let entity_pascal = to_pascal_case(entity);

    // Load and process template
    let template = load_template("command", "{{COMMAND_NAME_SNAKE}}.rs")?;

    let mut replacements = HashMap::new();
    replacements.insert("{{COMMAND_NAME}}".to_string(), command_pascal.clone());
    replacements.insert("{{COMMAND_NAME_SNAKE}}".to_string(), command_snake.clone());
    replacements.insert(
        "{{COMMAND_DESCRIPTION}}".to_string(),
        command_snake.replace('_', " "),
    );
    replacements.insert("{{ENTITY_NAME}}".to_string(), entity_pascal);

    let content = process_template(&template, &replacements);

    // Write file
    let commands_dir = module_path.join("src/application/commands");
    let output_path = commands_dir.join(format!("{}.rs", command_snake));

    write_generated_file(&output_path, &content, true, &command_snake)?;

    println!(
        "  {} Created: src/application/commands/{}.rs",
        "✅".green(),
        command_snake
    );
    println!();
    println!(
        "{}",
        format!("Command {} created! 🎉", command_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a CQRS query
async fn make_query(name: &str, module: &str, entity: &str) -> Result<()> {
    println!(
        "{}",
        format!("🔍 Creating query: {} for entity {}", name, entity)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let query_snake = to_snake_case(name);
    let query_pascal = to_pascal_case(name);
    let entity_pascal = to_pascal_case(entity);

    // Load and process template
    let template = load_template("query", "{{QUERY_NAME_SNAKE}}.rs")?;

    let mut replacements = HashMap::new();
    replacements.insert("{{QUERY_NAME}}".to_string(), query_pascal.clone());
    replacements.insert("{{QUERY_NAME_SNAKE}}".to_string(), query_snake.clone());
    replacements.insert("{{ENTITY_NAME}}".to_string(), entity_pascal);

    let content = process_template(&template, &replacements);

    // Write file
    let queries_dir = module_path.join("src/application/queries");
    let output_path = queries_dir.join(format!("{}.rs", query_snake));

    write_generated_file(&output_path, &content, true, &query_snake)?;

    println!(
        "  {} Created: src/application/queries/{}.rs",
        "✅".green(),
        query_snake
    );
    println!();
    println!(
        "{}",
        format!("Query {} created! 🎉", query_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a repository
async fn make_repository(name: &str, module: &str, database: &str) -> Result<()> {
    println!(
        "{}",
        format!("🗃️ Creating repository for {} using {}", name, database)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let entity_snake = to_snake_case(name);
    let entity_pascal = to_pascal_case(name);
    let collection = format!("{}s", entity_snake);

    let mut replacements = HashMap::new();
    replacements.insert("{{ENTITY_NAME}}".to_string(), entity_pascal.clone());
    replacements.insert("{{ENTITY_NAME_SNAKE}}".to_string(), entity_snake.clone());
    replacements.insert("{{COLLECTION}}".to_string(), collection);

    // Create repository trait
    let trait_template = load_template("repository", "{{ENTITY_NAME_SNAKE}}_repository.rs")?;
    let trait_content = process_template(&trait_template, &replacements);

    let repo_dir = module_path.join("src/domain/repository");
    let trait_path = repo_dir.join(format!("{}_repository.rs", entity_snake));

    write_generated_file(&trait_path, &trait_content, true, &format!("{}_repository", entity_snake))?;

    println!(
        "  {} Created: src/domain/repository/{}_repository.rs",
        "✅".green(),
        entity_snake
    );

    // Create implementation
    let impl_template_name = match database {
        "postgres" => "postgres_{{ENTITY_NAME_SNAKE}}_repository.rs",
        _ => "mongo_{{ENTITY_NAME_SNAKE}}_repository.rs",
    };

    let impl_template = load_template("repository", impl_template_name)?;
    let impl_content = process_template(&impl_template, &replacements);

    let impl_dir = module_path.join("src/infrastructure/persistence");
    let impl_path = impl_dir.join(format!("{}_{}_repository.rs", database, entity_snake));

    write_generated_file(
        &impl_path,
        &impl_content,
        true,
        &format!("{}_{}_repository", database, entity_snake),
    )?;

    println!(
        "  {} Created: src/infrastructure/persistence/{}_{}_repository.rs",
        "✅".green(),
        database,
        entity_snake
    );
    println!();
    println!(
        "{}",
        format!("Repository for {} created! 🎉", entity_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create an HTTP handler
async fn make_handler(name: &str, module: &str, crud: bool) -> Result<()> {
    println!(
        "{}",
        format!("🌐 Creating HTTP handler: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let handler_snake = to_snake_case(name);
    let handler_pascal = to_pascal_case(name);
    let collection = format!("{}s", handler_snake);

    let mut replacements = HashMap::new();
    replacements.insert("{{HANDLER_NAME}}".to_string(), handler_pascal.clone());
    replacements.insert("{{HANDLER_NAME_SNAKE}}".to_string(), handler_snake.clone());
    replacements.insert("{{COLLECTION}}".to_string(), collection);

    let template_name = if crud {
        "{{HANDLER_NAME_SNAKE}}_crud_handler.rs"
    } else {
        "{{HANDLER_NAME_SNAKE}}_handler.rs"
    };

    let template = load_template("handler", template_name)?;
    let content = process_template(&template, &replacements);

    let handlers_dir = module_path.join("src/presentation/http");
    let output_path = handlers_dir.join(format!("{}_handler.rs", handler_snake));

    write_generated_file(&output_path, &content, true, &format!("{}_handler", handler_snake))?;

    println!(
        "  {} Created: src/presentation/http/{}_handler.rs",
        "✅".green(),
        handler_snake
    );
    println!();
    println!(
        "{}",
        format!("Handler {} created! 🎉", handler_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a domain service
async fn make_service(name: &str, module: &str) -> Result<()> {
    println!(
        "{}",
        format!("⚙️ Creating domain service: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let service_snake = to_snake_case(name);
    let service_pascal = to_pascal_case(name);

    let mut replacements = HashMap::new();
    replacements.insert("{{SERVICE_NAME}}".to_string(), service_pascal.clone());
    replacements.insert("{{SERVICE_NAME_SNAKE}}".to_string(), service_snake.clone());
    replacements.insert(
        "{{SERVICE_DESCRIPTION}}".to_string(),
        service_snake.replace('_', " "),
    );

    let template = load_template("service", "{{SERVICE_NAME_SNAKE}}.rs")?;
    let content = process_template(&template, &replacements);

    let services_dir = module_path.join("src/domain/service");
    let output_path = services_dir.join(format!("{}.rs", service_snake));

    write_generated_file(&output_path, &content, true, &service_snake)?;

    println!(
        "  {} Created: src/domain/service/{}.rs",
        "✅".green(),
        service_snake
    );
    println!();
    println!(
        "{}",
        format!("Service {} created! 🎉", service_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a domain event
async fn make_event(name: &str, module: &str, entity: Option<&str>) -> Result<()> {
    println!(
        "{}",
        format!("📢 Creating domain event: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let event_snake = to_snake_case(name);
    let event_pascal = to_pascal_case(name);
    let entity_snake = entity.map(to_snake_case).unwrap_or_else(|| "entity".to_string());

    let mut replacements = HashMap::new();
    replacements.insert("{{EVENT_NAME}}".to_string(), event_pascal.clone());
    replacements.insert("{{EVENT_NAME_SNAKE}}".to_string(), event_snake.clone());
    replacements.insert("{{ENTITY_NAME_SNAKE}}".to_string(), entity_snake);
    replacements.insert("{{MODULE_NAME}}".to_string(), module.to_string());

    // Create proto event
    let proto_template = load_template("event", "{{EVENT_NAME_SNAKE}}.proto")?;
    let proto_content = process_template(&proto_template, &replacements);

    let proto_dir = module_path.join("proto/domain/event");
    let proto_path = proto_dir.join(format!("{}.proto", event_snake));

    write_generated_file(&proto_path, &proto_content, false, "")?;

    println!(
        "  {} Created: proto/domain/event/{}.proto",
        "✅".green(),
        event_snake
    );

    // Create Rust event handler
    let rust_template = load_template("event", "{{EVENT_NAME_SNAKE}}.rs")?;
    let rust_content = process_template(&rust_template, &replacements);

    let events_dir = module_path.join("src/domain/event");
    let rust_path = events_dir.join(format!("{}.rs", event_snake));

    write_generated_file(&rust_path, &rust_content, true, &event_snake)?;

    println!(
        "  {} Created: src/domain/event/{}.rs",
        "✅".green(),
        event_snake
    );
    println!();
    println!(
        "{}",
        format!("Event {} created! 🎉", event_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a database migration
async fn make_migration(
    name: &str,
    module: &str,
    create: Option<&str>,
    table: Option<&str>,
) -> Result<()> {
    println!(
        "{}",
        format!("🗄️ Creating migration: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let migrations_dir = module_path.join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let migration_name = format!("{}_{}", timestamp, name);

    let mut replacements = HashMap::new();
    replacements.insert("{{MIGRATION_NAME}}".to_string(), migration_name.clone());
    replacements.insert("{{MIGRATION_DESCRIPTION}}".to_string(), name.to_string());

    let (up_template_name, down_template_name) = if let Some(table_name) = create {
        let table_snake = to_snake_case(table_name);
        replacements.insert("{{TABLE_NAME}}".to_string(), table_snake);
        ("create_{{TABLE_NAME}}.up.sql", "create_{{TABLE_NAME}}.down.sql")
    } else if let Some(table_name) = table {
        let table_snake = to_snake_case(table_name);
        replacements.insert("{{TABLE_NAME}}".to_string(), table_snake);
        ("alter_{{TABLE_NAME}}.up.sql", "alter_{{TABLE_NAME}}.down.sql")
    } else {
        ("{{MIGRATION_NAME}}.up.sql", "{{MIGRATION_NAME}}.down.sql")
    };

    // Load and process templates
    let up_template = load_template("migration", up_template_name)?;
    let down_template = load_template("migration", down_template_name)?;

    let up_content = process_template(&up_template, &replacements);
    let down_content = process_template(&down_template, &replacements);

    // Write migration files
    let up_path = migrations_dir.join(format!("{}.up.sql", migration_name));
    let down_path = migrations_dir.join(format!("{}.down.sql", migration_name));

    fs::write(&up_path, up_content)?;
    fs::write(&down_path, down_content)?;

    println!(
        "  {} Created: migrations/{}.up.sql",
        "✅".green(),
        migration_name
    );
    println!(
        "  {} Created: migrations/{}.down.sql",
        "✅".green(),
        migration_name
    );
    println!();
    println!(
        "{}",
        format!("Migration {} created! 🎉", migration_name)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a value object
async fn make_value_object(name: &str, module: &str) -> Result<()> {
    println!(
        "{}",
        format!("💎 Creating value object: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let vo_snake = to_snake_case(name);
    let vo_pascal = to_pascal_case(name);

    let mut replacements = HashMap::new();
    replacements.insert("{{VALUE_OBJECT_NAME}}".to_string(), vo_pascal.clone());
    replacements.insert("{{VALUE_OBJECT_NAME_SNAKE}}".to_string(), vo_snake.clone());
    replacements.insert("{{MODULE_NAME}}".to_string(), module.to_string());

    // Create proto definition
    let proto_template = load_template("value_object", "{{VALUE_OBJECT_NAME_SNAKE}}.proto")?;
    let proto_content = process_template(&proto_template, &replacements);

    let proto_dir = module_path.join("proto/domain/value_object");
    let proto_path = proto_dir.join(format!("{}.proto", vo_snake));

    write_generated_file(&proto_path, &proto_content, false, "")?;

    println!(
        "  {} Created: proto/domain/value_object/{}.proto",
        "✅".green(),
        vo_snake
    );

    // Create Rust implementation
    let rust_template = load_template("value_object", "{{VALUE_OBJECT_NAME_SNAKE}}.rs")?;
    let rust_content = process_template(&rust_template, &replacements);

    let vo_dir = module_path.join("src/domain/value_object");
    let rust_path = vo_dir.join(format!("{}.rs", vo_snake));

    write_generated_file(&rust_path, &rust_content, true, &vo_snake)?;

    println!(
        "  {} Created: src/domain/value_object/{}.rs",
        "✅".green(),
        vo_snake
    );
    println!();
    println!(
        "{}",
        format!("Value object {} created! 🎉", vo_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Create a specification (business rule)
async fn make_specification(name: &str, module: &str) -> Result<()> {
    println!(
        "{}",
        format!("📋 Creating specification: {}", name)
            .bright_cyan()
            .bold()
    );
    println!();

    let module_path = PathBuf::from("libs/modules").join(module);
    if !module_path.exists() {
        anyhow::bail!("Module '{}' not found", module);
    }

    let spec_snake = to_snake_case(name);
    let spec_pascal = to_pascal_case(name);

    let mut replacements = HashMap::new();
    replacements.insert("{{SPEC_NAME}}".to_string(), spec_pascal.clone());
    replacements.insert("{{SPEC_NAME_SNAKE}}".to_string(), spec_snake.clone());

    let template = load_template("specification", "{{SPEC_NAME_SNAKE}}.rs")?;
    let content = process_template(&template, &replacements);

    let specs_dir = module_path.join("src/domain/specification");
    let output_path = specs_dir.join(format!("{}.rs", spec_snake));

    write_generated_file(&output_path, &content, true, &spec_snake)?;

    println!(
        "  {} Created: src/domain/specification/{}.rs",
        "✅".green(),
        spec_snake
    );
    println!();
    println!(
        "{}",
        format!("Specification {} created! 🎉", spec_pascal)
            .bright_green()
            .bold()
    );

    Ok(())
}
