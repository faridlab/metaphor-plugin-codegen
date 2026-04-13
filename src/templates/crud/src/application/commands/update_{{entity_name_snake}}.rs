// Update{{PascalCaseEntity}} Command
// Command for updating existing {{PascalCaseEntity}} entities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update{{PascalCaseEntity}}Command {
    pub id: String,
    // TODO: Add your command fields here based on entity proto
    // Example: pub name: Option<String>;
    // Example: pub description: Option<String>;

    // Generic fields for any custom data
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub updated_by: String,
}

impl Update{{PascalCaseEntity}}Command {
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
pub struct Update{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
    pub {{entity_name_snake}}: Option<super::{{PascalCaseEntity}}Dto>,
}

impl Update{{PascalCaseEntity}}Response {
    pub fn success({{entity_name_snake}}: super::{{PascalCaseEntity}}Dto) -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} updated successfully".to_string(),
            {{entity_name_snake}}: Some({{entity_name_snake}}),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            {{entity_name_snake}}: None,
        }
    }
}