// BulkCreate{{PascalCaseEntity}} Command
// Command for bulk creating multiple {{PascalCaseEntity}} entities

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCreate{{PascalCaseEntity}}Command {
    pub items: Vec<super::Create{{PascalCaseEntity}}Command>,
}

impl BulkCreate{{PascalCaseEntity}}Command {
    pub fn new(items: Vec<super::Create{{PascalCaseEntity}}Command>) -> Self {
        Self { items }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCreate{{PascalCaseEntity}}Response {
    pub success: bool,
    pub message: String,
    pub created_count: usize,
    pub failed_count: usize,
    pub created_{{entity_name_plural}}: Vec<super::{{PascalCaseEntity}}Dto>,
    pub errors: Vec<String>,
}

impl BulkCreate{{PascalCaseEntity}}Response {
    pub fn new(
        created_count: usize,
        failed_count: usize,
        created_{{entity_name_plural}}: Vec<super::{{PascalCaseEntity}}Dto>,
        errors: Vec<String>,
    ) -> Self {
        let total_count = created_count + failed_count;
        let success = failed_count == 0;

        Self {
            success,
            message: if success {
                format!("Successfully created {} {{entity_name_plural}}", created_count)
            } else {
                format!("Created {} of {} {{entity_name_plural}} ({} failed)", created_count, total_count, failed_count)
            },
            created_count,
            failed_count,
            created_{{entity_name_plural}},
            errors,
        }
    }
}