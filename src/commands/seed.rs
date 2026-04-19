//! Database seeding commands
//!
//! Provides CLI commands for managing database seeds:
//! - `metaphor seed create <name>` - Create a new seed file
//! - `metaphor seed run [name]` - Run seeds (all or specific)
//! - `metaphor seed revert` - Revert applied seeds
//! - `metaphor seed status` - Show seed status
//! - `metaphor seed history` - Show seed execution history

use anyhow::Result;
use clap::Subcommand;
use colored::*;
use std::fs;
use std::path::Path;
use chrono::Utc;

// Import centralized database config from utils
use crate::utils::get_database_url;

/// Seed format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedFormat {
    Sql,
    Rust,
}

impl std::str::FromStr for SeedFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sql" => Ok(SeedFormat::Sql),
            "rust" | "rs" => Ok(SeedFormat::Rust),
            _ => Err(format!("Invalid format '{}'. Use 'sql' or 'rust'", s)),
        }
    }
}

/// Seed action enum
#[derive(Debug, Clone, Subcommand)]
pub enum SeedAction {
    /// Create a new seed file
    Create {
        /// Seed name (e.g., "initial_users", "test_data")
        name: String,

        /// Seed type: data (production), test (development), reference (lookup tables)
        #[arg(long, short, default_value = "data")]
        r#type: String,

        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds, e.g., "metaphor", "api")
        #[arg(long, short, default_value = "metaphor")]
        app: String,

        /// Seed format: sql (raw SQL) or rust (programmatic seeder class)
        #[arg(long, short, default_value = "sql")]
        format: SeedFormat,
    },

    /// Run database seeds
    Run {
        /// Specific seed to run (runs all if not specified)
        name: Option<String>,

        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds)
        #[arg(long, short, default_value = "metaphor")]
        app: String,

        /// Force re-run even if already applied
        #[arg(long)]
        force: bool,

        /// Seed format: sql (default) or rust (run Rust seeders)
        #[arg(long, short, default_value = "rust")]
        format: SeedFormat,
    },

    /// Revert applied seeds
    Revert {
        /// Specific seed to revert (reverts all if not specified)
        name: Option<String>,

        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds)
        #[arg(long, short, default_value = "metaphor")]
        app: String,
    },

    /// Show seed status
    Status {
        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds)
        #[arg(long, short, default_value = "metaphor")]
        app: String,
    },

    /// Show seed execution history
    History {
        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds)
        #[arg(long, short, default_value = "metaphor")]
        app: String,

        /// Number of entries to show
        #[arg(long, short, default_value = "20")]
        limit: usize,
    },

    /// List all available seeds
    List {
        /// Target module (for module-level seeds)
        #[arg(long, short)]
        module: Option<String>,

        /// Target app (for app-level seeds)
        #[arg(long, short, default_value = "metaphor")]
        app: String,
    },

    /// Run seeds for ALL registered modules
    ///
    /// Discovers all enabled modules from app config and runs their seeds in order.
    /// This is the recommended way to seed the entire system.
    ///
    /// Examples:
    ///   metaphor seed run-all
    ///   metaphor seed run-all --format sql
    ///   metaphor seed run-all --force
    RunAll {
        /// Force re-run even if already applied
        #[arg(long)]
        force: bool,

        /// Seed format: sql (default) or rust (run Rust seeders)
        #[arg(long, short, default_value = "sql")]
        format: SeedFormat,
    },
}

/// Handle seed commands
pub async fn handle_command(action: &SeedAction) -> Result<()> {
    match action {
        SeedAction::Create { name, r#type, module, app, format } => {
            create_seed(name, r#type, module.as_deref(), app, *format).await
        }
        SeedAction::Run { name, module, app, force, format } => {
            match format {
                SeedFormat::Sql => run_seeds(name.as_deref(), module.as_deref(), app, *force).await,
                SeedFormat::Rust => run_rust_seeders(name.as_deref(), module.as_deref(), app, *force).await,
            }
        }
        SeedAction::Revert { name, module, app } => {
            revert_seeds(name.as_deref(), module.as_deref(), app).await
        }
        SeedAction::Status { module, app } => {
            show_status(module.as_deref(), app).await
        }
        SeedAction::History { module, app, limit } => {
            show_history(module.as_deref(), app, *limit).await
        }
        SeedAction::List { module, app } => {
            list_seeds(module.as_deref(), app).await
        }
        SeedAction::RunAll { force, format } => {
            run_all_seeds(*force, *format).await
        }
    }
}

/// Get seeds directory path
fn get_seeds_dir(module: Option<&str>, app: &str) -> std::path::PathBuf {
    match module {
        // Resolve via workspace manifest (metaphor.yaml) when available so seeds
        // are discovered at the real module path, not the `libs/modules/` default.
        Some(m) => super::migration::module_base_path(m).join("migrations/seeds"),
        None => Path::new("apps").join(app).join("migrations/seeds"),
    }
}

/// Create a new seed file
async fn create_seed(name: &str, seed_type: &str, module: Option<&str>, app: &str, format: SeedFormat) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    let format_str = match format {
        SeedFormat::Sql => "SQL",
        SeedFormat::Rust => "Rust",
    };

    println!("🌱 {} {} seed '{}' ({}) for {}...",
        "Creating".bright_green(),
        format_str.bright_magenta(),
        name.bright_cyan(),
        seed_type.bright_yellow(),
        location
    );

    let seeds_dir = get_seeds_dir(module, app);
    fs::create_dir_all(&seeds_dir)?;

    match format {
        SeedFormat::Sql => create_sql_seed(name, seed_type, &seeds_dir)?,
        SeedFormat::Rust => create_rust_seed(name, seed_type, module, &seeds_dir)?,
    }

    Ok(())
}

/// Create SQL seed files
fn create_sql_seed(name: &str, seed_type: &str, seeds_dir: &Path) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let seed_name = format!("{}_{}.sql", timestamp, name);
    let revert_name = format!("{}_{}_revert.sql", timestamp, name);

    let seed_path = seeds_dir.join(&seed_name);
    let revert_path = seeds_dir.join(&revert_name);

    if seed_path.exists() {
        return Err(anyhow::anyhow!("Seed '{}' already exists at {}", name, seed_path.display()));
    }

    let seed_content = match seed_type.to_lowercase().as_str() {
        "test" => generate_test_seed_template(name),
        "reference" | "ref" => generate_reference_seed_template(name),
        _ => generate_data_seed_template(name),
    };

    let revert_content = generate_revert_template(name, seed_type);

    fs::write(&seed_path, seed_content)?;
    fs::write(&revert_path, revert_content)?;

    println!("✅ {} seed file: {}",
        "Created".bright_green(),
        seed_path.display().to_string().bright_cyan()
    );
    println!("✅ {} revert file: {}",
        "Created".bright_green(),
        revert_path.display().to_string().bright_cyan()
    );

    show_sql_next_steps(&seed_path);
    Ok(())
}

/// Create Rust seeder files
fn create_rust_seed(name: &str, seed_type: &str, module: Option<&str>, seeds_dir: &Path) -> Result<()> {
    // For Rust seeds, we create in a 'seeders' subdirectory
    let seeders_dir = seeds_dir.parent().unwrap_or(seeds_dir).join("seeders");
    fs::create_dir_all(&seeders_dir)?;

    let snake_name = to_snake_case(name);
    let seeder_file = format!("{}_seeder.rs", snake_name);
    let seeder_path = seeders_dir.join(&seeder_file);

    if seeder_path.exists() {
        return Err(anyhow::anyhow!("Seeder '{}' already exists at {}", name, seeder_path.display()));
    }

    let seeder_content = generate_rust_seeder_template(name, seed_type, module);
    fs::write(&seeder_path, &seeder_content)?;

    // Update or create mod.rs
    let mod_path = seeders_dir.join("mod.rs");
    update_seeders_mod(&mod_path, &snake_name)?;

    println!("✅ {} seeder file: {}",
        "Created".bright_green(),
        seeder_path.display().to_string().bright_cyan()
    );
    println!("✅ {} mod.rs: {}",
        "Updated".bright_green(),
        mod_path.display().to_string().bright_cyan()
    );

    show_rust_next_steps(&seeder_path, name);
    Ok(())
}

/// Update seeders mod.rs file
fn update_seeders_mod(mod_path: &Path, seeder_name: &str) -> Result<()> {
    let module_line = format!("pub mod {}_seeder;", seeder_name);
    let use_line = format!("pub use {}_seeder::*;", seeder_name);

    if mod_path.exists() {
        let content = fs::read_to_string(mod_path)?;
        if !content.contains(&module_line) {
            let new_content = format!("{}\n{}\n{}", content.trim(), module_line, use_line);
            fs::write(mod_path, new_content)?;
        }
    } else {
        let content = format!(
            "//! Database seeders\n//!\n//! Programmatic seeders using metaphor-orm.\n\n{}\n{}\n",
            module_line, use_line
        );
        fs::write(mod_path, content)?;
    }

    Ok(())
}

/// Convert to snake_case
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

/// Convert to PascalCase
fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// Run database seeds
async fn run_seeds(name: Option<&str>, module: Option<&str>, app: &str, force: bool) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    match name {
        Some(n) => {
            println!("🌱 {} seed '{}' for {}...",
                "Running".bright_green(),
                n.bright_cyan(),
                location
            );
        }
        None => {
            println!("🌱 {} all seeds for {}...",
                "Running".bright_green(),
                location
            );
        }
    }

    let seeds_dir = get_seeds_dir(module, app);

    if !seeds_dir.exists() {
        println!("📂 No seeds directory found at {}", seeds_dir.display().to_string().bright_yellow());
        println!("   Create seeds with: {} {} {}",
            "metaphor".cyan(),
            "seed".cyan(),
            "create <name>".cyan()
        );
        return Ok(());
    }

    // Load seed files
    let seeds = load_seed_files(&seeds_dir)?;

    if seeds.is_empty() {
        println!("📋 No seed files found in {}", seeds_dir.display());
        return Ok(());
    }

    // Filter by name if specified
    let seeds_to_run: Vec<_> = if let Some(seed_name) = name {
        seeds.into_iter()
            .filter(|s| s.name.contains(seed_name))
            .collect()
    } else {
        seeds
    };

    if seeds_to_run.is_empty() {
        println!("⚠️  No matching seeds found{}",
            name.map(|n| format!(" for '{}'", n)).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("📋 Seeds to run ({}):", seeds_to_run.len().to_string().bright_cyan());
    for seed in &seeds_to_run {
        println!("   📄 {} [{}]", seed.name.bright_white(), seed.seed_type.bright_yellow());
    }
    println!();

    if force {
        println!("⚠️  {} flag set - will re-run even if already applied", "--force".bright_yellow());
    }

    // Try to get database URL from:
    // 1. DATABASE_URL environment variable
    // 2. App config files (same logic as migration command)
    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .or_else(|| get_database_url());

    if let Some(url) = database_url {
        println!("🔗 {} Database connection found", "Connecting:".bright_green());

        // Execute seeds using psql
        for seed in &seeds_to_run {
            let seed_path = seeds_dir.join(format!("{}.sql", seed.name));
            println!("   ⏳ Running {}...", seed.name.bright_cyan());

            let output = std::process::Command::new("psql")
                .arg(&url)
                .arg("-f")
                .arg(&seed_path)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("   ✅ {} completed", seed.name.bright_green());
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        // Check if it's just warnings/notices vs actual errors
                        if stderr.contains("ERROR:") {
                            println!("   ❌ {} failed: {}", seed.name.bright_red(), stderr.trim());
                        } else {
                            println!("   ✅ {} completed (with notices)", seed.name.bright_green());
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ {} failed to execute: {}", seed.name.bright_red(), e);
                }
            }
        }

        println!();
        println!("🎉 {} Seeding completed!", "Done:".bright_green());
    } else {
        // No DATABASE_URL - show instructions
        println!("💡 {} To actually run seeds against database:", "Note:".bright_yellow());
        println!("   1. Set DATABASE_URL environment variable");
        println!("   2. Seeds will be executed in order by timestamp");
        println!();
        show_run_instructions(&seeds_dir, &seeds_to_run);
    }

    Ok(())
}

/// Run seeds for ALL registered modules
///
/// Discovers all enabled modules from app config and runs their seeds in order.
/// Supports both SQL and Rust seed formats.
async fn run_all_seeds(force: bool, format: SeedFormat) -> Result<()> {
    let format_label = match format {
        SeedFormat::Sql => "SQL",
        SeedFormat::Rust => "Rust",
    };

    println!("🌱 {} Running {} seeds for ALL modules...", "Starting".bright_cyan().bold(), format_label);
    println!();

    // Discover all enabled modules
    let modules = super::migration::discover_modules()?;

    if modules.is_empty() {
        println!("   ⚠️  No modules found in libs/modules/");
        return Ok(());
    }

    // Filter to modules that have seeds
    let modules_with_seeds: Vec<_> = modules.iter().filter(|m| {
        let seeds_dir = get_seeds_dir(Some(m.as_str()), "metaphor");
        let seeders_dir = get_seeders_dir(Some(m.as_str()), "metaphor");
        match format {
            SeedFormat::Sql => seeds_dir.exists(),
            SeedFormat::Rust => seeders_dir.exists(),
        }
    }).collect();

    if modules_with_seeds.is_empty() {
        println!("   ⚠️  No modules with {} seeds found", format_label);
        return Ok(());
    }

    println!("📋 Found {} modules with {} seeds:", modules_with_seeds.len().to_string().bright_cyan(), format_label);
    for module in &modules_with_seeds {
        println!("   • {}", module.bright_yellow());
    }
    println!();

    let mut success_count = 0;
    let mut error_count = 0;

    for module in &modules_with_seeds {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 Module: {}", module.bright_yellow().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let result = match format {
            SeedFormat::Sql => run_seeds(None, Some(module.as_str()), "metaphor", force).await,
            SeedFormat::Rust => run_rust_seeders(None, Some(module.as_str()), "metaphor", force).await,
        };

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!("   ❌ Error: {}", e);
                error_count += 1;
            }
        }
        println!();
    }

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 {} Summary:", "Seeding".bright_white().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   ✅ Successful: {}", success_count.to_string().bright_green());
    if error_count > 0 {
        println!("   ❌ Failed: {}", error_count.to_string().bright_red());
    }
    println!();

    if error_count > 0 {
        Err(anyhow::anyhow!("{} module(s) failed to seed", error_count))
    } else {
        println!("🎉 {} All seeds completed successfully!", "Done!".bright_green().bold());
        Ok(())
    }
}

/// Get seeders directory path (for Rust seeders)
fn get_seeders_dir(module: Option<&str>, app: &str) -> std::path::PathBuf {
    match module {
        Some(m) => super::migration::module_base_path(m).join("migrations/seeders"),
        None => Path::new("apps").join(app).join("migrations/seeders"),
    }
}

/// Load Rust seeder files from directory
fn load_rust_seeder_files(seeders_dir: &Path) -> Result<Vec<String>> {
    let mut seeders = Vec::new();

    if !seeders_dir.exists() {
        return Ok(seeders);
    }

    for entry in fs::read_dir(seeders_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip non-Rust files and mod.rs
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy();
        if filename == "mod.rs" {
            continue;
        }

        // Get seeder name (remove _seeder.rs suffix)
        let name = path.file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        if name.ends_with("_seeder") {
            seeders.push(name);
        }
    }

    // Sort by name to ensure consistent order
    seeders.sort();
    Ok(seeders)
}

/// Run Rust seeders by executing the module's seed binary
async fn run_rust_seeders(name: Option<&str>, module: Option<&str>, _app: &str, force: bool) -> Result<()> {
    let module_name = module.ok_or_else(|| anyhow::anyhow!("Module name is required for Rust seeders. Use --module <name>"))?;

    println!("🌱 {} Rust seeders for module {}...",
        "Running".bright_green(),
        module_name.bright_yellow()
    );

    let seeders_dir = get_seeders_dir(Some(module_name), "");

    if !seeders_dir.exists() {
        println!("📂 No seeders directory found at {}", seeders_dir.display().to_string().bright_yellow());
        println!("   Create seeders with: {} {} {} --format rust",
            "metaphor".cyan(),
            "seed".cyan(),
            "create <name>".cyan()
        );
        return Ok(());
    }

    // Load seeder files
    let seeders = load_rust_seeder_files(&seeders_dir)?;

    if seeders.is_empty() {
        println!("📋 No Rust seeder files found in {}", seeders_dir.display());
        return Ok(());
    }

    // Filter by name if specified
    let seeders_to_run: Vec<_> = if let Some(seed_name) = name {
        seeders.into_iter()
            .filter(|s| s.contains(seed_name))
            .collect()
    } else {
        seeders
    };

    if seeders_to_run.is_empty() {
        println!("⚠️  No matching seeders found{}",
            name.map(|n| format!(" for '{}'", n)).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("📋 Seeders to run ({}):", seeders_to_run.len().to_string().bright_cyan());
    for seeder in &seeders_to_run {
        println!("   🦀 {}", seeder.bright_white());
    }
    println!();

    if force {
        println!("⚠️  {} flag set - will re-run even if already applied", "--force".bright_yellow());
    }

    // Check for DATABASE_URL
    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        anyhow::anyhow!("DATABASE_URL environment variable is required to run Rust seeders")
    })?;

    println!("🔗 {} Database connection found", "Connecting:".bright_green());
    println!();

    // Ensure seeder binary exists and is built
    let module_path = Path::new("libs/modules").join(module_name);

    // First, ensure the seeders module is exposed in lib.rs and create the bin
    ensure_seeder_bin_exists(&module_path, module_name, &seeders_to_run, force)?;

    println!("🔨 {} and running seeder...", "Building".bright_cyan());
    println!();

    // Run using cargo with the seeder bin
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg(format!("{}-seeder", module_name))
        .arg("--package")
        .arg(format!("metaphor-{}", module_name))
        .env("DATABASE_URL", &database_url);

    // Add seeder names as args if filtering
    if let Some(seed_name) = name {
        cmd.arg("--").arg(seed_name);
    }

    if force {
        cmd.arg("--").arg("--force");
    }

    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    if output.status.success() {
        println!();
        println!("🎉 {} Seeding completed!", "Done:".bright_green());
    } else {
        if !stderr.is_empty() {
            eprintln!("{}", stderr.bright_red());
        }
        return Err(anyhow::anyhow!("Seeder execution failed"));
    }

    Ok(())
}

/// Ensure seeder binary exists in the module
fn ensure_seeder_bin_exists(module_path: &Path, module_name: &str, seeders: &[String], force: bool) -> Result<()> {
    let bin_path = module_path.join("src/bin");
    fs::create_dir_all(&bin_path)?;

    let seeder_bin_path = bin_path.join("seeder.rs");

    // Generate seeder binary code
    let seeder_code = generate_seeder_bin_code(module_name, seeders, force);

    // Always regenerate to ensure it's up to date
    fs::write(&seeder_bin_path, seeder_code)?;

    // Update Cargo.toml to include the binary
    let cargo_path = module_path.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_path)?;

    // Check if binary is already defined
    if !cargo_content.contains(&format!("name = \"{}-seeder\"", module_name)) {
        let bin_section = format!(r#"

[[bin]]
name = "{module}-seeder"
path = "src/bin/seeder.rs"
"#, module = module_name);

        let new_cargo_content = format!("{}{}", cargo_content.trim_end(), bin_section);
        fs::write(&cargo_path, new_cargo_content)?;
        println!("📦 Added seeder binary to {}/Cargo.toml", module_name);
    }

    // Ensure seeders are accessible from src
    ensure_seeders_in_src(module_path)?;

    Ok(())
}

/// Ensure seeders module is in src and exported from lib.rs
fn ensure_seeders_in_src(module_path: &Path) -> Result<()> {
    let src_seeders_path = module_path.join("src/seeders");
    let migrations_seeders_path = module_path.join("migrations/seeders");

    // If seeders are in migrations/, we need to add them to src/
    if migrations_seeders_path.exists() && !src_seeders_path.exists() {
        // Create symlink or copy
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let relative_path = Path::new("../migrations/seeders");
            if let Err(_) = symlink(relative_path, &src_seeders_path) {
                // If symlink fails, copy the files
                copy_dir_all(&migrations_seeders_path, &src_seeders_path)?;
            }
        }
        #[cfg(not(unix))]
        {
            copy_dir_all(&migrations_seeders_path, &src_seeders_path)?;
        }

        println!("📁 Linked seeders to src/seeders");
    }

    // Check if lib.rs exports seeders
    let lib_path = module_path.join("src/lib.rs");
    let lib_content = fs::read_to_string(&lib_path)?;

    if !lib_content.contains("pub mod seeders;") {
        // Add seeders module export
        let new_lib_content = format!("{}\n\n// Seeders module\npub mod seeders;\n", lib_content.trim_end());
        fs::write(&lib_path, new_lib_content)?;
        println!("📝 Added seeders module to lib.rs");
    }

    Ok(())
}

/// Copy directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Generate seeder binary code
fn generate_seeder_bin_code(module_name: &str, seeders: &[String], _force: bool) -> String {
    let mut seeder_imports = String::new();
    let mut seeder_registrations = String::new();

    for seeder in seeders {
        // Convert seeder file name to struct name: seed_roles_seeder -> SeedRolesSeeder
        let struct_name = seeder
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect::<String>();

        seeder_imports.push_str(&format!(
            "use metaphor_{module}::seeders::{struct_name};\n",
            module = module_name,
            struct_name = struct_name
        ));

        seeder_registrations.push_str(&format!(
            "    seeders.push(Box::new({struct_name}::new()));\n",
            struct_name = struct_name
        ));
    }

    // Add import for Seeder trait
    seeder_imports.push_str(&format!(
        "use metaphor_{module}::seeders::Seeder;\n",
        module = module_name
    ));

    format!(r#"//! Database seeder binary for module: {module}
//!
//! Run with: cargo run --bin {module}-seeder
//! Or via CLI: metaphor seed run --module {module}
//!
//! This binary runs all registered seeders in order.

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Import seeders
{imports}
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

    // Register seeders in order
    let mut seeders: Vec<Box<dyn Seeder + Send + Sync>> = Vec::new();
{registrations}
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
"#,
        module = module_name,
        imports = seeder_imports,
        registrations = seeder_registrations
    )
}

/// Revert applied seeds
async fn revert_seeds(name: Option<&str>, module: Option<&str>, app: &str) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    match name {
        Some(n) => {
            println!("🔄 {} seed '{}' for {}...",
                "Reverting".bright_yellow(),
                n.bright_cyan(),
                location
            );
        }
        None => {
            println!("🔄 {} all seeds for {}...",
                "Reverting".bright_yellow(),
                location
            );
        }
    }

    let seeds_dir = get_seeds_dir(module, app);

    if !seeds_dir.exists() {
        println!("📂 No seeds directory found at {}", seeds_dir.display());
        return Ok(());
    }

    // Load revert files
    let revert_files = load_revert_files(&seeds_dir)?;

    if revert_files.is_empty() {
        println!("📋 No revert files found in {}", seeds_dir.display());
        return Ok(());
    }

    // Filter by name if specified
    let files_to_revert: Vec<_> = if let Some(seed_name) = name {
        revert_files.into_iter()
            .filter(|s| s.contains(seed_name))
            .collect()
    } else {
        revert_files
    };

    if files_to_revert.is_empty() {
        println!("⚠️  No matching revert files found");
        return Ok(());
    }

    println!();
    println!("📋 Revert files available ({}):", files_to_revert.len().to_string().bright_cyan());
    for file in &files_to_revert {
        println!("   📄 {}", file.bright_white());
    }
    println!();

    println!("💡 {} To revert seeds:", "Note:".bright_yellow());
    println!("   1. Set DATABASE_URL environment variable");
    println!("   2. Run: psql $DATABASE_URL -f <revert_file>");

    Ok(())
}

/// Show seed status
async fn show_status(module: Option<&str>, app: &str) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    println!("📊 {} status for {}:",
        "Seed".bright_cyan(),
        location
    );
    println!();

    let seeds_dir = get_seeds_dir(module, app);

    if !seeds_dir.exists() {
        println!("   📂 Seeds directory: {} (not created)", seeds_dir.display().to_string().bright_yellow());
        println!("   📋 Total seeds: 0");
        println!();
        println!("   💡 Create your first seed with:");
        println!("      {} {} {} <name>", "metaphor".cyan(), "seed".cyan(), "create".cyan());
        return Ok(());
    }

    let seeds = load_seed_files(&seeds_dir)?;
    let revert_files = load_revert_files(&seeds_dir)?;

    println!("   📂 Seeds directory: {}", seeds_dir.display().to_string().bright_white());
    println!("   📋 Total seeds: {}", seeds.len().to_string().bright_cyan());
    println!("   🔄 Revert files: {}", revert_files.len().to_string().bright_cyan());
    println!();

    if !seeds.is_empty() {
        // Group by type
        let data_seeds: Vec<_> = seeds.iter().filter(|s| s.seed_type == "data").collect();
        let test_seeds: Vec<_> = seeds.iter().filter(|s| s.seed_type == "test").collect();
        let ref_seeds: Vec<_> = seeds.iter().filter(|s| s.seed_type == "reference").collect();

        println!("   📊 By type:");
        println!("      Data seeds: {}", data_seeds.len().to_string().bright_green());
        println!("      Test seeds: {}", test_seeds.len().to_string().bright_yellow());
        println!("      Reference seeds: {}", ref_seeds.len().to_string().bright_blue());
    }

    println!();
    println!("   💡 {} Database tracking requires DATABASE_URL to be set", "Note:".bright_yellow());
    println!("      Applied/pending status is tracked in schema_seeds table");

    Ok(())
}

/// Show seed execution history
async fn show_history(module: Option<&str>, app: &str, limit: usize) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    println!("📜 {} history for {} (last {}):",
        "Seed".bright_cyan(),
        location,
        limit.to_string().bright_yellow()
    );
    println!();

    println!("   💡 {} Seed history is tracked in the database", "Note:".bright_yellow());
    println!();
    println!("   To view history, run this SQL query:");
    println!("   {}", "SELECT id, name, seed_type, applied_at".cyan());
    println!("   {}", "FROM schema_seeds".cyan());
    println!("   {}", format!("ORDER BY applied_at DESC LIMIT {};", limit).cyan());
    println!();
    println!("   Or use psql:");
    println!("   {}", format!("psql $DATABASE_URL -c \"SELECT * FROM schema_seeds ORDER BY applied_at DESC LIMIT {}\"", limit).cyan());

    Ok(())
}

/// List all available seeds
async fn list_seeds(module: Option<&str>, app: &str) -> Result<()> {
    let location = module
        .map(|m| format!("module {}", m.bright_yellow()))
        .unwrap_or_else(|| format!("app {}", app.bright_yellow()));

    println!("📋 {} for {}:",
        "Available seeds".bright_cyan(),
        location
    );
    println!();

    let seeds_dir = get_seeds_dir(module, app);

    if !seeds_dir.exists() {
        println!("   (no seeds directory found)");
        println!();
        println!("   💡 Create your first seed with:");
        println!("      {} {} {} <name>", "metaphor".cyan(), "seed".cyan(), "create".cyan());
        return Ok(());
    }

    let seeds = load_seed_files(&seeds_dir)?;

    if seeds.is_empty() {
        println!("   (no seed files found)");
        return Ok(());
    }

    for seed in &seeds {
        let type_color = match seed.seed_type.as_str() {
            "test" => seed.seed_type.bright_yellow(),
            "reference" => seed.seed_type.bright_blue(),
            _ => seed.seed_type.bright_green(),
        };
        println!("   📄 {} [{}]", seed.name.bright_white(), type_color);
    }

    println!();
    println!("   Total: {} seed(s)", seeds.len().to_string().bright_cyan());

    Ok(())
}

/// Seed file representation
struct SeedFile {
    name: String,
    seed_type: String,
    #[allow(dead_code)]
    content: String,
}

/// Load seed files from directory with ordering support
///
/// If a `seed_order.yml` file exists in the seeds directory, seeds will be
/// executed in that order. Otherwise, they are sorted alphabetically.
fn load_seed_files(seeds_dir: &Path) -> Result<Vec<SeedFile>> {
    let mut seeds = Vec::new();

    if !seeds_dir.exists() {
        return Ok(seeds);
    }

    // Try to load seed order from seed_order.yml
    let seed_order_path = seeds_dir.join("seed_order.yml");
    let ordered_seed_names = if seed_order_path.exists() {
        load_seed_order(&seed_order_path)?
    } else {
        Vec::new()
    };

    for entry in fs::read_dir(seeds_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip non-SQL files and revert files
        if path.extension().and_then(|s| s.to_str()) != Some("sql") {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy();
        if filename.contains("_revert") || filename == "seed_order.yml" {
            continue;
        }

        let name = path.file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let content = fs::read_to_string(&path)?;
        let seed_type = determine_seed_type(&name, &content);

        seeds.push(SeedFile {
            name,
            seed_type,
            content,
        });
    }

    // Sort by order from seed_order.yml if available, otherwise alphabetically
    if !ordered_seed_names.is_empty() {
        sort_seeds_by_order(&mut seeds, &ordered_seed_names);
    } else {
        seeds.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok(seeds)
}

/// Load seed order from YAML file
fn load_seed_order(order_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(order_path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse seed_order.yml: {}", e))?;

    let mut order = Vec::new();
    if let Some(seq) = yaml.as_sequence() {
        for item in seq {
            if let Some(name) = item.as_str() {
                order.push(name.to_string());
            }
        }
    }

    Ok(order)
}

/// Sort seeds according to the order defined in seed_order.yml
///
/// Seeds that appear in the order list come first in that order.
/// Seeds not in the order list come after, sorted alphabetically.
fn sort_seeds_by_order(seeds: &mut Vec<SeedFile>, order: &[String]) {
    seeds.sort_by(|a, b| {
        let a_pos = order.iter().position(|x| x == &a.name);
        let b_pos = order.iter().position(|x| x == &b.name);

        match (a_pos, b_pos) {
            (Some(ai), Some(bi)) => ai.cmp(&bi),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });
}

/// Load revert files from directory
fn load_revert_files(seeds_dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    if !seeds_dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(seeds_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("sql") {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy();
        if filename.contains("_revert") {
            files.push(filename.to_string());
        }
    }

    files.sort();
    files.reverse(); // Most recent first for reverting
    Ok(files)
}

/// Determine seed type from name or content
fn determine_seed_type(name: &str, content: &str) -> String {
    let lower_name = name.to_lowercase();
    let lower_content = content.to_lowercase();

    if lower_name.contains("test") || lower_content.contains("-- type: test") {
        "test".to_string()
    } else if lower_name.contains("ref") || lower_content.contains("-- type: reference") {
        "reference".to_string()
    } else {
        "data".to_string()
    }
}

/// Generate Rust seeder template (Laravel-style)
fn generate_rust_seeder_template(name: &str, seed_type: &str, module: Option<&str>) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);

    let seed_type_enum = match seed_type.to_lowercase().as_str() {
        "test" => "SeedType::Test",
        "reference" | "ref" => "SeedType::Reference",
        _ => "SeedType::Data",
    };

    let module_use = module
        .map(|m| format!("use {}::domain::entity::*;\n", m))
        .unwrap_or_default();

    // Use include_str! style template with replacements to avoid format string complexity
    let template = r#"//! __PASCAL_NAME__ Seeder
//!
//! Created: __TIMESTAMP__
//! Type: __SEED_TYPE__
//!
//! This seeder creates __NAME__ data programmatically.
//! Implements the Seeder trait from metaphor-orm for database seeding.

use anyhow::Result;
use async_trait::async_trait;
use metaphor_orm::{PgPool, SeedType};
use chrono::Utc;
use uuid::Uuid;
__MODULE_USE__
/// __PASCAL_NAME__ seeder for programmatic data creation
pub struct __PASCAL_NAME__Seeder;

impl __PASCAL_NAME__Seeder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for __PASCAL_NAME__Seeder {
    fn default() -> Self {
        Self::new()
    }
}

/// Seeder trait for programmatic database seeding
#[async_trait]
pub trait Seeder {
    /// Seeder name for tracking
    fn name(&self) -> &'static str;

    /// Seeder type (Data, Test, Reference)
    fn seed_type(&self) -> SeedType;

    /// Run the seeder (insert data)
    async fn run(&self, pool: &PgPool) -> Result<()>;

    /// Revert the seeder (delete data)
    async fn revert(&self, pool: &PgPool) -> Result<()>;

    /// Check if seeder should run (optional condition)
    async fn should_run(&self, _pool: &PgPool) -> Result<bool> {
        Ok(true)
    }
}

#[async_trait]
impl Seeder for __PASCAL_NAME__Seeder {
    fn name(&self) -> &'static str {
        "__SNAKE_NAME__"
    }

    fn seed_type(&self) -> SeedType {
        __SEED_TYPE_ENUM__
    }

    async fn run(&self, pool: &PgPool) -> Result<()> {
        // ================================================================
        // Add your seeding logic here
        // ================================================================

        // Example: Create records using raw SQL
        // sqlx::query("INSERT INTO users (id, email, username, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
        //     .bind(Uuid::new_v4())
        //     .bind("admin@example.com")
        //     .bind("admin")
        //     .bind(Utc::now())
        //     .bind(Utc::now())
        //     .execute(pool)
        //     .await?;

        // Example: Create multiple records in a loop
        // let items = vec![
        //     ("Item 1", "description 1"),
        //     ("Item 2", "description 2"),
        //     ("Item 3", "description 3"),
        // ];
        //
        // for (name, description) in items {
        //     sqlx::query("INSERT INTO items (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
        //         .bind(Uuid::new_v4())
        //         .bind(name)
        //         .bind(description)
        //         .bind(Utc::now())
        //         .bind(Utc::now())
        //         .execute(pool)
        //         .await?;
        // }

        // Example: Use factory pattern for complex objects
        // let user = UserFactory::new()
        //     .with_email("test@example.com")
        //     .with_role("admin")
        //     .create(pool)
        //     .await?;

        println!("✅ __PASCAL_NAME__Seeder: Seeding completed");
        Ok(())
    }

    async fn revert(&self, pool: &PgPool) -> Result<()> {
        // ================================================================
        // Add your revert logic here
        // ================================================================

        // Example: Delete seeded records
        // sqlx::query("DELETE FROM users WHERE email = $1")
        //     .bind("admin@example.com")
        //     .execute(pool)
        //     .await?;

        // Example: Delete by pattern
        // sqlx::query("DELETE FROM items WHERE name LIKE 'Item %'")
        //     .execute(pool)
        //     .await?;

        println!("🔄 __PASCAL_NAME__Seeder: Revert completed");
        Ok(())
    }

    async fn should_run(&self, pool: &PgPool) -> Result<bool> {
        // Optional: Check if seeder should run
        // Example: Skip if data already exists
        // let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        //     .bind("admin@example.com")
        //     .fetch_one(pool)
        //     .await?;
        // Ok(count.0 == 0)

        Ok(true)
    }
}

// ============================================================================
// Usage in your application:
// ============================================================================
//
// use crate::seeders::__PASCAL_NAME__Seeder;
//
// async fn seed_database(pool: &PgPool) -> Result<()> {
//     let seeder = __PASCAL_NAME__Seeder::new();
//
//     if seeder.should_run(pool).await? {
//         seeder.run(pool).await?;
//     }
//
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeder_name() {
        let seeder = __PASCAL_NAME__Seeder::new();
        assert_eq!(seeder.name(), "__SNAKE_NAME__");
    }

    #[test]
    fn test_seeder_type() {
        let seeder = __PASCAL_NAME__Seeder::new();
        assert!(matches!(seeder.seed_type(), __SEED_TYPE_ENUM__));
    }
}
"#;

    template
        .replace("__PASCAL_NAME__", &pascal_name)
        .replace("__SNAKE_NAME__", &snake_name)
        .replace("__TIMESTAMP__", &timestamp.to_string())
        .replace("__SEED_TYPE__", seed_type)
        .replace("__SEED_TYPE_ENUM__", seed_type_enum)
        .replace("__MODULE_USE__", &module_use)
        .replace("__NAME__", name)
}

/// Generate data seed template
fn generate_data_seed_template(name: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(r#"-- Seed: {name}
-- Type: Data
-- Created: {timestamp}
-- Description: Initial data seed for production/staging
--
-- This seed contains data required for the application to function.
-- It will be tracked in the schema_seeds table.

-- ============================================================================
-- Insert your seed data below
-- ============================================================================

-- Example: Insert initial admin user
-- INSERT INTO users (id, email, username, role, created_at, updated_at) VALUES
-- (gen_random_uuid(), 'admin@example.com', 'admin', 'admin', NOW(), NOW());

-- Example: Insert system settings
-- INSERT INTO system_settings (key, value, description, created_at, updated_at) VALUES
-- ('app_name', 'My Application', 'Application display name', NOW(), NOW()),
-- ('maintenance_mode', 'false', 'Enable maintenance mode', NOW(), NOW());

-- Example: Insert default categories
-- INSERT INTO categories (id, name, slug, sort_order, created_at, updated_at) VALUES
-- (gen_random_uuid(), 'General', 'general', 1, NOW(), NOW()),
-- (gen_random_uuid(), 'Featured', 'featured', 2, NOW(), NOW());

-- ============================================================================
-- Add your data below this line
-- ============================================================================

"#,
        name = name,
        timestamp = timestamp,
    )
}

/// Generate test seed template
fn generate_test_seed_template(name: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(r#"-- Seed: {name}
-- Type: Test
-- Created: {timestamp}
-- Description: Test data for development and testing environments
--
-- ⚠️  WARNING: This seed contains test data - DO NOT run in production!
-- Test data uses identifiable patterns:
-- - Email domain: @test.local
-- - ID prefixes: test-*
-- - Usernames: test_*, demo_*

-- ============================================================================
-- Test Users
-- ============================================================================

-- INSERT INTO users (id, email, username, role, created_at, updated_at) VALUES
-- (gen_random_uuid(), 'user1@test.local', 'test_user_1', 'user', NOW(), NOW()),
-- (gen_random_uuid(), 'user2@test.local', 'test_user_2', 'user', NOW(), NOW()),
-- (gen_random_uuid(), 'admin@test.local', 'test_admin', 'admin', NOW(), NOW()),
-- (gen_random_uuid(), 'moderator@test.local', 'test_moderator', 'moderator', NOW(), NOW());

-- ============================================================================
-- Test Data
-- ============================================================================

-- INSERT INTO posts (id, title, content, user_id, status, created_at, updated_at) VALUES
-- (gen_random_uuid(), 'Test Post 1', 'This is test content 1', (SELECT id FROM users WHERE email = 'user1@test.local'), 'published', NOW(), NOW()),
-- (gen_random_uuid(), 'Test Post 2', 'This is test content 2', (SELECT id FROM users WHERE email = 'user1@test.local'), 'draft', NOW(), NOW()),
-- (gen_random_uuid(), 'Test Post 3', 'This is test content 3', (SELECT id FROM users WHERE email = 'user2@test.local'), 'published', NOW(), NOW());

-- ============================================================================
-- Add your test data below this line
-- ============================================================================

"#,
        name = name,
        timestamp = timestamp,
    )
}

/// Generate reference seed template
fn generate_reference_seed_template(name: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(r#"-- Seed: {name}
-- Type: Reference
-- Created: {timestamp}
-- Description: Reference data and lookup tables
--
-- Reference data is typically:
-- - Static and rarely changes
-- - Required for application functionality
-- - Used in foreign key relationships
-- - Loaded early in the application lifecycle

-- ============================================================================
-- Countries / Regions
-- ============================================================================

-- INSERT INTO countries (code, name, iso3, phone_code, currency, is_active) VALUES
-- ('US', 'United States', 'USA', '+1', 'USD', true),
-- ('CA', 'Canada', 'CAN', '+1', 'CAD', true),
-- ('GB', 'United Kingdom', 'GBR', '+44', 'GBP', true),
-- ('ID', 'Indonesia', 'IDN', '+62', 'IDR', true);

-- ============================================================================
-- Roles / Permissions
-- ============================================================================

-- INSERT INTO roles (id, name, description, permissions, created_at, updated_at) VALUES
-- (gen_random_uuid(), 'admin', 'System Administrator', '["*"]', NOW(), NOW()),
-- (gen_random_uuid(), 'moderator', 'Content Moderator', '["read", "update", "moderate"]', NOW(), NOW()),
-- (gen_random_uuid(), 'user', 'Standard User', '["read", "create_own", "update_own"]', NOW(), NOW()),
-- (gen_random_uuid(), 'guest', 'Guest User', '["read"]', NOW(), NOW());

-- ============================================================================
-- Status Values / Enums
-- ============================================================================

-- INSERT INTO status_types (code, name, description, color, sort_order) VALUES
-- ('active', 'Active', 'Item is active and visible', '#22c55e', 1),
-- ('inactive', 'Inactive', 'Item is inactive but preserved', '#f59e0b', 2),
-- ('archived', 'Archived', 'Item is archived for historical reference', '#6b7280', 3),
-- ('deleted', 'Deleted', 'Item is soft-deleted', '#ef4444', 4);

-- ============================================================================
-- Add your reference data below this line
-- ============================================================================

"#,
        name = name,
        timestamp = timestamp,
    )
}

/// Generate revert template
fn generate_revert_template(name: &str, seed_type: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    format!(r#"-- Revert Seed: {name}
-- Type: {seed_type}
-- Created: {timestamp}
--
-- This file reverts the changes made by the corresponding seed file.
-- Run this to undo the seed data.

-- ============================================================================
-- Add your revert SQL below
-- ============================================================================

-- Example: Delete seeded users
-- DELETE FROM users WHERE email LIKE '%@test.local';

-- Example: Delete seeded settings
-- DELETE FROM system_settings WHERE key IN ('app_name', 'maintenance_mode');

-- Example: Delete seeded categories
-- DELETE FROM categories WHERE slug IN ('general', 'featured');

-- ============================================================================
-- Add your revert statements below this line
-- ============================================================================

"#,
        name = name,
        seed_type = seed_type,
        timestamp = timestamp,
    )
}

/// Show next steps after creating SQL seed
fn show_sql_next_steps(seed_path: &Path) {
    println!();
    println!("📋 {} Next steps:", "Created!".bright_green());
    println!("   1. Edit the seed file to add your data:");
    println!("      {}", seed_path.display().to_string().cyan());
    println!("   2. Edit the revert file to add cleanup SQL");
    println!("   3. Run the seed:");
    println!("      {} {} {}", "metaphor".cyan(), "seed".cyan(), "run".cyan());
    println!("   4. Or run directly with psql:");
    println!("      {} $DATABASE_URL -f {}", "psql".cyan(), seed_path.display());
}

/// Show next steps after creating Rust seeder
fn show_rust_next_steps(seeder_path: &Path, name: &str) {
    let pascal_name = to_pascal_case(name);

    println!();
    println!("📋 {} Next steps:", "Created!".bright_green());
    println!();
    println!("   1. Edit the seeder to add your data logic:");
    println!("      {}", seeder_path.display().to_string().cyan());
    println!();
    println!("   2. Add the seeders module to your lib.rs:");
    println!("      {}", "pub mod seeders;".cyan());
    println!();
    println!("   3. Use the seeder in your application:");
    println!("      {}", format!("use crate::seeders::{}Seeder;", pascal_name).cyan());
    println!("      {}", format!("let seeder = {}Seeder::new();", pascal_name).cyan());
    println!("      {}", "seeder.run(&pool).await?;".cyan());
    println!();
    println!("   4. Or run all seeders with SeederRunner:");
    println!("      {}", "use crate::seeders::SeederRunner;".cyan());
    println!("      {}", "let runner = SeederRunner::new(&pool);".cyan());
    println!("      {}", "runner.run_all().await?;".cyan());
}

/// Show run instructions
fn show_run_instructions(seeds_dir: &Path, seeds: &[SeedFile]) {
    println!("📋 {} To run seeds manually:", "Instructions:".bright_green());
    println!();
    println!("   Option 1: Run all seeds in order:");
    for seed in seeds {
        let seed_path = seeds_dir.join(format!("{}.sql", seed.name));
        println!("   {} $DATABASE_URL -f {}", "psql".cyan(), seed_path.display());
    }
    println!();
    println!("   Option 2: Use metaphor-orm SeedManager in your app:");
    println!("   {}", "use metaphor_orm::seeding::{SeedManager, SeedType};".cyan());
    println!("   {}", "let manager = SeedManager::new(pool);".cyan());
    println!("   {}", "manager.seed().await?;".cyan());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_seed_type() {
        assert_eq!(determine_seed_type("test_users", ""), "test");
        assert_eq!(determine_seed_type("initial_data", "-- Type: test"), "test");
        assert_eq!(determine_seed_type("ref_countries", ""), "reference");
        assert_eq!(determine_seed_type("initial_users", ""), "data");
    }

    #[test]
    fn test_get_seeds_dir() {
        assert_eq!(get_seeds_dir(None, "metaphor"), Path::new("apps/metaphor/migrations/seeds"));
        assert_eq!(get_seeds_dir(None, "api"), Path::new("apps/api/migrations/seeds"));
        // Module path is resolved via module_base_path(); without a metaphor.yaml
        // in the test CWD it falls back to libs/modules/<name>.
        assert_eq!(get_seeds_dir(Some("sapiens"), "metaphor"), Path::new("libs/modules/sapiens/migrations/seeds"));
    }

    #[test]
    fn test_seed_format_from_str() {
        assert_eq!("sql".parse::<SeedFormat>().unwrap(), SeedFormat::Sql);
        assert_eq!("rust".parse::<SeedFormat>().unwrap(), SeedFormat::Rust);
        assert_eq!("rs".parse::<SeedFormat>().unwrap(), SeedFormat::Rust);
        assert!("invalid".parse::<SeedFormat>().is_err());
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("InitialUsers"), "initial_users");
        assert_eq!(to_snake_case("initial_users"), "initial_users");
        assert_eq!(to_snake_case("TestData"), "test_data");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("initial_users"), "InitialUsers");
        assert_eq!(to_pascal_case("test_data"), "TestData");
        assert_eq!(to_pascal_case("user"), "User");
    }
}
