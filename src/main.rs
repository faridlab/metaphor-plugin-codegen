//! Metaphor Codegen Plugin — scaffolding and code generation commands.
//!
//! Binary: `metaphor-codegen`
//! Commands: make, module, apps, proto, migration, seed

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

use metaphor_codegen::commands::{
    apps::AppsArgs,
    make::MakeAction,
    migration::MigrationAction,
    module::ModuleAction,
    proto::ProtoAction,
    routes::RoutesArgs,
    seed::SeedAction,
};

#[derive(Parser)]
#[command(
    name = "metaphor-codegen",
    version,
    about = "Code generation and scaffolding plugin for Metaphor CLI",
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Laravel-style scaffolding commands (make:*)
    #[command(subcommand)]
    Make(MakeAction),

    /// Module management commands (scaffolding)
    Module {
        #[command(subcommand)]
        action: CliModuleAction,
    },

    /// Application management commands (scaffolding)
    #[command(subcommand)]
    Apps(AppsArgs),

    /// Protocol buffer commands (buf/tonic operations)
    Proto {
        #[command(subcommand)]
        action: ProtoAction,
    },

    /// Database migration commands
    Migration {
        #[command(subcommand)]
        action: CliMigrationAction,
    },

    /// Database seeding commands
    Seed {
        #[command(subcommand)]
        action: SeedAction,
    },

    /// List HTTP routes defined in the project
    Routes(RoutesArgs),
}

// Module action needs CLI-level clap definitions that map to the handler's ModuleAction
#[derive(Subcommand)]
enum CliModuleAction {
    /// Create a new module with standard structure
    Create {
        /// Module name (e.g., "analytics", "payments")
        name: String,
        /// Author name
        #[arg(long, default_value = "Metaphor Developer")]
        author: String,
        /// Module description
        #[arg(long)]
        description: Option<String>,
    },
    /// List all available modules
    List,
    /// Show detailed information about a module
    Info {
        /// Module name
        name: String,
    },
    /// Enable a module in configuration
    Enable {
        /// Module name
        name: String,
    },
    /// Disable a module in configuration
    Disable {
        /// Module name
        name: String,
    },
    /// Install an external module package
    Install {
        /// Package name
        package: String,
        /// Add to production dependencies
        #[arg(long)]
        production: bool,
        /// Version specification
        #[arg(long)]
        version: Option<String>,
        /// Install from git repository
        #[arg(long)]
        git: bool,
    },
}

impl From<&CliModuleAction> for ModuleAction {
    fn from(cli_action: &CliModuleAction) -> Self {
        match cli_action {
            CliModuleAction::Create { name, author, description } => ModuleAction::Create {
                name: name.clone(),
                author: author.clone(),
                description: description.clone(),
            },
            CliModuleAction::List => ModuleAction::List,
            CliModuleAction::Info { name } => ModuleAction::Info { name: name.clone() },
            CliModuleAction::Enable { name } => ModuleAction::Enable { name: name.clone() },
            CliModuleAction::Disable { name } => ModuleAction::Disable { name: name.clone() },
            CliModuleAction::Install { package, production, version, git } => ModuleAction::Install {
                package: package.clone(),
                production: *production,
                version: version.clone(),
                git: *git,
            },
        }
    }
}

// Migration CLI-level definitions
#[derive(Subcommand)]
enum CliMigrationAction {
    /// Generate PostgreSQL migration for an entity
    Generate {
        entity: String,
        module: String,
        #[arg(long)]
        force: bool,
    },
    /// Generate migrations for all entities in a module
    GenerateAll {
        module: String,
        #[arg(long)]
        force: bool,
    },
    /// List existing migrations for a module
    List { module: String },
    /// Generate ALTER TABLE migration to add new fields
    Alter {
        entity: String,
        module: String,
        #[arg(long, short)]
        description: String,
    },
    /// Compare entity fields with existing migration
    Diff { entity: String, module: String },
    /// Run pending migrations using sqlx
    Run {
        #[arg(long, short)]
        module: String,
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Run pending migrations for ALL registered modules
    RunAll {
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Show migration status for all modules
    Status {
        #[arg(long, short)]
        module: Option<String>,
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Run database seeders for a module
    Seed {
        #[arg(long, short)]
        module: String,
        #[arg(long, short)]
        name: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        database_url: Option<String>,
    },
    /// Generate SQL seed files from Rust seeders
    GenerateSeeds {
        #[arg(long, short)]
        module: String,
        #[arg(long)]
        force: bool,
    },
}

impl From<&CliMigrationAction> for MigrationAction {
    fn from(cli_action: &CliMigrationAction) -> Self {
        match cli_action {
            CliMigrationAction::Generate { entity, module, force } => MigrationAction::Generate {
                entity: entity.clone(), module: module.clone(), force: *force,
            },
            CliMigrationAction::GenerateAll { module, force } => MigrationAction::GenerateAll {
                module: module.clone(), force: *force,
            },
            CliMigrationAction::List { module } => MigrationAction::List { module: module.clone() },
            CliMigrationAction::Alter { entity, module, description } => MigrationAction::Alter {
                entity: entity.clone(), module: module.clone(), description: description.clone(),
            },
            CliMigrationAction::Diff { entity, module } => MigrationAction::Diff {
                entity: entity.clone(), module: module.clone(),
            },
            CliMigrationAction::Run { module, database_url } => MigrationAction::Run {
                module: module.clone(), database_url: database_url.clone(),
            },
            CliMigrationAction::RunAll { database_url } => MigrationAction::RunAll {
                database_url: database_url.clone(),
            },
            CliMigrationAction::Status { module, database_url } => MigrationAction::Status {
                module: module.clone(), database_url: database_url.clone(),
            },
            CliMigrationAction::Seed { module, name, force, database_url } => MigrationAction::Seed {
                module: module.clone(), name: name.clone(), force: *force, database_url: database_url.clone(),
            },
            CliMigrationAction::GenerateSeeds { module, force } => MigrationAction::GenerateSeeds {
                module: module.clone(), force: *force,
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from CWD (and walk up to find one) so users don't have to
    // `source .env` before every run. Silent if no .env is found.
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    println!("{}", "⚡ Metaphor Codegen".bright_green().bold());
    println!();

    match &cli.command {
        Command::Make(action) => metaphor_codegen::commands::make::handle_command(action).await,
        Command::Module { action } => {
            let module_action = ModuleAction::from(action);
            metaphor_codegen::commands::module::handle_command(&module_action).await
        }
        Command::Apps(args) => metaphor_codegen::commands::apps::handle_command(args.clone()).await,
        Command::Proto { action } => metaphor_codegen::commands::proto::handle_command(action).await,
        Command::Migration { action } => {
            let migration_action = MigrationAction::from(action);
            metaphor_codegen::commands::migration::handle_command(&migration_action).await
        }
        Command::Seed { action } => metaphor_codegen::commands::seed::handle_command(action).await,
        Command::Routes(args) => metaphor_codegen::commands::routes::handle_command(args).await,
    }
}
