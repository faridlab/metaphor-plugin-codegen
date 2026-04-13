// Upsert{{PascalCaseEntity}} Command
// Command for upserting {{PascalCaseEntity}} entities (update or insert)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upsert{{PascalCaseEntity}}Command {
    pub id: String,
    // TODO: Add your command fields here based on entity proto
    // Example: pub name: String;
    // Example: pub description: Option<String>;

    // Generic fields for any custom data
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub updated_by: String,
}

impl Upsert{{PascalCaseEntity}}Command {
    pub fn new(
        id: String,
        custom_fields: HashMap<String, serde_json::Value>,
        updated_by: String,
    ) -> Self {
        Self {
            id,
            custom_fields,
            updated_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upsert{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
    pub {{entity_name_snake}}: Option<super::{{PascalCaseEntity}}Dto>,
    pub was_created: bool,
}

impl Upsert{{PascalCaseEntity}}Response {
    pub fn created({{entity_name_snake}}: super::{{PascalCaseEntity}}Dto) -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} created successfully".to_string(),
            {{entity_name_snake}}: Some({{entity_name_snake}}),
            was_created: true,
        }
    }

    pub fn updated({{entity_name_snake}}: super::{{PascalCaseEntity}}Dto) -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} updated successfully".to_string(),
            {{entity_name_snake}}: Some({{entity_name_snake}}),
            was_created: false,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            {{entity_name_snake}}: None,
            was_created: false,
        }
    }
}