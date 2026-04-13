// Get{{PascalCaseEntity}} Query
// Query for retrieving a {{PascalCaseEntity}} by ID

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Get{{PascalCaseEntity}}Query {
    pub id: String,
}

impl Get{{PascalCaseEntity}}Query {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Get{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
    pub {{entity_name_snake}}: Option<crate::application::commands::{{PascalCaseEntity}}Dto>,
}

impl Get{{PascalCaseEntity}}Response {
    pub fn success({{entity_name_snake}}: crate::application::commands::{{PascalCaseEntity}}Dto) -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} retrieved successfully".to_string(),
            {{entity_name_snake}}: Some({{entity_name_snake}}),
        }
    }

    pub fn not_found(id: &str) -> Self {
        Self {
            success: false,
            message: format!("{{PascalCaseEntity}} with id '{}' not found", id),
            {{entity_name_snake}}: None,
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