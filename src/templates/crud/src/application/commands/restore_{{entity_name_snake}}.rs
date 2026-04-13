// Restore{{PascalCaseEntity}} Command
// Command for restoring soft-deleted {{PascalCaseEntity}} entities

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restore{{PascalCaseEntity}}Command {
    pub id: String,
}

impl Restore{{PascalCaseEntity}}Command {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restore{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
    pub {{entity_name_snake}}: Option<super::{{PascalCaseEntity}}Dto>,
}

impl Restore{{PascalCaseEntity}}Response {
    pub fn success({{entity_name_snake}}: super::{{PascalCaseEntity}}Dto) -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} restored successfully".to_string(),
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