// EmptyTrash Command
// Command for permanently deleting all soft-deleted {{PascalCaseEntity}} entities

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTrashCommand {
    // No fields needed for empty trash command
}

impl EmptyTrashCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTrashResponse {
    pub success: bool,
    pub message: String,
    pub deleted_count: usize,
}

impl EmptyTrashResponse {
    pub fn success(deleted_count: usize) -> Self {
        Self {
            success: true,
            message: format!("Permanently deleted {} {{entity_name_plural}} from trash", deleted_count),
            deleted_count,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            deleted_count: 0,
        }
    }
}