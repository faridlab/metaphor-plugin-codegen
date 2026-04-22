//! Migration generation command
//!
//! Generates PostgreSQL migration files from entity definitions.
//! Each entity gets its own migration file with:
//! - CREATE TABLE statement with all fields
//! - Proper indexes for common query patterns
//! - Triggers for updated_at timestamp
//! - Soft delete support

use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::fs;
use std::path::Path;

// Use centralized database config from utils module
use crate::utils::sanitize_db_url;

/// Represents a parsed entity field
#[derive(Debug, Clone)]
struct EntityField {
    name: String,
    rust_type: String,
    is_optional: bool,
}

/// Represents a parsed entity with its fields
#[derive(Debug, Clone)]
struct ParsedEntity {
    name: String,
    fields: Vec<EntityField>,
}

/// Migration action enum
#[derive(Debug, Clone)]
pub enum MigrationAction {
    /// Generate a migration for a specific entity
    Generate {
        /// Entity name (PascalCase)
        entity: String,
        /// Target module name
        module: String,
        /// Force overwrite existing migration
        force: bool,
    },
    /// Generate migrations for all entities in a module
    GenerateAll {
        /// Target module name
        module: String,
        /// Force overwrite existing migrations
        force: bool,
    },
    /// List existing migrations for a module
    List {
        /// Target module name
        module: String,
    },
    /// Generate ALTER TABLE migration to add new fields
    Alter {
        /// Entity name (PascalCase)
        entity: String,
        /// Target module name
        module: String,
        /// Description of the change (e.g., "add_profile_fields")
        description: String,
    },
    /// Diff entity fields vs existing migration to detect changes
    Diff {
        /// Entity name (PascalCase)
        entity: String,
        /// Target module name
        module: String,
    },
    /// Run pending migrations using sqlx
    Run {
        /// Target module name
        module: String,
        /// Database URL (defaults to DATABASE_URL env var)
        database_url: Option<String>,
    },
    /// Run database seeders
    Seed {
        /// Target module name
        module: String,
        /// Specific seeder to run (optional, runs all if not specified)
        name: Option<String>,
        /// Force run even if already applied
        force: bool,
        /// Database URL (defaults to DATABASE_URL env var)
        database_url: Option<String>,
    },
    /// Generate SQL seed files from Rust seeders
    GenerateSeeds {
        /// Target module name
        module: String,
        /// Force overwrite existing SQL seed files
        force: bool,
    },
    /// Run pending migrations for ALL registered modules
    RunAll {
        /// Database URL (defaults to DATABASE_URL env var)
        database_url: Option<String>,
    },
    /// Show migration status for all modules
    Status {
        /// Filter by specific module (optional, shows all if not specified)
        module: Option<String>,
        /// Database URL (defaults to DATABASE_URL env var)
        database_url: Option<String>,
    },
}

/// Handle migration commands
pub async fn handle_command(action: &MigrationAction) -> Result<()> {
    match action {
        MigrationAction::Generate { entity, module, force } => {
            generate_entity_migration(entity, module, *force).await
        }
        MigrationAction::GenerateAll { module, force } => {
            generate_all_migrations(module, *force).await
        }
        MigrationAction::List { module } => {
            list_migrations(module).await
        }
        MigrationAction::Alter { entity, module, description } => {
            generate_alter_migration(entity, module, description).await
        }
        MigrationAction::Diff { entity, module } => {
            diff_entity_migration(entity, module).await
        }
        MigrationAction::Run { module, database_url } => {
            run_migrations(module, database_url.as_deref()).await
        }
        MigrationAction::Seed { module, name, force, database_url } => {
            run_seeders(module, name.as_deref(), *force, database_url.as_deref()).await
        }
        MigrationAction::GenerateSeeds { module, force } => {
            generate_sql_seeds(module, *force).await
        }
        MigrationAction::RunAll { database_url } => {
            run_all_migrations(database_url.as_deref()).await
        }
        MigrationAction::Status { module, database_url } => {
            show_migration_status(module.as_deref(), database_url.as_deref()).await
        }
    }
}

/// Generate migration for a single entity
async fn generate_entity_migration(entity: &str, module: &str, force: bool) -> Result<()> {
    println!("🗄️  {} PostgreSQL migration for {} in module {}...",
        "Generating".bright_green(),
        entity.bright_cyan(),
        module.bright_yellow()
    );

    let migrations_dir = get_migrations_dir(module);
    fs::create_dir_all(&migrations_dir)?;

    // Generate migration number based on existing files
    let migration_number = get_next_migration_number(&migrations_dir)?;
    let entity_snake = to_snake_case(entity);
    let table_name = to_plural(&entity_snake);

    let migration_filename = format!("{:03}_create_{}_table.sql", migration_number, table_name);
    let migration_path = migrations_dir.join(&migration_filename);

    if migration_path.exists() && !force {
        println!("⚠️  {} Migration already exists: {}",
            "Skipped".bright_yellow(),
            migration_path.display()
        );
        println!("   Use --force to overwrite");
        return Ok(());
    }

    // Generate migration content
    let migration_content = generate_migration_sql(entity, &entity_snake, &table_name);

    fs::write(&migration_path, migration_content)?;

    println!("✅ {} migration: {}",
        "Created".bright_green(),
        migration_path.display().to_string().bright_cyan()
    );

    show_next_steps(&migration_path);
    Ok(())
}

/// Generate migrations for all entities found in module's entity_models.rs
async fn generate_all_migrations(module: &str, force: bool) -> Result<()> {
    println!("🗄️  {} PostgreSQL migrations for all entities in module {}...",
        "Generating".bright_green(),
        module.bright_yellow()
    );

    // Look for entity definitions in the module
    let entity_models_path = module_base_path(module)
        .join("src/infrastructure/persistence/postgresql/entity_models.rs");

    if !entity_models_path.exists() {
        return Err(anyhow::anyhow!(
            "Entity models file not found at: {}\nPlease create entity models first.",
            entity_models_path.display()
        ));
    }

    // Parse entities with their fields from the file
    let content = fs::read_to_string(&entity_models_path)?;
    let entities = parse_entities_with_fields(&content);

    if entities.is_empty() {
        println!("⚠️  No entities found in {}", entity_models_path.display());
        return Ok(());
    }

    let entity_names: Vec<_> = entities.iter().map(|e| e.name.clone()).collect();
    println!("📋 Found {} entities: {}",
        entities.len().to_string().bright_cyan(),
        entity_names.join(", ").bright_yellow()
    );
    println!();

    let migrations_dir = get_migrations_dir(module);
    fs::create_dir_all(&migrations_dir)?;

    for (i, entity) in entities.iter().enumerate() {
        let entity_snake = to_snake_case(&entity.name);
        let table_name = to_plural(&entity_snake);

        let migration_filename = format!("{:03}_create_{}_table.sql", i + 1, table_name);
        let migration_path = migrations_dir.join(&migration_filename);

        if migration_path.exists() && !force {
            println!("⚠️  {} {}: already exists",
                "Skipped".bright_yellow(),
                entity.name.bright_cyan()
            );
            continue;
        }

        let migration_content = generate_migration_sql_with_fields(&entity.name, &table_name, &entity.fields);
        fs::write(&migration_path, &migration_content)?;

        println!("✅ {} {}: {} ({} fields)",
            "Created".bright_green(),
            entity.name.bright_cyan(),
            migration_filename.bright_white(),
            entity.fields.len()
        );
    }

    println!();
    println!("🎉 {} Generated {} migration files",
        "Done!".bright_green().bold(),
        entities.len()
    );

    Ok(())
}

/// List existing migrations for a module
async fn list_migrations(module: &str) -> Result<()> {
    let migrations_dir = get_migrations_dir(module);

    if !migrations_dir.exists() {
        println!("📂 No migrations directory found for module {}", module.bright_yellow());
        return Ok(());
    }

    println!("📋 {} for module {}:",
        "Migrations".bright_cyan(),
        module.bright_yellow()
    );
    println!();

    let mut entries: Vec<_> = fs::read_dir(&migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("   (no migrations found)");
        return Ok(());
    }

    for entry in entries {
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();
        println!("   📄 {}", filename_str.bright_white());
    }

    Ok(())
}

/// Generate SQL migration content for an entity
fn generate_migration_sql(entity: &str, _entity_snake: &str, table_name: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(r#"-- Migration: Create {table_name} table
-- Entity: {entity}
-- Generated: {timestamp}
--
-- This migration creates the {table_name} table with:
-- - UUID primary key
-- - Standard timestamp fields (created_at, updated_at, deleted_at)
-- - Indexes for common query patterns
-- - Trigger for automatic updated_at

-- ============================================================================
-- Table: {table_name}
-- ============================================================================

CREATE TABLE IF NOT EXISTS {table_name} (
    -- Primary key (native UUID type for better performance and validation)
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- TODO: Add your entity-specific fields here
    -- Example fields (customize based on your entity):
    -- name VARCHAR(255) NOT NULL,
    -- description TEXT,
    -- status VARCHAR(50) DEFAULT 'active',
    -- is_active BOOLEAN DEFAULT true,

    -- Timestamps (standard for all entities)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ  -- NULL means not deleted (soft delete)
);

-- ============================================================================
-- Indexes
-- ============================================================================

-- Index for soft delete queries (most common pattern)
CREATE INDEX IF NOT EXISTS idx_{table_name}_deleted_at
    ON {table_name}(deleted_at)
    WHERE deleted_at IS NULL;

-- Index for listing by creation date
CREATE INDEX IF NOT EXISTS idx_{table_name}_created_at
    ON {table_name}(created_at DESC);

-- Index for listing deleted items (trash)
CREATE INDEX IF NOT EXISTS idx_{table_name}_deleted
    ON {table_name}(deleted_at DESC)
    WHERE deleted_at IS NOT NULL;

-- ============================================================================
-- Triggers
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_{table_name}_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update updated_at on row update
DROP TRIGGER IF EXISTS trigger_{table_name}_updated_at ON {table_name};
CREATE TRIGGER trigger_{table_name}_updated_at
    BEFORE UPDATE ON {table_name}
    FOR EACH ROW
    EXECUTE FUNCTION update_{table_name}_updated_at();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE {table_name} IS '{entity} entity table - auto-generated by metaphor CLI';
COMMENT ON COLUMN {table_name}.id IS 'Unique identifier (UUID)';
COMMENT ON COLUMN {table_name}.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN {table_name}.updated_at IS 'Last update timestamp (auto-updated)';
COMMENT ON COLUMN {table_name}.deleted_at IS 'Soft delete timestamp (NULL if not deleted)';
"#,
        table_name = table_name,
        entity = entity,
        timestamp = timestamp,
    )
}


/// Parse entities with their fields from entity_models.rs content
fn parse_entities_with_fields(content: &str) -> Vec<ParsedEntity> {
    let mut entities = Vec::new();
    let mut current_entity: Option<ParsedEntity> = None;
    let mut in_struct = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Early returns for cleaner control flow
        if let Some(entity_name) = try_extract_entity_name(trimmed) {
            current_entity = Some(ParsedEntity {
                name: entity_name,
                fields: Vec::new(),
            });
            in_struct = true;
            brace_depth = 1;
            continue;
        }

        if !in_struct {
            continue;
        }

        brace_depth = update_brace_depth(trimmed, brace_depth);

        if brace_depth == 0 {
            if let Some(entity) = current_entity.take() {
                entities.push(entity);
            }
            in_struct = false;
            continue;
        }

        if let Some(field) = parse_field_line(trimmed) {
            if let Some(ref mut entity) = current_entity {
                entity.fields.push(field);
            }
        }
    }

    entities
}

/// Try to extract entity name from a struct declaration line
///
/// Looks for patterns like `pub struct MyEntity {` and returns "My"
fn try_extract_entity_name(line: &str) -> Option<String> {
    if !(line.starts_with("pub struct ") && line.contains("Entity") && line.ends_with('{')) {
        return None;
    }

    let start = line.find("struct ")?;
    let after_struct = &line[start + 7..];
    let end = after_struct.find("Entity")?;
    let entity_name = &after_struct[..end];

    if entity_name.is_empty() {
        return None;
    }

    Some(entity_name.to_string())
}

/// Update brace depth by counting braces in a line
///
/// Returns the new brace depth after processing the line.
fn update_brace_depth(line: &str, current_depth: usize) -> usize {
    let mut depth = current_depth;
    for c in line.chars() {
        match c {
            '{' => depth += 1,
            '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }
    depth
}

/// Parse a single field line from a Rust struct
fn parse_field_line(line: &str) -> Option<EntityField> {
    // Remove "pub " prefix
    let line = line.strip_prefix("pub ")?;

    // Split by ":"
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let mut type_part = parts[1].trim();

    // Remove trailing comma
    if type_part.ends_with(',') {
        type_part = &type_part[..type_part.len() - 1];
    }

    let is_optional = type_part.starts_with("Option<");
    let rust_type = if is_optional {
        // Extract inner type from Option<Type>
        type_part
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(type_part)
            .to_string()
    } else {
        type_part.to_string()
    };

    Some(EntityField {
        name,
        rust_type,
        is_optional,
    })
}

/// Convert Rust type to PostgreSQL type
fn rust_type_to_sql(rust_type: &str, field_name: &str) -> &'static str {
    // Foreign key fields (ending with _id) should be UUID
    if field_name.ends_with("_id") {
        return "UUID";
    }

    match rust_type.trim() {
        "String" => "VARCHAR(255)",
        "i32" | "i64" => "BIGINT",
        "i16" => "SMALLINT",
        "u32" | "u64" => "BIGINT",
        "f32" | "f64" => "DOUBLE PRECISION",
        "bool" => "BOOLEAN",
        "DateTime<Utc>" => "TIMESTAMPTZ",
        "chrono::DateTime<chrono::Utc>" => "TIMESTAMPTZ",
        "Vec<String>" => "TEXT[]",
        "serde_json::Value" => "JSONB",
        _ if rust_type.contains("DateTime") => "TIMESTAMPTZ",
        _ if rust_type.contains("Vec<") => "TEXT[]",
        _ => "TEXT",
    }
}

/// Generate SQL migration content with actual entity fields
fn generate_migration_sql_with_fields(entity: &str, table_name: &str, fields: &[EntityField]) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    // Build field definitions
    let mut field_defs = Vec::new();
    let mut indexes = Vec::new();
    let mut comments = Vec::new();

    for field in fields {
        // Skip id, created_at, updated_at, deleted_at as they're added separately
        if matches!(field.name.as_str(), "id" | "created_at" | "updated_at" | "deleted_at") {
            continue;
        }

        let sql_type = rust_type_to_sql(&field.rust_type, &field.name);
        let null_constraint = if field.is_optional { "" } else { " NOT NULL" };

        // Add default for boolean fields
        let default = if field.rust_type == "bool" {
            " DEFAULT false"
        } else {
            ""
        };

        field_defs.push(format!("    {} {}{}{}", field.name, sql_type, null_constraint, default));

        // Add indexes for common lookup fields
        if field.name.ends_with("_id") || field.name == "email" || field.name == "username" ||
           field.name == "name" || field.name == "key" || field.name == "token_hash" {
            indexes.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_{}\n    ON {}({});",
                table_name, field.name, table_name, field.name
            ));
        }

        // Add comments for each field
        comments.push(format!(
            "COMMENT ON COLUMN {}.{} IS '{} field';",
            table_name, field.name, field.name.replace('_', " ")
        ));
    }

    let fields_sql = if field_defs.is_empty() {
        "    -- No additional fields defined".to_string()
    } else {
        field_defs.join(",\n")
    };

    let additional_indexes = if indexes.is_empty() {
        "-- No additional indexes".to_string()
    } else {
        indexes.join("\n\n")
    };

    let additional_comments = if comments.is_empty() {
        "".to_string()
    } else {
        comments.join("\n")
    };

    format!(r#"-- Migration: Create {table_name} table
-- Entity: {entity}
-- Generated: {timestamp}
-- Fields: {field_count} custom + 4 standard (id, created_at, updated_at, deleted_at)
--
-- This migration creates the {table_name} table with:
-- - UUID primary key
-- - Entity-specific fields (auto-parsed from entity_models.rs)
-- - Standard timestamp fields (created_at, updated_at, deleted_at)
-- - Indexes for common query patterns
-- - Trigger for automatic updated_at

-- ============================================================================
-- Table: {table_name}
-- ============================================================================

CREATE TABLE IF NOT EXISTS {table_name} (
    -- Primary key (native UUID type for better performance and validation)
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Entity-specific fields
{fields_sql},

    -- Timestamps (standard for all entities)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ  -- NULL means not deleted (soft delete)
);

-- ============================================================================
-- Indexes
-- ============================================================================

-- Index for soft delete queries (most common pattern)
CREATE INDEX IF NOT EXISTS idx_{table_name}_deleted_at
    ON {table_name}(deleted_at)
    WHERE deleted_at IS NULL;

-- Index for listing by creation date
CREATE INDEX IF NOT EXISTS idx_{table_name}_created_at
    ON {table_name}(created_at DESC);

-- Index for listing deleted items (trash)
CREATE INDEX IF NOT EXISTS idx_{table_name}_deleted
    ON {table_name}(deleted_at DESC)
    WHERE deleted_at IS NOT NULL;

-- Entity-specific indexes
{additional_indexes}

-- ============================================================================
-- Triggers
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_{table_name}_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update updated_at on row update
DROP TRIGGER IF EXISTS trigger_{table_name}_updated_at ON {table_name};
CREATE TRIGGER trigger_{table_name}_updated_at
    BEFORE UPDATE ON {table_name}
    FOR EACH ROW
    EXECUTE FUNCTION update_{table_name}_updated_at();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE {table_name} IS '{entity} entity table - auto-generated by metaphor CLI';
COMMENT ON COLUMN {table_name}.id IS 'Unique identifier (UUID)';
COMMENT ON COLUMN {table_name}.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN {table_name}.updated_at IS 'Last update timestamp (auto-updated)';
COMMENT ON COLUMN {table_name}.deleted_at IS 'Soft delete timestamp (NULL if not deleted)';
{additional_comments}
"#,
        table_name = table_name,
        entity = entity,
        timestamp = timestamp,
        field_count = fields.len().saturating_sub(4), // Subtract standard fields
        fields_sql = fields_sql,
        additional_indexes = additional_indexes,
        additional_comments = additional_comments,
    )
}

/// Get migrations directory path for a module.
///
/// Prefers top-level `migrations/` because that's where `metaphor-plugin-schema`
/// emits. Falls back to `migrations/postgres/` for legacy per-backend layouts.
/// The preference must match `run_migrations()`'s directory selection logic.
fn get_migrations_dir(module: &str) -> std::path::PathBuf {
    let base = module_base_path(module);
    let root_dir = base.join("migrations");
    let postgres_dir = root_dir.join("postgres");

    let has_sql_files = |dir: &Path| -> bool {
        fs::read_dir(dir)
            .map(|entries| entries.filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .any(|e| e.path().extension().map_or(false, |ext| ext == "sql")))
            .unwrap_or(false)
    };

    if root_dir.exists() && has_sql_files(&root_dir) {
        return root_dir;
    }
    if postgres_dir.exists() && has_sql_files(&postgres_dir) {
        return postgres_dir;
    }
    // Neither has files — return root so callers get a sensible "not found" path.
    root_dir
}

/// Get the next migration number based on existing files
fn get_next_migration_number(migrations_dir: &Path) -> Result<u32> {
    if !migrations_dir.exists() {
        return Ok(1);
    }

    let max_number = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let filename = e.file_name();
            let name = filename.to_string_lossy();
            // Extract number from "001_create_xxx.sql"
            name.split('_').next()?.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0);

    Ok(max_number + 1)
}

/// Convert PascalCase to snake_case
fn to_snake_case(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }

    input.chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect()
}

/// Convert singular to plural (simple English rules)
fn to_plural(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }

    // Handle common irregular plurals
    match input {
        "person" => "people".to_string(),
        "child" => "children".to_string(),
        "man" => "men".to_string(),
        "woman" => "women".to_string(),
        _ => {
            if input.ends_with('s') || input.ends_with('x') || input.ends_with('z') ||
               input.ends_with("ch") || input.ends_with("sh") {
                format!("{}es", input)
            } else if input.ends_with('y') && input.len() > 1 {
                let consonants = "bcdfghjklmnpqrstvwxyz";
                let second_last = input.chars().nth(input.len() - 2).unwrap_or('a');
                if consonants.contains(second_last) {
                    let base = &input[..input.len()-1];
                    format!("{}ies", base)
                } else {
                    format!("{}s", input)
                }
            } else {
                format!("{}s", input)
            }
        }
    }
}

/// Generate ALTER TABLE migration to add new fields
async fn generate_alter_migration(entity: &str, module: &str, description: &str) -> Result<()> {
    println!("🔄 {} ALTER migration for {} in module {}...",
        "Generating".bright_green(),
        entity.bright_cyan(),
        module.bright_yellow()
    );

    let entity_snake = to_snake_case(entity);
    let table_name = to_plural(&entity_snake);

    // Parse current entity fields from entity_models.rs
    let entity_models_path = module_base_path(module)
        .join("src/infrastructure/persistence/postgresql/entity_models.rs");

    if !entity_models_path.exists() {
        return Err(anyhow::anyhow!(
            "Entity models file not found at: {}",
            entity_models_path.display()
        ));
    }

    let content = fs::read_to_string(&entity_models_path)?;
    let entities = parse_entities_with_fields(&content);
    let parsed_entity = entities.iter()
        .find(|e| e.name == entity)
        .ok_or_else(|| anyhow::anyhow!("Entity {} not found in entity_models.rs", entity))?;

    // Parse existing migration to find which fields already exist
    let migrations_dir = get_migrations_dir(module);
    let existing_fields = parse_existing_migration_fields(&migrations_dir, &table_name)?;

    // Find new fields (in entity but not in migration)
    let new_fields: Vec<_> = parsed_entity.fields.iter()
        .filter(|f| !matches!(f.name.as_str(), "id" | "created_at" | "updated_at" | "deleted_at"))
        .filter(|f| !existing_fields.contains(&f.name))
        .collect();

    if new_fields.is_empty() {
        println!("✅ {} No new fields to add - entity and migration are in sync",
            "Up to date!".bright_green()
        );
        return Ok(());
    }

    println!("📋 Found {} new fields: {}",
        new_fields.len().to_string().bright_cyan(),
        new_fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ").bright_yellow()
    );

    // Generate ALTER TABLE migration
    let migration_number = get_next_migration_number(&migrations_dir)?;
    let description_snake = description.replace(' ', "_").replace('-', "_").to_lowercase();
    let migration_filename = format!("{:03}_alter_{}_{}.sql", migration_number, table_name, description_snake);
    let migration_path = migrations_dir.join(&migration_filename);

    let migration_content = generate_alter_sql(&table_name, entity, &new_fields);
    fs::create_dir_all(&migrations_dir)?;
    fs::write(&migration_path, migration_content)?;

    println!("✅ {} migration: {}",
        "Created".bright_green(),
        migration_path.display().to_string().bright_cyan()
    );

    show_alter_next_steps(&migration_path);
    Ok(())
}

/// Diff entity fields vs existing migration
async fn diff_entity_migration(entity: &str, module: &str) -> Result<()> {
    println!("🔍 {} fields for {} in module {}...",
        "Comparing".bright_cyan(),
        entity.bright_cyan(),
        module.bright_yellow()
    );
    println!();

    let entity_snake = to_snake_case(entity);
    let table_name = to_plural(&entity_snake);

    // Parse current entity fields
    let entity_models_path = module_base_path(module)
        .join("src/infrastructure/persistence/postgresql/entity_models.rs");

    if !entity_models_path.exists() {
        return Err(anyhow::anyhow!(
            "Entity models file not found at: {}",
            entity_models_path.display()
        ));
    }

    let content = fs::read_to_string(&entity_models_path)?;
    let entities = parse_entities_with_fields(&content);
    let parsed_entity = entities.iter()
        .find(|e| e.name == entity)
        .ok_or_else(|| anyhow::anyhow!("Entity {} not found in entity_models.rs", entity))?;

    // Parse existing migration fields
    let migrations_dir = get_migrations_dir(module);
    let existing_fields = parse_existing_migration_fields(&migrations_dir, &table_name)?;

    // Get entity field names (excluding standard fields)
    let entity_field_names: Vec<_> = parsed_entity.fields.iter()
        .filter(|f| !matches!(f.name.as_str(), "id" | "created_at" | "updated_at" | "deleted_at"))
        .map(|f| f.name.as_str())
        .collect();

    // Find differences
    let new_fields: Vec<_> = entity_field_names.iter()
        .filter(|f| !existing_fields.contains(&f.to_string()))
        .collect();

    let removed_fields: Vec<_> = existing_fields.iter()
        .filter(|f| !entity_field_names.contains(&f.as_str()))
        .filter(|f| !matches!(f.as_str(), "id" | "created_at" | "updated_at" | "deleted_at"))
        .collect();

    // Report
    println!("📊 {} Comparison Results:", "Entity vs Migration".bright_white());
    println!();

    println!("   Entity fields ({}): {}",
        entity_field_names.len().to_string().bright_cyan(),
        entity_field_names.join(", ").bright_white()
    );
    println!();

    println!("   Migration fields ({}): {}",
        existing_fields.len().to_string().bright_cyan(),
        existing_fields.join(", ").bright_white()
    );
    println!();

    if new_fields.is_empty() && removed_fields.is_empty() {
        println!("   ✅ {} Entity and migration are in sync!", "Perfect!".bright_green());
    } else {
        if !new_fields.is_empty() {
            println!("   ➕ {} New fields to add: {}",
                "ADD".bright_green(),
                new_fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ").bright_green()
            );
        }
        if !removed_fields.is_empty() {
            println!("   ➖ {} Fields in migration but not in entity: {}",
                "REMOVE".bright_red(),
                removed_fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ").bright_red()
            );
        }
        println!();
        println!("   💡 To add new fields, run:");
        println!("      {} {} {} {} --description \"add_new_fields\"",
            "metaphor".cyan(),
            "migration".cyan(),
            "alter".cyan(),
            entity.bright_yellow()
        );
    }

    Ok(())
}

/// Parse existing migration to find field names
fn parse_existing_migration_fields(migrations_dir: &Path, table_name: &str) -> Result<Vec<String>> {
    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    // Find the CREATE TABLE migration for this table
    let entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.contains(&format!("create_{}_table", table_name)) && name.ends_with(".sql")
        })
        .collect();

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    // Read the migration file
    let migration_content = fs::read_to_string(entries[0].path())?;

    // Parse field names from CREATE TABLE statement
    let mut fields = Vec::new();
    let mut in_create_table = false;

    for line in migration_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("CREATE TABLE") {
            in_create_table = true;
            continue;
        }

        if in_create_table {
            if trimmed.starts_with(')') {
                break;
            }

            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Parse field: "field_name TYPE ..."
            if !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if !parts.is_empty() && !parts[0].starts_with("--") {
                    let field_name = parts[0].trim_end_matches(',');
                    fields.push(field_name.to_string());
                }
            }
        }
    }

    // Also check ALTER TABLE migrations for added fields
    let alter_entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.contains(&format!("alter_{}", table_name)) && name.ends_with(".sql")
        })
        .collect();

    for entry in alter_entries {
        let alter_content = fs::read_to_string(entry.path())?;
        for line in alter_content.lines() {
            let trimmed = line.trim().to_uppercase();
            if trimmed.starts_with("ADD COLUMN") || trimmed.contains("ADD COLUMN") {
                // Parse: ADD COLUMN field_name TYPE ...
                let lower_line = line.to_lowercase();
                if let Some(pos) = lower_line.find("add column") {
                    let after = &line[pos + 10..].trim();
                    let parts: Vec<&str> = after.split_whitespace().collect();
                    if !parts.is_empty() {
                        fields.push(parts[0].to_string());
                    }
                }
            }
        }
    }

    Ok(fields)
}

/// Generate ALTER TABLE SQL for new fields
fn generate_alter_sql(table_name: &str, entity: &str, new_fields: &[&EntityField]) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let mut alter_statements = Vec::new();
    let mut index_statements = Vec::new();
    let mut comment_statements = Vec::new();

    for field in new_fields {
        let sql_type = rust_type_to_sql(&field.rust_type, &field.name);
        let null_constraint = if field.is_optional { "" } else { " NOT NULL" };
        let default = if field.rust_type == "bool" {
            " DEFAULT false"
        } else if !field.is_optional {
            // For NOT NULL columns, we need a default value
            match sql_type {
                "VARCHAR(255)" => " DEFAULT ''",
                "TEXT" => " DEFAULT ''",
                "BIGINT" | "SMALLINT" => " DEFAULT 0",
                "DOUBLE PRECISION" => " DEFAULT 0.0",
                "BOOLEAN" => " DEFAULT false",
                "TIMESTAMPTZ" => " DEFAULT NOW()",
                "UUID" => " DEFAULT gen_random_uuid()",
                _ => "",
            }
        } else {
            ""
        };

        alter_statements.push(format!(
            "ALTER TABLE {} ADD COLUMN {} {}{}{};",
            table_name, field.name, sql_type, null_constraint, default
        ));

        // Add indexes for common lookup fields
        if field.name.ends_with("_id") || field.name == "email" || field.name == "username" ||
           field.name == "name" || field.name == "key" {
            index_statements.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({});",
                table_name, field.name, table_name, field.name
            ));
        }

        comment_statements.push(format!(
            "COMMENT ON COLUMN {}.{} IS '{} field';",
            table_name, field.name, field.name.replace('_', " ")
        ));
    }

    format!(r#"-- Migration: Alter {table_name} table
-- Entity: {entity}
-- Generated: {timestamp}
-- Adding {count} new field(s)
--
-- IMPORTANT: Review this migration before running!
-- If the table has existing data and new fields are NOT NULL,
-- you may need to adjust the DEFAULT values or make fields nullable.

-- ============================================================================
-- Add new columns
-- ============================================================================

{alter_statements}

-- ============================================================================
-- Add indexes for new columns
-- ============================================================================

{index_statements}

-- ============================================================================
-- Add comments for new columns
-- ============================================================================

{comment_statements}
"#,
        table_name = table_name,
        entity = entity,
        timestamp = timestamp,
        count = new_fields.len(),
        alter_statements = alter_statements.join("\n"),
        index_statements = if index_statements.is_empty() {
            "-- No indexes needed for new columns".to_string()
        } else {
            index_statements.join("\n")
        },
        comment_statements = comment_statements.join("\n"),
    )
}

/// Show next steps after generating alter migration
fn show_alter_next_steps(migration_path: &Path) {
    println!();
    println!("📋 {} Next steps:", "Generated!".bright_green());
    println!("   1. ⚠️  Review the migration file - check DEFAULT values!");
    println!("   2. Run the migration: {}", "sqlx migrate run".cyan());
    println!("   3. Or manually: {}", format!("psql -f {}", migration_path.display()).cyan());
}

/// Show next steps after generating migration
fn show_next_steps(migration_path: &Path) {
    println!();
    println!("📋 {} Next steps:", "Generated!".bright_green());
    println!("   1. Edit the migration file to add your entity-specific fields");
    println!("   2. Run the migration: {}", "sqlx migrate run".cyan());
    println!("   3. Or manually: {}", format!("psql -f {}", migration_path.display()).cyan());
}

// ============================================================================
// Migration Run (using sqlx)
// ============================================================================

/// Run pending migrations using sqlx
async fn run_migrations(module: &str, database_url: Option<&str>) -> Result<()> {
    println!("🗄️  {} migrations for module {}...",
        "Running".bright_green(),
        module.bright_yellow()
    );

    // Check both possible migration directories.
    // The schema generator (metaphor-plugin-schema) emits to top-level `migrations/`,
    // so that's preferred. `migrations/postgres/` is a legacy per-backend layout kept
    // as a fallback for projects that still use it.
    let module_base = module_base_path(module);
    let root_dir = module_base.join("migrations");
    let postgres_dir = module_base.join("migrations/postgres");

    // Helper to check if directory has top-level SQL files (non-recursive).
    let has_sql_files = |dir: &Path| -> bool {
        if !dir.exists() {
            return false;
        }
        fs::read_dir(dir)
            .map(|entries| entries.filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .any(|e| e.path().extension().map_or(false, |ext| ext == "sql")))
            .unwrap_or(false)
    };

    // Prefer `migrations/` (top-level) when it has SQL files, since that's where
    // the schema generator emits. Fall back to `migrations/postgres/` for legacy.
    let migrations_dir = if has_sql_files(&root_dir) {
        root_dir
    } else if has_sql_files(&postgres_dir) {
        postgres_dir
    } else {
        return Err(anyhow::anyhow!(
            "No SQL migration files found at:\n  - {}\n  - {}\nRun 'metaphor migration generate' first.",
            root_dir.display(),
            postgres_dir.display()
        ));
    };

    // Get database URL from parameter, environment, or app config
    let db_url = match database_url {
        Some(url) => url.to_string(),
        None => std::env::var("DATABASE_URL")
            .ok()
            .or_else(crate::utils::get_database_url)
            .ok_or_else(|| anyhow::anyhow!(
                "DATABASE_URL not set. Provide --database-url, set DATABASE_URL env var (or add it to .env), or configure database.url in config/application.yml"
            ))?,
    };

    println!("   📂 Migrations: {}", migrations_dir.display().to_string().bright_white());
    println!("   🔗 Database: {}...",
        sanitize_db_url(&db_url).bright_white()
    );
    println!();

    // Check if migrations use .up.sql suffix (non-sqlx compatible)
    // In that case, directly use psql instead of sqlx
    let has_up_suffix = fs::read_dir(&migrations_dir)?
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().ends_with(".up.sql"));

    if has_up_suffix {
        println!("   ℹ️  Detected .up.sql migration files, using psql execution...");
        return run_migrations_with_tracking(&migrations_dir, module, &db_url).await;
    }

    // Use sqlx-cli to run migrations
    let output = std::process::Command::new("sqlx")
        .args([
            "migrate",
            "run",
            "--source",
            migrations_dir.to_str().unwrap(),
            "--database-url",
            &db_url,
        ])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                println!("✅ {} Migrations applied successfully!", "Done!".bright_green());
                Ok(())
            } else {
                let _stderr = String::from_utf8_lossy(&result.stderr);
                // If sqlx fails, try manual approach
                println!("⚠️  sqlx failed, falling back to psql execution...");
                run_migrations_with_tracking(&migrations_dir, module, &db_url).await
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                // Fallback: run migrations manually with psql
                println!("⚠️  {} not found, using manual SQL execution...", "sqlx-cli".yellow());
                run_migrations_with_tracking(&migrations_dir, module, &db_url).await
            } else {
                Err(anyhow::anyhow!("Failed to run sqlx: {}", e))
            }
        }
    }
}

/// Legacy: run migrations manually using psql (without tracking)
/// DEPRECATED: Use run_migrations_with_tracking instead
#[allow(dead_code)]

/// Fallback: run migrations manually using psql
async fn run_migrations_manually(migrations_dir: &Path, database_url: &str) -> Result<()> {
    // Parse database URL to get connection parameters
    let url = url::Url::parse(database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let user = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    // Get sorted list of migration files (support both .sql and .up.sql)
    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Include .sql files but exclude down.sql and files in subdirectories like seeds/
            (name.ends_with(".sql") || name.ends_with(".up.sql"))
                && name != "down.sql"
                && !name.starts_with("seed")
                && e.path().is_file()
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("   (no migration files found)");
        return Ok(());
    }

    println!("   📋 Found {} migration files", entries.len());

    for entry in entries {
        let filename = entry.file_name();
        println!("   ⏳ Running {}...", filename.to_string_lossy().bright_white());

        let output = std::process::Command::new("psql")
            .env("PGPASSWORD", password)
            .args([
                "-h", host,
                "-p", &port.to_string(),
                "-U", user,
                "-d", database,
                "-f", entry.path().to_str().unwrap(),
                "-q",  // Quiet mode
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("   ❌ Failed: {}", stderr);
            return Err(anyhow::anyhow!("Migration {} failed", filename.to_string_lossy()));
        }

        println!("   ✅ {}", filename.to_string_lossy());
    }

    println!();
    println!("✅ {} All migrations applied!", "Done!".bright_green());
    Ok(())
}

// ============================================================================
// Schema Migrations Tracking (database-level tracking of applied migrations)
// ============================================================================

/// Database connection parameters parsed from a URL
struct DbConnectionParams {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

impl DbConnectionParams {
    /// Parse database URL into connection parameters
    fn from_url(database_url: &str) -> Result<Self> {
        let url = url::Url::parse(database_url)?;
        Ok(Self {
            host: url.host_str().unwrap_or("localhost").to_string(),
            port: url.port().unwrap_or(5432),
            user: url.username().to_string(),
            password: url.password().unwrap_or("").to_string(),
            database: url.path().trim_start_matches('/').to_string(),
        })
    }

    /// Execute a SQL query using psql and return stdout
    fn execute_query(&self, sql: &str) -> Result<String> {
        let output = std::process::Command::new("psql")
            .env("PGPASSWORD", &self.password)
            .args([
                "-h", &self.host,
                "-p", &self.port.to_string(),
                "-U", &self.user,
                "-d", &self.database,
                "-t",  // Tuples only (no headers/footers)
                "-A",  // Unaligned output
                "-c", sql,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Query failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Execute a SQL file using psql
    fn execute_file(&self, file_path: &Path) -> Result<()> {
        let output = std::process::Command::new("psql")
            .env("PGPASSWORD", &self.password)
            .args([
                "-h", &self.host,
                "-p", &self.port.to_string(),
                "-U", &self.user,
                "-d", &self.database,
                "-f", file_path.to_str().unwrap(),
                "-q",  // Quiet mode
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Migration failed: {}", stderr));
        }

        Ok(())
    }
}

/// Ensure the schema_migrations table exists
fn ensure_schema_migrations_table(db: &DbConnectionParams) -> Result<()> {
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            id SERIAL PRIMARY KEY,
            module VARCHAR(100) NOT NULL,
            name VARCHAR(255) NOT NULL,
            applied_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
            UNIQUE(module, name)
        );
        CREATE INDEX IF NOT EXISTS idx_schema_migrations_module ON schema_migrations(module);
    "#;

    db.execute_query(create_table_sql)?;
    Ok(())
}

/// Check if a migration has already been applied
fn is_migration_applied(db: &DbConnectionParams, module: &str, migration_name: &str) -> Result<bool> {
    let sql = format!(
        "SELECT COUNT(*) FROM schema_migrations WHERE module = '{}' AND name = '{}'",
        module.replace('\'', "''"),
        migration_name.replace('\'', "''")
    );

    let result = db.execute_query(&sql)?;
    let count: i32 = result.parse().unwrap_or(0);
    Ok(count > 0)
}

/// Record a migration as applied
fn record_migration(db: &DbConnectionParams, module: &str, migration_name: &str) -> Result<()> {
    let sql = format!(
        "INSERT INTO schema_migrations (module, name) VALUES ('{}', '{}') ON CONFLICT (module, name) DO NOTHING",
        module.replace('\'', "''"),
        migration_name.replace('\'', "''")
    );

    db.execute_query(&sql)?;
    Ok(())
}

/// Get the count of applied migrations for a module
fn get_applied_migration_count(db: &DbConnectionParams, module: &str) -> Result<i32> {
    let sql = format!(
        "SELECT COUNT(*) FROM schema_migrations WHERE module = '{}'",
        module.replace('\'', "''")
    );

    let result = db.execute_query(&sql)?;
    let count: i32 = result.parse().unwrap_or(0);
    Ok(count)
}

/// Run migrations manually with tracking via schema_migrations table
async fn run_migrations_with_tracking(
    migrations_dir: &Path,
    module: &str,
    database_url: &str,
) -> Result<()> {
    let db = DbConnectionParams::from_url(database_url)?;

    // Ensure schema_migrations table exists
    ensure_schema_migrations_table(&db)?;

    // Get sorted list of migration files
    let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            (name.ends_with(".sql") || name.ends_with(".up.sql"))
                && name != "down.sql"
                && !name.starts_with("seed")
                && e.path().is_file()
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("   (no migration files found)");
        return Ok(());
    }

    println!("   📋 Found {} migration files", entries.len());

    let mut applied_count = 0;
    let mut skipped_count = 0;

    for entry in entries {
        let filename = entry.file_name().to_string_lossy().to_string();

        // Check if already applied
        if is_migration_applied(&db, module, &filename)? {
            skipped_count += 1;
            continue;
        }

        println!("   ⏳ Running {}...", filename.bright_white());

        // Execute the migration
        db.execute_file(&entry.path())?;

        // Record it as applied
        record_migration(&db, module, &filename)?;

        println!("   ✅ {}", filename);
        applied_count += 1;
    }

    println!();
    if applied_count > 0 {
        println!("✅ {} Applied {} new migration(s), skipped {} already applied",
            "Done!".bright_green(),
            applied_count.to_string().bright_cyan(),
            skipped_count.to_string().bright_white()
        );
    } else {
        println!("✅ {} All {} migration(s) already applied",
            "Done!".bright_green(),
            skipped_count.to_string().bright_white()
        );
    }

    Ok(())
}

// ============================================================================
// Seeder Execution (using module seeder binary)
// ============================================================================

/// Run database seeders for a module
/// Supports both SQL seed files (from migrations/seeds/) and Rust seeders
async fn run_seeders(module: &str, name: Option<&str>, force: bool, database_url: Option<&str>) -> Result<()> {
    println!("🌱 {} seeders for module {}...",
        "Running".bright_green(),
        module.bright_yellow()
    );

    // Get database URL from parameter, environment, or app config
    let db_url = match database_url {
        Some(url) => url.to_string(),
        None => std::env::var("DATABASE_URL")
            .ok()
            .or_else(crate::utils::get_database_url)
            .ok_or_else(|| anyhow::anyhow!(
                "DATABASE_URL not set. Provide --database-url, set DATABASE_URL env var (or add it to .env), or configure database.url in config/application.yml"
            ))?,
    };

    let module_path = module_base_path(module);
    let seeds_dir = module_path.join("migrations/seeds");

    // Check for SQL seed files first (preferred for simplicity)
    if seeds_dir.exists() {
        println!("   📂 Found SQL seeds directory: {}", seeds_dir.display());
        return run_sql_seeds(&seeds_dir, name, &db_url).await;
    }

    // Fall back to Rust seeder binary
    let cargo_toml_path = module_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!(
            "Module {} not found at {}",
            module,
            module_path.display()
        ));
    }

    // Check for seeder binary
    let cargo_content = fs::read_to_string(&cargo_toml_path)?;
    let seeder_bin_name = format!("{}-seeder", module);

    if !cargo_content.contains(&format!("name = \"{}\"", seeder_bin_name)) {
        // Generate seeder binary if not exists
        println!("   ⚠️  Seeder binary not found, generating...");
        ensure_seeder_binary_exists(module, &module_path)?;
    }

    println!("   🔗 Database: {}...",
        sanitize_db_url(&db_url).bright_white()
    );
    println!();

    // Build and run the seeder binary
    let mut args = vec![];
    if force {
        args.push("--force");
    }
    if let Some(filter) = name {
        args.push(filter);
    }

    // Get package name from Cargo.toml
    let package_name = parse_package_name(&cargo_content)
        .ok_or_else(|| anyhow::anyhow!("Could not parse package name from Cargo.toml"))?;

    println!("   ⏳ Building and running seeder...");
    let output = std::process::Command::new("cargo")
        .env("DATABASE_URL", &db_url)
        .args([
            "run",
            "--package", &package_name,
            "--bin", &seeder_bin_name,
            "--",
        ])
        .args(&args)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        println!();
        println!("✅ {} Seeders completed successfully!", "Done!".bright_green());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        Err(anyhow::anyhow!("Seeder failed:\n{}", stderr))
    }
}

/// Run SQL seed files from seeds directory
async fn run_sql_seeds(seeds_dir: &Path, name: Option<&str>, database_url: &str) -> Result<()> {
    // Parse database URL to get connection parameters
    let url = url::Url::parse(database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let user = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    println!("   🔗 Database: {}...",
        sanitize_db_url(database_url).bright_white()
    );
    println!();

    // Get sorted list of seed files
    let mut entries: Vec<_> = fs::read_dir(seeds_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let file_name = e.file_name().to_string_lossy().to_string();
            file_name.ends_with(".sql") && e.path().is_file()
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("   (no seed files found)");
        return Ok(());
    }

    // Filter by name if specified
    if let Some(filter) = name {
        entries.retain(|e| {
            let file_name = e.file_name().to_string_lossy().to_string();
            file_name.contains(filter)
        });
    }

    println!("   📋 Running {} seed file(s)", entries.len());

    for entry in entries {
        let filename = entry.file_name();
        println!("   ⏳ Seeding {}...", filename.to_string_lossy().bright_white());

        let output = std::process::Command::new("psql")
            .env("PGPASSWORD", password)
            .args([
                "-h", host,
                "-p", &port.to_string(),
                "-U", user,
                "-d", database,
                "-f", entry.path().to_str().unwrap(),
                "-q",  // Quiet mode
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("   ❌ Failed: {}", stderr);
            return Err(anyhow::anyhow!("Seed {} failed", filename.to_string_lossy()));
        }

        println!("   ✅ {}", filename.to_string_lossy());
    }

    println!();
    println!("✅ {} All seeds applied!", "Done!".bright_green());
    Ok(())
}

/// Parse package name from Cargo.toml content
fn parse_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") && trimmed.contains('=') {
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let name = parts[1].trim().trim_matches('"');
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Ensure seeder binary exists in module
fn ensure_seeder_binary_exists(module: &str, module_path: &Path) -> Result<()> {
    let seeder_bin_path = module_path.join("src/bin/seeder.rs");
    let seeder_bin_name = format!("{}-seeder", module);

    // Create src/bin directory if needed
    if let Some(parent) = seeder_bin_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check if seeder.rs exists
    if !seeder_bin_path.exists() {
        // Generate seeder binary code
        let seeder_code = generate_seeder_binary_code(module);
        fs::write(&seeder_bin_path, seeder_code)?;
        println!("   ✅ Created {}", seeder_bin_path.display().to_string().bright_cyan());
    }

    // Update Cargo.toml to include binary
    let cargo_toml_path = module_path.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml_path)?;

    if !cargo_content.contains(&format!("name = \"{}\"", seeder_bin_name)) {
        // Add [[bin]] section
        let bin_section = format!(r#"

[[bin]]
name = "{}"
path = "src/bin/seeder.rs"
"#, seeder_bin_name);

        let updated_content = format!("{}{}", cargo_content.trim_end(), bin_section);
        fs::write(&cargo_toml_path, updated_content)?;
        println!("   ✅ Added seeder binary to Cargo.toml");
    }

    Ok(())
}

/// Generate seeder binary code for a module
fn generate_seeder_binary_code(module: &str) -> String {
    // Get package name (metaphor-{module})
    let package_name = format!("metaphor_{}", module);

    format!(r#"//! Database seeder binary for module: {module}
//!
//! Run with: cargo run --bin {module}-seeder
//! Or via CLI: metaphor migration seed --module {module}
//!
//! This binary runs all registered seeders in order.

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Import seeders from the module's seeders directory
// TODO: Update these imports based on your actual seeders
use {package_name}::seeders::Seeder;
// use {package_name}::seeders::SeedExampleSeeder;

#[tokio::main]
async fn main() -> Result<()> {{
    let args: Vec<String> = env::args().collect();
    let force = args.iter().any(|a| a == "--force");
    let filter: Option<&str> = args.iter()
        .skip(1)
        .find(|a| !a.starts_with("-"))
        .map(|s| s.as_str());

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("🔗 Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("✅ Connected to database");
    println!();
    println!("🌱 Running seeders for module: {module}");
    println!();

    // Register your seeders here in order
    let seeders: Vec<Box<dyn Seeder + Send + Sync>> = Vec::new();
    // Example:
    // seeders.push(Box::new(SeedExampleSeeder::new()));

    if seeders.is_empty() {{
        println!("   ⚠️  No seeders registered!");
        println!("   Add seeders in src/bin/seeder.rs");
        return Ok(());
    }}

    // Run seeders
    for seeder in &seeders {{
        let name = seeder.name();

        // Apply filter if specified
        if let Some(f) = filter {{
            if !name.contains(f) {{
                continue;
            }}
        }}

        if force {{
            println!("   ⏳ Running {{}}...", name);
            seeder.run(&pool).await?;
            println!("   ✅ {{}} completed", name);
        }} else {{
            println!("   ⏳ Checking {{}}...", name);
            if seeder.should_run(&pool).await? {{
                println!("   🔄 Running {{}}...", name);
                seeder.run(&pool).await?;
                println!("   ✅ {{}} completed", name);
            }} else {{
                println!("   ⏭️  {{}} already applied, skipping", name);
            }}
        }}
    }}

    println!();
    println!("🎉 All seeders completed successfully!");

    Ok(())
}}
"#, module = module, package_name = package_name)
}

// ============================================================================
// Generate SQL Seeds from Rust Seeders
// ============================================================================

/// Generate SQL seed files from Rust seeder source files
///
/// This command reads the Rust seeder source files and generates equivalent SQL
/// INSERT statements. This is useful for:
/// - Running seeds without compiling Rust code
/// - Exporting seeds to pure SQL for other tools
/// - Documentation purposes
async fn generate_sql_seeds(module: &str, force: bool) -> Result<()> {
    println!("📄 {} SQL seed files for module {}...",
        "Generating".bright_green(),
        module.bright_yellow()
    );

    let module_path = module_base_path(module);
    let seeders_dir = module_path.join("migrations/seeders");
    let seeds_dir = module_path.join("migrations/seeds");

    if !seeders_dir.exists() {
        return Err(anyhow::anyhow!(
            "Seeders directory not found at: {}\nCreate seeders first with 'metaphor make:seeder'",
            seeders_dir.display()
        ));
    }

    // Create seeds directory
    fs::create_dir_all(&seeds_dir)?;

    // Find all seeder files
    let seeder_files: Vec<_> = fs::read_dir(&seeders_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with("_seeder.rs") && name != "mod.rs"
        })
        .collect();

    if seeder_files.is_empty() {
        println!("   ⚠️  No seeder files found in {}", seeders_dir.display());
        return Ok(());
    }

    println!("   📂 Source: {}", seeders_dir.display().to_string().bright_white());
    println!("   📂 Output: {}", seeds_dir.display().to_string().bright_white());
    println!();

    for entry in seeder_files {
        let filename = entry.file_name();
        let name = filename.to_string_lossy().to_string();
        let seeder_name = name.trim_end_matches("_seeder.rs");

        let sql_filename = format!("{}.sql", seeder_name);
        let sql_path = seeds_dir.join(&sql_filename);

        if sql_path.exists() && !force {
            println!("   ⏭️  {} (already exists)", sql_filename.bright_white());
            continue;
        }

        // Parse the seeder file and generate SQL
        let content = fs::read_to_string(entry.path())?;
        let sql_content = parse_seeder_to_sql(&content, seeder_name);

        fs::write(&sql_path, &sql_content)?;
        println!("   ✅ {}", sql_filename.bright_cyan());
    }

    println!();
    println!("🎉 {} SQL seed files generated!", "Done!".bright_green());
    println!();
    println!("📋 {} To run SQL seeds:", "Next steps:".bright_white());
    println!("   psql -d your_database -f {}/seed_roles.sql", seeds_dir.display());
    println!("   Or use: metaphor migration seed --module {}", module);

    Ok(())
}

/// Parse Rust seeder source and generate SQL statements
fn parse_seeder_to_sql(content: &str, seeder_name: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    // Try to extract table name and data from the seeder
    let table_name = infer_table_from_seeder(content, seeder_name);
    let insert_data = extract_insert_data(content);

    let mut sql = format!(r#"-- SQL Seed: {seeder_name}
-- Generated from: {seeder_name}_seeder.rs
-- Generated: {timestamp}
--
-- This SQL file was auto-generated from the Rust seeder.
-- Review and customize as needed before running.

-- ============================================================================
-- Seed Data for: {table_name}
-- ============================================================================

"#, seeder_name = seeder_name, timestamp = timestamp, table_name = table_name);

    if insert_data.is_empty() {
        sql.push_str(&format!(r#"-- TODO: Could not automatically parse INSERT data from seeder.
-- Please manually create INSERT statements based on the seeder logic.
--
-- Example format:
-- INSERT INTO {table_name} (id, name, description, metadata)
-- VALUES
--   (gen_random_uuid(), 'example', 'Example description', '{{"created_at": "2024-01-01"}}');
"#, table_name = table_name));
    } else {
        sql.push_str(&insert_data);
    }

    sql.push_str(&format!(r#"
-- ============================================================================
-- Verification
-- ============================================================================

-- SELECT COUNT(*) as count FROM {table_name};
"#, table_name = table_name));

    sql
}

/// Infer table name from seeder content
fn infer_table_from_seeder(content: &str, seeder_name: &str) -> String {
    // Look for INSERT INTO table_name patterns
    for line in content.lines() {
        if line.contains("INSERT INTO") {
            if let Some(start) = line.find("INSERT INTO") {
                let after = &line[start + 12..];
                let parts: Vec<&str> = after.trim().split(|c: char| c.is_whitespace() || c == '(').collect();
                if !parts.is_empty() {
                    return parts[0].to_string();
                }
            }
        }
    }

    // Fallback: derive from seeder name (seed_users -> users)
    if seeder_name.starts_with("seed_") {
        seeder_name[5..].to_string()
    } else {
        seeder_name.to_string()
    }
}

/// Extract INSERT data patterns from seeder source
fn extract_insert_data(content: &str) -> String {
    let mut inserts = Vec::new();

    // Look for const arrays with definition data
    let mut in_const_array = false;
    let mut in_struct_literal = false;
    let mut current_table = String::new();
    let mut items = Vec::new();
    let mut current_struct = String::new();
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect const arrays like: const DEFAULT_ROLES: &[RoleDefinition] = &[
        if (trimmed.starts_with("const DEFAULT_") || trimmed.starts_with("const ")) && trimmed.contains(": &[") {
            in_const_array = true;
            items.clear();

            // Try to extract table name from const name
            if let Some(start) = trimmed.find("DEFAULT_") {
                let after = &trimmed[start + 8..];
                if let Some(end) = after.find(':') {
                    current_table = after[..end].to_lowercase();
                }
            }
            continue;
        }

        if in_const_array {
            // End of array
            if trimmed == "];" {
                in_const_array = false;

                if !items.is_empty() && !current_table.is_empty() {
                    // Generate SQL for this table
                    let sql = generate_sql_from_items(&current_table, &items);
                    if !sql.is_empty() {
                        inserts.push(sql);
                    }
                }
                items.clear();
                continue;
            }

            // Track struct literal that may span multiple lines
            // Look for patterns like: StructName {
            if trimmed.ends_with('{') && !in_struct_literal {
                in_struct_literal = true;
                current_struct.clear();
                current_struct.push_str(trimmed);
                current_struct.push(' ');
                brace_depth = 1;
                continue;
            }

            if in_struct_literal {
                // Track braces
                for c in trimmed.chars() {
                    match c {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }

                current_struct.push_str(trimmed);
                current_struct.push(' ');

                // End of struct literal
                if brace_depth == 0 {
                    in_struct_literal = false;
                    items.push(current_struct.trim().to_string());
                    current_struct.clear();
                }
                continue;
            }

            // Single-line struct literals: { name: "admin", description: "..." },
            if trimmed.contains('{') && trimmed.contains('}') {
                items.push(trimmed.to_string());
            }
        }
    }

    inserts.join("\n\n")
}

/// Generate SQL INSERT from parsed items
///
/// Takes a table name and a list of struct literal strings, parses them,
/// and generates a SQL INSERT statement with proper column definitions.
fn generate_sql_from_items(table: &str, items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let values = generate_values_for_table(table, items);

    if values.is_empty() {
        return String::new();
    }

    let columns = get_columns_for_table(table);

    format!(r#"INSERT INTO {} {}
VALUES
{}
ON CONFLICT DO NOTHING;
"#, table, columns, values)
}

/// Get SQL column definitions for a specific table
fn get_columns_for_table(table: &str) -> &'static str {
    match table {
        "roles" => "(id, name, description, is_default, metadata)",
        "permissions" => "(id, name, description, resource, action, metadata)",
        "users" => "(id, email, username, password_hash, email_verified, metadata)",
        _ => "(id, name, description, metadata)",
    }
}

/// Generate SQL VALUES clause for a specific table from parsed items
fn generate_values_for_table(table: &str, items: &[String]) -> String {
    let mut values = Vec::new();

    for item in items {
        let fields = parse_struct_literal(item);
        if fields.is_empty() {
            continue;
        }

        let value_tuple = match table {
            "roles" => generate_role_values(&fields),
            "users" => generate_user_values(&fields),
            "permissions" => generate_permission_values(&fields),
            _ => generate_generic_values(&fields),
        };

        if !value_tuple.is_empty() {
            values.push(format!("  ({})", value_tuple));
        }
    }

    values.join(",\n")
}

/// Generate values for roles table
fn generate_role_values(fields: &[(String, String)]) -> String {
    let name = fields.iter().find(|(k, _)| k == "name").map(|(_, v)| v.as_str()).unwrap_or("");
    let desc = fields.iter().find(|(k, _)| k == "description").map(|(_, v)| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return String::new();
    }

    format!(
        "gen_random_uuid(), '{}', '{}', false, '{{\"created_at\": \"now\", \"updated_at\": \"now\"}}'::jsonb",
        name.replace('\'', "''"),
        desc.replace('\'', "''")
    )
}

/// Generate values for users table
fn generate_user_values(fields: &[(String, String)]) -> String {
    let email = fields.iter().find(|(k, _)| k == "email").map(|(_, v)| v.as_str()).unwrap_or("");
    let username = fields.iter().find(|(k, _)| k == "username").map(|(_, v)| v.as_str()).unwrap_or("");
    let first_name = fields.iter().find(|(k, _)| k == "first_name").map(|(_, v)| v.as_str()).unwrap_or("");
    let last_name = fields.iter().find(|(k, _)| k == "last_name").map(|(_, v)| v.as_str()).unwrap_or("");

    if email.is_empty() {
        return String::new();
    }

    // Default password hash (Argon2id hash of "password")
    let default_hash = "$argon2id$v=19$m=19456,t=2,p=1$PLACEHOLDER_SALT$PLACEHOLDER_HASH";

    format!(
        "gen_random_uuid(), '{}', '{}', '{}', true, '{{\"created_at\": \"now\", \"updated_at\": \"now\", \"first_name\": \"{}\", \"last_name\": \"{}\"}}'::jsonb",
        email.replace('\'', "''"),
        username.replace('\'', "''"),
        default_hash,
        first_name.replace('\'', "''"),
        last_name.replace('\'', "''")
    )
}

/// Generate values for permissions table
fn generate_permission_values(fields: &[(String, String)]) -> String {
    let identifier = fields.iter().find(|(k, _)| k == "identifier").map(|(_, v)| v.as_str()).unwrap_or("");
    let desc = fields.iter().find(|(k, _)| k == "description").map(|(_, v)| v.as_str()).unwrap_or("");

    if identifier.is_empty() {
        return String::new();
    }

    // Parse action:resource from identifier
    let parts: Vec<&str> = identifier.split(':').collect();
    let (action, resource) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        (identifier, "*")
    };

    format!(
        "gen_random_uuid(), '{}', '{}', '{}', '{}', '{{\"created_at\": \"now\", \"updated_at\": \"now\"}}'::jsonb",
        identifier.replace('\'', "''"),
        desc.replace('\'', "''"),
        resource.replace('\'', "''"),
        action.replace('\'', "''")
    )
}

/// Generate generic values
fn generate_generic_values(fields: &[(String, String)]) -> String {
    let name = fields.iter().find(|(k, _)| k == "name").map(|(_, v)| v.as_str()).unwrap_or("");
    let desc = fields.iter().find(|(k, _)| k == "description").map(|(_, v)| v.as_str()).unwrap_or("");

    if name.is_empty() {
        return String::new();
    }

    format!(
        "gen_random_uuid(), '{}', '{}', '{{\"created_at\": \"now\", \"updated_at\": \"now\"}}'::jsonb",
        name.replace('\'', "''"),
        desc.replace('\'', "''")
    )
}

/// Parse a Rust struct literal into key-value pairs
///
/// Parses struct literals in the format `{ key: "value", other: 123 }`
/// and returns a vector of (key, value) pairs.
fn parse_struct_literal(item: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();

    // Find content between { }
    let start = item.find('{');
    let end = item.rfind('}');

    let (Some(s), Some(e)) = (start, end) else { return fields };

    // Guard against malformed input
    if s >= e || s + 1 >= item.len() {
        return fields;
    }

    let content = &item[s + 1..e];

    // Parse field: value pairs
    for part in content.split(',') {
        let part = part.trim();
        if let Some(colon) = part.find(':') {
            // Bounds check before indexing
            if colon + 1 >= part.len() {
                continue;
            }
            let key = part[..colon].trim().to_string();
            let value = part[colon + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            // Skip empty keys
            if !key.is_empty() {
                fields.push((key, value));
            }
        }
    }

    fields
}

// ============================================================================
// Run All Migrations (Multi-Module)
// ============================================================================

/// Locate `metaphor.yaml` by walking up from cwd. Returns the workspace root
/// (directory containing the manifest) and the parsed `projects:` list as
/// `(name, path)` pairs. Paths are absolute, resolved against the workspace root.
fn read_workspace_projects() -> Option<(std::path::PathBuf, Vec<(String, std::path::PathBuf)>)> {
    let start = std::env::current_dir().ok()?;
    let mut dir: &Path = &start;
    let manifest_path = loop {
        let candidate = dir.join("metaphor.yaml");
        if candidate.is_file() {
            break candidate;
        }
        dir = dir.parent()?;
    };
    let workspace_root = manifest_path.parent()?.to_path_buf();

    let content = fs::read_to_string(&manifest_path).ok()?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    let projects = yaml.get("projects")?.as_sequence()?;

    let mut out = Vec::new();
    for p in projects {
        let name = p.get("name").and_then(|v| v.as_str())?.to_string();
        let path = p.get("path").and_then(|v| v.as_str())?;
        out.push((name, workspace_root.join(path)));
    }
    Some((workspace_root, out))
}

/// Resolve the base directory for `module`. Prefers a project entry in
/// `metaphor.yaml` whose `name` matches; falls back to `libs/modules/<module>`
/// (legacy layout) so pre-workspace callers keep working.
pub(crate) fn module_base_path(module: &str) -> std::path::PathBuf {
    if let Some((_, projects)) = read_workspace_projects() {
        if let Some((_, path)) = projects.into_iter().find(|(name, _)| name == module) {
            return path;
        }
    }
    Path::new("libs/modules").join(module)
}

/// Discover all modules that have a `migrations/` directory.
///
/// Strategy:
/// 1. If `metaphor.yaml` is present, iterate its `projects:` and return any
///    whose resolved path contains `migrations/` or `migrations/postgres/`.
///    This is the modern path — source of truth is the workspace manifest.
/// 2. Otherwise fall back to scanning `libs/modules/` and filtering against
///    `config/application.yml` `modules:` section (legacy behavior).
pub(crate) fn discover_modules() -> Result<Vec<String>> {
    if let Some((_, projects)) = read_workspace_projects() {
        let mut modules: Vec<String> = projects
            .into_iter()
            .filter(|(_, path)| {
                path.join("migrations/postgres").exists() || path.join("migrations").exists()
            })
            .map(|(name, _)| name)
            .collect();
        modules.sort();
        return Ok(modules);
    }

    let modules_dir = Path::new("libs/modules");
    if !modules_dir.exists() {
        return Ok(Vec::new());
    }

    // Legacy layout: scan libs/modules/ and filter by enabled modules from app config
    let enabled_modules = get_enabled_modules_from_app_config();

    let mut modules = Vec::new();
    for entry in fs::read_dir(modules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let module_name = entry.file_name().to_string_lossy().to_string();

            if !enabled_modules.contains(&module_name) {
                continue;
            }

            let migrations_postgres = path.join("migrations/postgres");
            let migrations_dir = path.join("migrations");

            if migrations_postgres.exists() || migrations_dir.exists() {
                modules.push(module_name);
            }
        }
    }

    modules.sort();
    Ok(modules)
}

/// Get list of enabled modules from app config
pub(crate) fn get_enabled_modules_from_app_config() -> Vec<String> {
    let mut enabled_modules = Vec::new();

    // Try to read the app config file
    let app_config_paths = [
        "apps/metaphor/config/application.yml",
        "config/application.yml", // Fallback for different project structure
    ];

    for config_path in &app_config_paths {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                // Parse the modules section
                if let Some(modules) = yaml.get("modules") {
                    if let Some(mapping) = modules.as_mapping() {
                        for (module_name, module_config) in mapping {
                            // Check if module is enabled
                            if let Some(config) = module_config.as_mapping() {
                                if let Some(enabled) = config.get("enabled") {
                                    if enabled.as_bool().unwrap_or(false) {
                                        enabled_modules.push(module_name.as_str().unwrap_or("").to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    enabled_modules.sort();
    enabled_modules
}

/// Run pending migrations for ALL discovered modules
///
/// Reads the database.url from application.yml and expands environment variables.
///
/// # Configuration Search Order
/// 1. Command-line `--database-url` argument
/// 2. `.env` file in project root (loads DATABASE_URL, POSTGRES_DB, etc.)
/// 3. `config/application.yml`
/// 4. `apps/metaphor/config/application.yml` (legacy fallback)
///
/// # Environment Variable Expansion
/// Supports the format `${VAR:default}` where:
/// - `VAR` is the environment variable name
/// - `default` is the optional fallback value if VAR is not set
pub async fn run_all_migrations(database_url: Option<&str>) -> Result<()> {
    println!("🚀 {} Running migrations for ALL modules...", "Starting".bright_cyan().bold());
    println!();

    // Get database URL from multiple sources with proper priority
    let db_url = database_url
        .map(|s| s.to_string())
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .or_else(crate::utils::get_database_url)
        .ok_or_else(|| anyhow::anyhow!(
            "DATABASE_URL not set. Provide --database-url, set DATABASE_URL env var (or add it to .env), or configure database.url in config/application.yml"
        ))?;

    println!("   🔗 Database: {}...",
        sanitize_db_url(&db_url).bright_white()
    );
    println!();

    // Discover all modules
    let modules = discover_modules()?;

    if modules.is_empty() {
        println!("   ⚠️  No projects with migrations found (checked metaphor.yaml projects and libs/modules/)");
        return Ok(());
    }

    println!("📋 Found {} modules with migrations:", modules.len().to_string().bright_cyan());
    for module in &modules {
        println!("   • {}", module.bright_yellow());
    }
    println!();

    // Run migrations for each module in order
    let mut success_count = 0;
    let mut error_count = 0;

    for module in &modules {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 Module: {}", module.bright_yellow().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match run_migrations(module, Some(&db_url)).await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                eprintln!("   ❌ Error: {}", e);
                error_count += 1;
            }
        }
        println!();
    }

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 {} Summary:", "Migration".bright_white().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   ✅ Successful: {}", success_count.to_string().bright_green());
    if error_count > 0 {
        println!("   ❌ Failed: {}", error_count.to_string().bright_red());
    }
    println!();

    if error_count > 0 {
        Err(anyhow::anyhow!("{} module(s) failed to migrate", error_count))
    } else {
        println!("🎉 {} All migrations completed successfully!", "Done!".bright_green().bold());
        Ok(())
    }
}

/// Show migration status for all modules
async fn show_migration_status(module_filter: Option<&str>, database_url: Option<&str>) -> Result<()> {
    println!("📊 {} Migration Status", "Checking".bright_cyan());
    println!();

    // Discover modules
    let modules = discover_modules()?;

    if modules.is_empty() {
        println!("   ⚠️  No modules with migrations found");
        return Ok(());
    }

    // Filter if specified
    let modules_to_check: Vec<_> = if let Some(filter) = module_filter {
        modules.into_iter().filter(|m| m == filter).collect()
    } else {
        modules
    };

    if modules_to_check.is_empty() {
        println!("   ⚠️  Module '{}' not found", module_filter.unwrap_or(""));
        return Ok(());
    }

    // Try to get database connection for applied migration counts
    let db = match database_url {
        Some(url) => DbConnectionParams::from_url(url).ok(),
        None => std::env::var("DATABASE_URL")
            .ok()
            .and_then(|url| DbConnectionParams::from_url(&url).ok()),
    };

    // Ensure schema_migrations table exists if we have a connection
    if let Some(ref db_conn) = db {
        let _ = ensure_schema_migrations_table(db_conn);
    }

    // Print header
    println!("┌─────────────────────────────────────────────────────────────────────┐");
    println!("│ {:20} │ {:10} │ {:10} │ {:20} │", "Module", "Total", "Applied", "Last Migration");
    println!("├─────────────────────────────────────────────────────────────────────┤");

    for module in &modules_to_check {
        // Count migration files (excluding down.sql and seed files)
        let migrations_dir = get_migrations_dir(module);
        let is_migration_file = |name: &str| -> bool {
            // Include numbered .sql or .up.sql files, exclude down.sql and seed files
            name.ends_with(".sql")
                && !name.starts_with("seed")
                && name != "down.sql"
                && !name.ends_with(".down.sql")
        };

        let total_files = if migrations_dir.exists() {
            fs::read_dir(&migrations_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| is_migration_file(&e.file_name().to_string_lossy()))
                .count()
        } else {
            0
        };

        // Get last migration file name
        let last_migration = if migrations_dir.exists() {
            let mut entries: Vec<_> = fs::read_dir(&migrations_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| is_migration_file(&e.file_name().to_string_lossy()))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            entries.last()
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.len() > 18 {
                        format!("{}...", &name[..15])
                    } else {
                        name
                    }
                })
                .unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };

        // Query database for applied count if we have a connection
        let applied = match &db {
            Some(db_conn) => {
                match get_applied_migration_count(db_conn, module) {
                    Ok(count) => count.to_string(),
                    Err(_) => "?".to_string(),
                }
            }
            None => "?".to_string(),
        };

        // Calculate pending count
        let pending_indicator = if applied != "?" {
            let applied_count: usize = applied.parse().unwrap_or(0);
            if total_files > applied_count {
                format!(" ({})", (total_files - applied_count).to_string().yellow())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        println!("│ {:20} │ {:10} │ {:10}{:8} │ {:20} │",
            module.bright_yellow(),
            total_files.to_string().bright_cyan(),
            applied.bright_green(),
            pending_indicator,
            last_migration.bright_white()
        );
    }

    println!("└─────────────────────────────────────────────────────────────────────┘");
    println!();

    // Show connection status
    if db.is_none() {
        println!("⚠️  {} Set DATABASE_URL or use --database-url to see applied counts",
            "Note:".bright_yellow()
        );
        println!();
    }

    println!("📋 {} To run all migrations:", "Next:".bright_white());
    println!("   metaphor migration run-all");
    println!();
    println!("📋 {} To run single module:", "Or:".bright_white());
    println!("   metaphor migration run --module <module_name>");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("UserRole"), "user_role");
        assert_eq!(to_snake_case("MfaDevice"), "mfa_device");
        assert_eq!(to_snake_case("PasswordResetToken"), "password_reset_token");
    }

    #[test]
    fn test_to_plural() {
        assert_eq!(to_plural("user"), "users");
        assert_eq!(to_plural("role"), "roles");
        assert_eq!(to_plural("session"), "sessions");
        assert_eq!(to_plural("category"), "categories");
        assert_eq!(to_plural("box"), "boxes");
        assert_eq!(to_plural("bush"), "bushes");
    }

    #[test]
    fn test_expand_env_vars_with_default() {
        std::env::set_var("TEST_EXPAND_WITH_DEFAULT", "value");
        let result = expand_env_vars("${TEST_EXPAND_WITH_DEFAULT:default}");
        assert_eq!(result, "value");
        std::env::remove_var("TEST_EXPAND_WITH_DEFAULT");
    }

    #[test]
    fn test_expand_env_vars_default_used() {
        std::env::remove_var("TEST_EXPAND_DEFAULT");
        let result = expand_env_vars("${TEST_EXPAND_DEFAULT:default}");
        assert_eq!(result, "default");
    }

    #[test]
    fn test_expand_env_vars_no_default() {
        std::env::remove_var("TEST_EXPAND_NO_DEFAULT");
        let result = expand_env_vars("${TEST_EXPAND_NO_DEFAULT}");
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_env_vars_multiple() {
        std::env::set_var("TEST_HOST", "localhost");
        std::env::set_var("TEST_PORT", "5432");
        let result = expand_env_vars("${TEST_HOST:127.0.0.1}:${TEST_PORT:3306}");
        assert_eq!(result, "localhost:5432");
        std::env::remove_var("TEST_HOST");
        std::env::remove_var("TEST_PORT");
    }

    #[test]
    fn test_sanitize_db_url_with_password() {
        let url = "postgresql://root:secret@localhost:5432/bersihirdb";
        let result = sanitize_db_url(url);
        // URL parsing doesn't include port when it's the default
        assert_eq!(result, "postgresql://root:***@localhost/bersihirdb");
    }

    #[test]
    fn test_sanitize_db_url_without_password() {
        let url = "postgresql://localhost:5432/bersihirdb";
        let result = sanitize_db_url(url);
        // URL parsing doesn't include port when it's the default
        assert_eq!(result, "postgresql://localhost/bersihirdb");
    }

    #[test]
    fn test_sanitize_db_url_long_url() {
        let url = "postgresql://user:very-long-password@very-long-hostname.example.com:5432/database";
        let result = sanitize_db_url(url);
        // Should redact password even for long URLs
        assert!(result.contains("user:***@"));
        assert!(!result.contains("very-long-password"));
        assert!(result.contains("very-long-hostname.example.com"));
    }

    #[test]
    fn test_try_extract_entity_name_valid() {
        let line = "pub struct UserEntity {";
        assert_eq!(try_extract_entity_name(line), Some("User".to_string()));
    }

    #[test]
    fn test_try_extract_entity_name_multiple_words() {
        let line = "pub struct PasswordResetTokenEntity {";
        assert_eq!(try_extract_entity_name(line), Some("PasswordResetToken".to_string()));
    }

    #[test]
    fn test_try_extract_entity_name_invalid() {
        assert_eq!(try_extract_entity_name("pub struct Something {"), None);
        assert_eq!(try_extract_entity_name("pub struct UserEntity"), None);
        assert_eq!(try_extract_entity_name("let x = 5;"), None);
    }

    #[test]
    fn test_update_brace_depth() {
        assert_eq!(update_brace_depth("{", 0), 1);
        assert_eq!(update_brace_depth("}", 1), 0);
        assert_eq!(update_brace_depth("{ { }", 1), 2);
        assert_eq!(update_brace_depth("normal text", 5), 5);
    }

    #[test]
    fn test_parse_struct_literal_bounds_safety() {
        // Edge case: colon at end of string - no value after colon
        let result = parse_struct_literal("{key:}");
        // Empty values are not added (key is present but value is empty)
        assert!(result.is_empty());

        // Edge case: empty struct
        let result = parse_struct_literal("{}");
        assert!(result.is_empty());

        // Edge case: valid field with quoted empty string
        let result = parse_struct_literal(r#"{key: ""}"#);
        assert_eq!(result, vec![("key".to_string(), "".to_string())]);

        // Edge case: valid struct with multiple fields
        let result = parse_struct_literal(r#"{name: "test", count: 42}"#);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "name");
        assert_eq!(result[0].1, "test");
        assert_eq!(result[1].0, "count");
        assert_eq!(result[1].1, "42");
    }

    #[test]
    fn test_get_columns_for_table() {
        assert_eq!(get_columns_for_table("roles"), "(id, name, description, is_default, metadata)");
        assert_eq!(get_columns_for_table("users"), "(id, email, username, password_hash, email_verified, metadata)");
        assert_eq!(get_columns_for_table("permissions"), "(id, name, description, resource, action, metadata)");
        assert_eq!(get_columns_for_table("unknown"), "(id, name, description, metadata)");
    }
}
