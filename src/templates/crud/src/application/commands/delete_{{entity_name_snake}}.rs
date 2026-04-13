// Delete{{PascalCaseEntity}} Command
// Command for soft-deleting {{PascalCaseEntity}} entities

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delete{{PascalCaseEntity}}Command {
    pub id: String,
}

impl Delete{{PascalCaseEntity}}Command {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delete{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
}

impl Delete{{PascalCaseEntity}}Response {
    pub fn success() -> Self {
        Self {
            success: true,
            message: "{{PascalCaseEntity}} deleted successfully".to_string(),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
        }
    }
}