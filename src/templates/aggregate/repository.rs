//! {{PascalCaseEntity}} Repository Interface
//!
//! This repository provides collection-like access to {{PascalCaseEntity}} aggregates.
//! It follows DDD repository patterns and abstracts persistence concerns.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::{{entity_name_snake}}::*;

/// {{PascalCaseEntity}} Repository Interface
///
/// This trait defines the contract for {{PascalCaseEntity}} persistence.
/// Implementations can use different storage mechanisms (SQL, NoSQL, in-memory, etc.).
#[async_trait]
pub trait {{PascalCaseEntity}}Repository: Send + Sync {
    /// Save {{PascalCaseEntity}} aggregate (create or update)
    ///
    /// # Arguments
    ///
    /// * `aggregate` - The aggregate to save
    ///
    /// # Returns
    ///
    /// Returns the saved aggregate
    async fn save(&self, aggregate: {{PascalCaseEntity}}) -> Result<{{PascalCaseEntity}}, {{PascalCaseEntity}}RepositoryError>;

    /// Find {{PascalCaseEntity}} by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the aggregate
    ///
    /// # Returns
    ///
    /// Returns `Some(aggregate)` if found, `None` otherwise
    async fn find_by_id(&self, id: Uuid) -> Result<Option<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;

    /// Find multiple {{PascalCaseEntity}} aggregates by IDs
    ///
    /// # Arguments
    ///
    /// * `ids` - Vector of unique identifiers
    ///
    /// # Returns
    ///
    /// Returns a vector of found aggregates (order not guaranteed)
    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;

    /// Find all {{PascalCaseEntity}} aggregates
    ///
    /// # Arguments
    ///
    /// * `include_deleted` - Whether to include soft-deleted aggregates
    ///
    /// # Returns
    ///
    /// Returns a vector of all aggregates
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;

    /// Find {{PascalCaseEntity}} aggregates with pagination
    ///
    /// # Arguments
    ///
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Number of items per page
    /// * `include_deleted` - Whether to include soft-deleted aggregates
    ///
    /// # Returns
    ///
    /// Returns a paginated result with aggregates and metadata
    async fn find_with_pagination(
        &self,
        page: u32,
        limit: u32,
        include_deleted: bool,
    ) -> Result<PaginatedResult<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;

    /// Find {{PascalCaseEntity}} aggregates by criteria
    ///
    /// # Arguments
    ///
    /// * `criteria` - Query criteria
    ///
    /// # Returns
    ///
    /// Returns a vector of matching aggregates
    async fn find_by_criteria(
        &self,
        criteria: {{PascalCaseEntity}}QueryCriteria,
    ) -> Result<Vec<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;

    /// Delete {{PascalCaseEntity}} (soft delete)
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the aggregate to delete
    /// * `reason` - Optional reason for deletion
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if deleted, `Ok(false)` if not found
    async fn delete(&self, id: Uuid, reason: Option<String>) -> Result<bool, {{PascalCaseEntity}}RepositoryError>;

    /// Restore soft-deleted {{PascalCaseEntity}}
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the aggregate to restore
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if restored, `Ok(false)` if not found or not deleted
    async fn restore(&self, id: Uuid) -> Result<bool, {{PascalCaseEntity}}RepositoryError>;

    /// Check if {{PascalCaseEntity}} exists
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier to check
    ///
    /// # Returns
    ///
    /// Returns `true` if exists, `false` otherwise
    async fn exists(&self, id: Uuid) -> Result<bool, {{PascalCaseEntity}}RepositoryError>;

    /// Count {{PascalCaseEntity}} aggregates
    ///
    /// # Arguments
    ///
    /// * `exclude_deleted` - Whether to exclude soft-deleted aggregates
    ///
    /// # Returns
    ///
    /// Returns the total count
    async fn count(&self, exclude_deleted: bool) -> Result<i64, {{PascalCaseEntity}}RepositoryError>;

    // TODO: Add custom query methods specific to your domain
    // Example:
    // /// Find aggregates by status
    // async fn find_by_status(
    //     &self,
    //     status: {{PascalCaseEntity}}Status,
    // ) -> Result<Vec<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;
    //
    // /// Find aggregates by date range
    // async fn find_by_date_range(
    //     &self,
    //     start: DateTime<Utc>,
    //     end: DateTime<Utc>,
    // ) -> Result<Vec<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError>;
}

/// Paginated result wrapper
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// Items in the current page
    pub items: Vec<T>,

    /// Total number of items across all pages
    pub total_count: i64,

    /// Current page number (1-indexed)
    pub page: u32,

    /// Number of items per page
    pub limit: u32,

    /// Total number of pages
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(items: Vec<T>, total_count: i64, page: u32, limit: u32) -> Self {
        let total_pages = if limit == 0 {
            0
        } else {
            ((total_count as f64) / (limit as f64)).ceil() as u32
        };

        Self {
            items,
            total_count,
            page,
            limit,
            total_pages,
        }
    }

    /// Check if there is a next page
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there is a previous page
    pub fn has_previous(&self) -> bool {
        self.page > 1
    }
}

/// Query criteria for finding {{PascalCaseEntity}} aggregates
#[derive(Debug, Clone, Default)]
pub struct {{PascalCaseEntity}}QueryCriteria {
    // TODO: Add criteria fields specific to your domain
    // Example:
    // /// Name contains this string
    // pub name_contains: Option<String>,
    //
    // /// Status filter
    // pub status: Option<{{PascalCaseEntity}}Status>,
    //
    // /// Created after this timestamp
    // pub created_after: Option<DateTime<Utc>>,
    //
    // /// Created before this timestamp
    // pub created_before: Option<DateTime<Utc>>,
    //
    // /// Updated after this timestamp
    // pub updated_after: Option<DateTime<Utc>>,
    //
    // /// Custom filters
    // pub custom_filters: std::collections::HashMap<String, String>,
}

/// Repository error types
#[derive(Debug, thiserror::Error)]
pub enum {{PascalCaseEntity}}RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Concurrency conflict: {0}")]
    ConcurrencyError(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Helper conversions for common database errors
impl From<sqlx::Error> for {{PascalCaseEntity}}RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {{PascalCaseEntity}}RepositoryError::NotFound(err.to_string()),
            sqlx::Error::Database(db_err) => {
                match db_err.code() {
                    Some(code) if code == "23505" => { // Unique violation
                        {{PascalCaseEntity}}RepositoryError::AlreadyExists(db_err.message().to_string())
                    }
                    Some(code) if code == "23503" => { // Foreign key violation
                        {{PascalCaseEntity}}RepositoryError::ValidationError(db_err.message().to_string())
                    }
                    _ => {{PascalCaseEntity}}RepositoryError::DatabaseError(db_err.message().to_string()),
                }
            }
            _ => {{PascalCaseEntity}}RepositoryError::DatabaseError(err.to_string()),
        }
    }
}

// TODO: Add a concrete repository implementation example
// /// PostgreSQL implementation of {{PascalCaseEntity}}Repository
// pub struct Postgres{{PascalCaseEntity}}Repository {
//     pool: sqlx::PgPool,
// }
//
// impl Postgres{{PascalCaseEntity}}Repository {
//     pub fn new(pool: sqlx::PgPool) -> Self {
//         Self { pool }
//     }
// }
//
// #[async_trait]
// impl {{PascalCaseEntity}}Repository for Postgres{{PascalCaseEntity}}Repository {
//     async fn save(&self, aggregate: {{PascalCaseEntity}}) -> Result<{{PascalCaseEntity}}, {{PascalCaseEntity}}RepositoryError> {
//         // TODO: Implement save logic
//         // Example:
//         // sqlx::query!(
//         //     r#"
//         //     INSERT INTO {{entity_plural}} (id, name, created_at, updated_at, deleted_at)
//         //     VALUES ($1, $2, $3, $4, $5)
//         //     ON CONFLICT (id) DO UPDATE SET
//         //         name = EXCLUDED.name,
//         //         updated_at = EXCLUDED.updated_at
//         //     "#,
//         //     aggregate.id,
//         //     aggregate.name,
//         //     aggregate.created_at,
//             //     aggregate.updated_at,
//         //     aggregate.deleted_at,
//         // )
//         // .execute(&self.pool)
//         // .await?;
//         //
//         // Ok(aggregate)
//         todo!("Implement save")
//     }
//
//     async fn find_by_id(&self, id: Uuid) -> Result<Option<{{PascalCaseEntity}}>, {{PascalCaseEntity}}RepositoryError> {
//         // TODO: Implement find_by_id logic
//         todo!("Implement find_by_id")
//     }
//
//     // ... implement other methods
// }
