// {{PascalCaseEntity}} CLI Commands
// Command-line interface for {{PascalCaseEntity}} CRUD operations

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

use crate::application::{
    commands::{
        Create{{PascalCaseEntity}}Command, Update{{PascalCaseEntity}}Command, Delete{{PascalCaseEntity}}Command,
        BulkCreate{{PascalCaseEntity}}Command, Upsert{{PascalCaseEntity}}Command, Restore{{PascalCaseEntity}}Command,
        EmptyTrashCommand, {{PascalCaseEntity}}Dto, {{PascalCaseEntity}}Filters,
    },
    queries::{
        Get{{PascalCaseEntity}}Query, List{{PascalCaseEntity}}Query, ListDeleted{{PascalCaseEntity}}Query,
    },
    {{entity_name_snake}}_services::{{PascalCaseEntity}}ApplicationServices,
};

/// {{PascalCaseEntity}} management commands
#[derive(Parser)]
#[command(name = "{{entity_name_snake}}")]
#[command(about = "Manage {{entity_name_plural}} in the system")]
pub struct {{PascalCaseEntity}}Commands {
    #[command(subcommand)]
    pub action: {{PascalCaseEntity}}Action,
}

#[derive(Subcommand)]
pub enum {{PascalCaseEntity}}Action {
    /// Create a new {{entity_name_snake}}
    Create {
        /// {{PascalCaseEntity}} data in JSON format
        #[arg(short, long)]
        data: Option<String>,
        /// {{PascalCaseEntity}} data file path
        #[arg(short, long)]
        file: Option<String>,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// Get a {{entity_name_snake}} by ID
    Get {
        /// {{PascalCaseEntity}} ID
        id: String,
    },
    /// List {{entity_name_plural}} with optional filtering
    List {
        /// Page number (default: 1)
        #[arg(short, long, default_value = "1")]
        page: usize,
        /// Page size (default: 20)
        #[arg(short, long, default_value = "20")]
        page_size: usize,
        /// Sort field
        #[arg(short, long)]
        sort_by: Option<String>,
        /// Sort direction (asc|desc)
        #[arg(long, default_value = "asc")]
        sort_direction: String,
        /// Search term
        #[arg(short, long)]
        search: Option<String>,
        /// Filter JSON
        #[arg(long)]
        filters: Option<String>,
    },
    /// Update an existing {{entity_name_snake}}
    Update {
        /// {{PascalCaseEntity}} ID
        id: String,
        /// {{PascalCaseEntity}} data in JSON format
        #[arg(short, long)]
        data: Option<String>,
        /// {{PascalCaseEntity}} data file path
        #[arg(short, long)]
        file: Option<String>,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// Delete a {{entity_name_snake}} (soft delete)
    Delete {
        /// {{PascalCaseEntity}} ID
        id: String,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// Bulk create {{entity_name_plural}} from file or stdin
    BulkCreate {
        /// Input file path (JSON array)
        #[arg(short, long)]
        file: Option<String>,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// Upsert {{entity_name_plural}} (update or insert)
    Upsert {
        /// Input file path (JSON array)
        #[arg(short, long)]
        file: Option<String>,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// List deleted {{entity_name_plural}} (trash)
    ListTrash {
        /// Page number (default: 1)
        #[arg(short, long, default_value = "1")]
        page: usize,
        /// Page size (default: 20)
        #[arg(short, long, default_value = "20")]
        page_size: usize,
    },
    /// Restore a deleted {{entity_name_snake}}
    Restore {
        /// {{PascalCaseEntity}} ID
        id: String,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
    /// Empty trash (permanently delete all {{entity_name_plural}})
    EmptyTrash {
        /// Confirmation flag
        #[arg(long)]
        confirm: bool,
        /// User ID performing the action
        #[arg(long, default_value = "cli-user")]
        user_id: String,
    },
}

pub struct {{PascalCaseEntity}}CliHandler {
    services: {{PascalCaseEntity}}ApplicationServices,
}

impl {{PascalCaseEntity}}CliHandler {
    pub fn new(services: {{PascalCaseEntity}}ApplicationServices) -> Self {
        Self { services }
    }

    pub async fn handle(&self, commands: {{PascalCaseEntity}}Commands) -> Result<()> {
        match commands.action {
            {{PascalCaseEntity}}Action::Create { data, file, user_id } => {
                self.create_{{entity_name_snake}}(data, file, user_id).await
            }
            {{PascalCaseEntity}}Action::Get { id } => {
                self.get_{{entity_name_snake}}(&id).await
            }
            {{PascalCaseEntity}}Action::List { page, page_size, sort_by, sort_direction, search, filters } => {
                self.list_{{entity_name_plural}}(page, page_size, sort_by, sort_direction, search, filters).await
            }
            {{PascalCaseEntity}}Action::Update { id, data, file, user_id } => {
                self.update_{{entity_name_snake}}(&id, data, file, user_id).await
            }
            {{PascalCaseEntity}}Action::Delete { id, user_id } => {
                self.delete_{{entity_name_snake}}(&id, user_id).await
            }
            {{PascalCaseEntity}}Action::BulkCreate { file, user_id } => {
                self.bulk_create_{{entity_name_plural}}(file, user_id).await
            }
            {{PascalCaseEntity}}Action::Upsert { file, user_id } => {
                self.upsert_{{entity_name_plural}}(file, user_id).await
            }
            {{PascalCaseEntity}}Action::ListTrash { page, page_size } => {
                self.list_trash(page, page_size).await
            }
            {{PascalCaseEntity}}Action::Restore { id, user_id } => {
                self.restore_{{entity_name_snake}}(&id, user_id).await
            }
            {{PascalCaseEntity}}Action::EmptyTrash { confirm, user_id } => {
                self.empty_trash(confirm, user_id).await
            }
        }
    }

    async fn create_{{entity_name_snake}}(&self, data: Option<String>, file: Option<String>, user_id: String) -> Result<()> {
        let {{entity_name_snake}}_data = self.parse_{{entity_name_snake}}_data(data, file)?;

        let command = Create{{PascalCaseEntity}}Command {
            // TODO: Map parsed data to command fields
            custom_fields: {{entity_name_snake}}_data,
            created_by: user_id,
        };

        let response = self.services.create_{{entity_name_snake}}_handler().handle(command).await?;

        if response.success {
            println!("{}", "✅ {{PascalCaseEntity}} created successfully!".green());
            if let Some({{entity_name_snake}}) = response.{{entity_name_snake}} {
                self.display_{{entity_name_snake}}(&{{entity_name_snake}})?;
            }
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn get_{{entity_name_snake}}(&self, id: &str) -> Result<()> {
        let query = Get{{PascalCaseEntity}}Query {
            id: id.to_string(),
        };

        let response = self.services.get_{{entity_name_snake}}_handler().handle(query).await?;

        if response.success {
            if let Some({{entity_name_snake}}) = response.{{entity_name_snake}} {
                self.display_{{entity_name_snake}}(&{{entity_name_snake}})?;
            } else {
                println!("{}", "{{PascalCaseEntity}} not found".yellow());
            }
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn list_{{entity_name_plural}}(&self, page: usize, page_size: usize, sort_by: Option<String>, sort_direction: String, search: Option<String>, filters: Option<String>) -> Result<()> {
        let filters = if let Some(filters_json) = filters {
            Some(serde_json::from_str::<serde_json::Value>(&filters_json)?)
        } else {
            None
        };

        let filters = if let Some(filter_value) = filters {
            // TODO: Convert JSON filters to application filters
            Some({{PascalCaseEntity}}Filters::new().with_search(search.unwrap_or_default()))
        } else if search.is_some() {
            Some({{PascalCaseEntity}}Filters::new().with_search(search.unwrap_or_default()))
        } else {
            None
        };

        let query = List{{PascalCaseEntity}}Query {
            page,
            page_size,
            sort_by,
            sort_direction,
            filters,
        };

        let response = self.services.list_{{entity_name_plural}}_handler().handle(query).await?;

        if response.success {
            println!("{}", format!("📄 Found {} {{entity_name_plural}} (page {}/{}):",
                response.{{entity_name_plural}}.len(), response.page, response.total_pages).cyan());

            for {{entity_name_snake}} in &response.{{entity_name_plural}} {
                self.display_{{entity_name_snake}}_compact({{entity_name_snake}})?;
                println!("{}", "-".repeat(80));
            }

            if response.has_next || response.has_previous {
                println!();
                if response.has_previous {
                    println!("{}", format!("← Previous page: {}", response.page - 1).blue());
                }
                println!("{}", format!("Current page: {} of {}", response.page, response.total_pages).white());
                if response.has_next {
                    println!("{}", format!("Next page: {} →", response.page + 1).blue());
                }
            }
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn update_{{entity_name_snake}}(&self, id: &str, data: Option<String>, file: Option<String>, user_id: String) -> Result<()> {
        let {{entity_name_snake}}_data = self.parse_{{entity_name_snake}}_data(data, file)?;

        let command = Update{{PascalCaseEntity}}Command {
            id: id.to_string(),
            // TODO: Map parsed data to command fields
            custom_fields: {{entity_name_snake}}_data,
            updated_by: user_id,
        };

        let response = self.services.update_{{entity_name_snake}}_handler().handle(command).await?;

        if response.success {
            println!("{}", "✅ {{PascalCaseEntity}} updated successfully!".green());
            if let Some({{entity_name_snake}}) = response.{{entity_name_snake}} {
                self.display_{{entity_name_snake}}(&{{entity_name_snake}})?;
            }
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn delete_{{entity_name_snake}}(&self, id: &str, user_id: String) -> Result<()> {
        let command = Delete{{PascalCaseEntity}}Command {
            id: id.to_string(),
            deleted_by: user_id,
        };

        let response = self.services.delete_{{entity_name_snake}}_handler().handle(command).await?;

        if response.success {
            println!("{}", "✅ {{PascalCaseEntity}} deleted successfully!".green());
            println!("{}", "💡 Use 'restore' command to recover if needed".blue());
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn bulk_create_{{entity_name_plural}}(&self, file: Option<String>, user_id: String) -> Result<()> {
        let {{entity_name_plural}}_data = self.parse_bulk_{{entity_name_plural}}_data(file)?;

        let commands: Vec<Create{{PascalCaseEntity}}Command> = {{entity_name_plural}}_data
            .into_iter()
            .map(|data| Create{{PascalCaseEntity}}Command {
                custom_fields: data,
                created_by: user_id.clone(),
            })
            .collect();

        let command = BulkCreate{{PascalCaseEntity}}Command {
            {{entity_name_plural}}: commands,
        };

        let response = self.services.bulk_create_{{entity_name_plural}}_handler().handle(command).await?;

        if response.success {
            println!("{}", format!("✅ {} {{entity_name_plural}} created successfully!", response.created_count).green());
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn upsert_{{entity_name_plural}}(&self, file: Option<String>, user_id: String) -> Result<()> {
        let {{entity_name_plural}}_data = self.parse_bulk_{{entity_name_plural}}_data(file)?;

        let command = Upsert{{PascalCaseEntity}}Command {
            // TODO: Map parsed data to command fields
            custom_fields: {{entity_name_plural}}_data,
            user_id,
        };

        let response = self.services.upsert_{{entity_name_snake}}_handler().handle(command).await?;

        if response.success {
            println!("{}", format!("✅ Upsert completed! Created: {}, Updated: {}", response.created, response.updated).green());
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn list_trash(&self, page: usize, page_size: usize) -> Result<()> {
        // This would use ListDeleted{{PascalCaseEntity}}Query
        println!("{}", "🗑️  Deleted {{entity_name_plural}} (trash):".yellow());
        println!("{}", "Feature not yet implemented - TODO: Add ListDeleted{{PascalCaseEntity}}Query handler".yellow());
        Ok(())
    }

    async fn restore_{{entity_name_snake}}(&self, id: &str, user_id: String) -> Result<()> {
        let command = Restore{{PascalCaseEntity}}Command {
            id: id.to_string(),
            restored_by: user_id,
        };

        let response = self.services.restore_{{entity_name_snake}}_handler().handle(command).await?;

        if response.success {
            println!("{}", "✅ {{PascalCaseEntity}} restored successfully!".green());
            if let Some({{entity_name_snake}}) = response.{{entity_name_snake}} {
                self.display_{{entity_name_snake}}(&{{entity_name_snake}})?;
            }
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    async fn empty_trash(&self, confirm: bool, user_id: String) -> Result<()> {
        if !confirm {
            println!("{}", "⚠️  This action will permanently delete all deleted {{entity_name_plural}}!".yellow());
            println!("{}", "Use --confirm to proceed".yellow());
            return Ok(());
        }

        let command = EmptyTrashCommand {
            user_id,
        };

        let response = self.services.empty_{{entity_name_snake}}_trash_handler().handle(command).await?;

        if response.success {
            println!("{}", format!("✅ Trash emptied! {} {{entity_name_plural}} permanently deleted.", response.deleted_count).green());
        } else {
            eprintln!("{} {}", "❌".red(), response.message.red());
        }

        Ok(())
    }

    // Helper methods
    fn parse_{{entity_name_snake}}_data(&self, data: Option<String>, file: Option<String>) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        if let Some(file_path) = file {
            let content = std::fs::read_to_string(file_path)?;
            Ok(serde_json::from_str(&content)?)
        } else if let Some(json_data) = data {
            Ok(serde_json::from_str(&json_data)?)
        } else {
            // Return empty hash for interactive mode
            Ok(std::collections::HashMap::new())
        }
    }

    fn parse_bulk_{{entity_name_plural}}_data(&self, file: Option<String>) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        let content = if let Some(file_path) = file {
            std::fs::read_to_string(file_path)?
        } else {
            // Read from stdin
            use std::io::{self, Read};
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        };

        Ok(serde_json::from_str(&content)?)
    }

    fn display_{{entity_name_snake}}(&self, {{entity_name_snake}}: &{{PascalCaseEntity}}Dto) -> Result<()> {
        println!("{}", format!("📋 {{PascalCaseEntity}} Details:").cyan());
        println!("{}", format!("  ID: {}", {{entity_name_snake}}.id).white());
        // TODO: Add more field display based on your {{entity_name_snake}} structure
        println!("{}", format!("  Created: {:?}", {{entity_name_snake}}.created_at).white());
        println!("{}", format!("  Updated: {:?}", {{entity_name_snake}}.updated_at).white());
        Ok(())
    }

    fn display_{{entity_name_snake}}_compact(&self, {{entity_name_snake}}: &{{PascalCaseEntity}}Dto) -> Result<()> {
        println!("{}", format!("🔹 {} | {:?}", {{entity_name_snake}}.id, {{entity_name_snake}}.created_at).white());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{{entity_name_snake}}_commands_parsing() {
        // TODO: Add CLI command parsing tests
    }
}