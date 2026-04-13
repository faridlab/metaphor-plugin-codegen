//! Generic CRUD Repository Implementation
//!
//! Provides a generic PostgreSQL-based CRUD repository that can work with any entity.
//! This is the core of the Metaphor Framework's 11-endpoint CRUD system.
#![allow(dead_code)]
#![allow(unused_variables)]

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};

/// Pagination parameters for list queries
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub sort_by: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
    pub search: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}
fn default_sort_order() -> String {
    "asc".to_string()
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: "asc".to_string(),
            search: None,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

/// Base entity trait that all CRUD entities must implement
pub trait CrudEntity: Send + Sync + Clone + Serialize + DeserializeOwned + 'static {
    fn id(&self) -> Uuid;
    fn table_name() -> &'static str;
    fn searchable_fields() -> Vec<&'static str> {
        vec![]
    }
    fn sortable_fields() -> Vec<&'static str> {
        vec!["id", "created_at", "updated_at"]
    }
}

/// Generic CRUD Repository trait
#[async_trait]
pub trait CrudRepository<T: CrudEntity>: Send + Sync {
    // Core CRUD operations (11 standard Metaphor endpoints)

    /// 1. Create a new entity
    async fn create(&self, entity: &T) -> AppResult<T>;

    /// 2. Get entity by ID
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<T>>;

    /// 3. List entities with pagination
    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>>;

    /// 4. Full update (PUT)
    async fn update(&self, id: Uuid, entity: &T) -> AppResult<Option<T>>;

    /// 5. Partial update (PATCH)
    async fn partial_update(
        &self,
        id: Uuid,
        updates: serde_json::Value,
    ) -> AppResult<Option<T>>;

    /// 6. Soft delete
    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> AppResult<bool>;

    /// 7. Bulk create
    async fn bulk_create(&self, entities: &[T]) -> AppResult<BulkCreateResult<T>>;

    /// 8. Upsert (create or update)
    async fn upsert(&self, entity: &T) -> AppResult<UpsertResult<T>>;

    /// 9. List deleted (trash)
    async fn find_deleted(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>>;

    /// 10. Restore from trash
    async fn restore(&self, id: Uuid, restored_by: Option<Uuid>) -> AppResult<Option<T>>;

    /// 11. Empty trash (permanent delete)
    async fn empty_trash(&self) -> AppResult<u64>;

    // Additional utility methods
    async fn count(&self, include_deleted: bool) -> AppResult<i64>;
    async fn exists(&self, id: Uuid) -> AppResult<bool>;
    async fn hard_delete(&self, id: Uuid) -> AppResult<bool>;
}

/// Result of bulk create operation
#[derive(Debug, Clone, Serialize)]
pub struct BulkCreateResult<T> {
    pub created: Vec<T>,
    pub failed: Vec<BulkCreateError>,
    pub created_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkCreateError {
    pub index: usize,
    pub error: String,
}

/// Result of upsert operation
#[derive(Debug, Clone, Serialize)]
pub struct UpsertResult<T> {
    pub entity: T,
    pub was_created: bool,
}

/// Generic PostgreSQL CRUD Repository Implementation
pub struct PostgresCrudRepository<T: CrudEntity> {
    pool: PgPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: CrudEntity> PostgresCrudRepository<T> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }

    fn build_search_clause(search: &str, fields: &[&str]) -> String {
        if fields.is_empty() {
            return "TRUE".to_string();
        }

        let conditions: Vec<String> = fields
            .iter()
            .map(|field| format!("CAST({} AS TEXT) ILIKE $1", field))
            .collect();

        format!("({})", conditions.join(" OR "))
    }

    fn validate_sort_field(field: &str, allowed: &[&str]) -> bool {
        allowed.contains(&field)
    }
}

#[async_trait]
impl<T: CrudEntity + for<'r> FromRow<'r, sqlx::postgres::PgRow> + Unpin> CrudRepository<T>
    for PostgresCrudRepository<T>
{
    async fn create(&self, entity: &T) -> AppResult<T> {
        let table = T::table_name();
        let json = serde_json::to_value(entity).map_err(|e| AppError::Serialization(e.to_string()))?;

        let row = sqlx::query(&format!(
            r#"
            INSERT INTO {} (id, data, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            RETURNING *
            "#,
            table
        ))
        .bind(entity.id())
        .bind(&json)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        let data: serde_json::Value = row.try_get("data").map_err(AppError::Database)?;
        serde_json::from_value(data).map_err(|e| AppError::Serialization(e.to_string()))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<T>> {
        let table = T::table_name();

        let result = sqlx::query(&format!(
            "SELECT data FROM {} WHERE id = $1 AND deleted_at IS NULL",
            table
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match result {
            Some(row) => {
                let data: serde_json::Value = row.try_get("data").map_err(AppError::Database)?;
                let entity =
                    serde_json::from_value(data).map_err(|e| AppError::Serialization(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>> {
        let table = T::table_name();
        let offset = (params.page.saturating_sub(1)) * params.page_size;

        let sort_field = params
            .sort_by
            .as_ref()
            .filter(|f| Self::validate_sort_field(f, &T::sortable_fields()))
            .map(|s| s.as_str())
            .unwrap_or("created_at");

        let sort_order = if params.sort_order.to_lowercase() == "desc" {
            "DESC"
        } else {
            "ASC"
        };

        let (where_clause, search_param) = if let Some(ref search) = params.search {
            let clause = Self::build_search_clause(search, &T::searchable_fields());
            (format!("AND {}", clause), Some(format!("%{}%", search)))
        } else {
            ("".to_string(), None)
        };

        let count_query = format!(
            "SELECT COUNT(*) as count FROM {} WHERE deleted_at IS NULL {}",
            table, where_clause
        );

        let total: i64 = if let Some(ref search) = search_param {
            sqlx::query(&count_query)
                .bind(search)
                .fetch_one(&self.pool)
                .await
                .map_err(AppError::Database)?
                .try_get("count")
                .map_err(AppError::Database)?
        } else {
            sqlx::query(&count_query)
                .fetch_one(&self.pool)
                .await
                .map_err(AppError::Database)?
                .try_get("count")
                .map_err(AppError::Database)?
        };

        let rows = if search_param.is_some() {
            sqlx::query(&format!(
                "SELECT data FROM {} WHERE deleted_at IS NULL {} ORDER BY {} {} LIMIT $2 OFFSET $3",
                table, where_clause, sort_field, sort_order
            ))
            .bind(search_param.as_ref().unwrap())
            .bind(params.page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query(&format!(
                "SELECT data FROM {} WHERE deleted_at IS NULL ORDER BY {} {} LIMIT $1 OFFSET $2",
                table, sort_field, sort_order
            ))
            .bind(params.page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?
        };

        let data: Vec<T> = rows
            .into_iter()
            .filter_map(|row| {
                let json: serde_json::Value = row.try_get("data").ok()?;
                serde_json::from_value(json).ok()
            })
            .collect();

        let total_pages = ((total as f64) / (params.page_size as f64)).ceil() as u32;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationInfo {
                page: params.page,
                page_size: params.page_size,
                total,
                total_pages,
                has_next: params.page < total_pages,
                has_previous: params.page > 1,
            },
        })
    }

    async fn update(&self, id: Uuid, entity: &T) -> AppResult<Option<T>> {
        let table = T::table_name();
        let json = serde_json::to_value(entity).map_err(|e| AppError::Serialization(e.to_string()))?;

        let result = sqlx::query(&format!(
            r#"
            UPDATE {}
            SET data = $1, updated_at = NOW()
            WHERE id = $2 AND deleted_at IS NULL
            RETURNING data
            "#,
            table
        ))
        .bind(&json)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match result {
            Some(row) => {
                let data: serde_json::Value = row.try_get("data").map_err(AppError::Database)?;
                let entity =
                    serde_json::from_value(data).map_err(|e| AppError::Serialization(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn partial_update(
        &self,
        id: Uuid,
        updates: serde_json::Value,
    ) -> AppResult<Option<T>> {
        let table = T::table_name();

        let existing = self.find_by_id(id).await?;
        let Some(existing) = existing else {
            return Ok(None);
        };

        let mut existing_json =
            serde_json::to_value(&existing).map_err(|e| AppError::Serialization(e.to_string()))?;

        if let (serde_json::Value::Object(ref mut existing_map), serde_json::Value::Object(updates_map)) =
            (&mut existing_json, updates)
        {
            for (key, value) in updates_map {
                existing_map.insert(key, value);
            }
        }

        let result = sqlx::query(&format!(
            r#"
            UPDATE {}
            SET data = $1, updated_at = NOW()
            WHERE id = $2 AND deleted_at IS NULL
            RETURNING data
            "#,
            table
        ))
        .bind(&existing_json)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match result {
            Some(row) => {
                let data: serde_json::Value = row.try_get("data").map_err(AppError::Database)?;
                let entity =
                    serde_json::from_value(data).map_err(|e| AppError::Serialization(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> AppResult<bool> {
        let table = T::table_name();

        let result = sqlx::query(&format!(
            r#"
            UPDATE {}
            SET deleted_at = NOW(), deleted_by = $1
            WHERE id = $2 AND deleted_at IS NULL
            "#,
            table
        ))
        .bind(deleted_by)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected() > 0)
    }

    async fn bulk_create(&self, entities: &[T]) -> AppResult<BulkCreateResult<T>> {
        let mut created = Vec::new();
        let mut failed = Vec::new();

        for (index, entity) in entities.iter().enumerate() {
            match self.create(entity).await {
                Ok(e) => created.push(e),
                Err(e) => failed.push(BulkCreateError {
                    index,
                    error: e.to_string(),
                }),
            }
        }

        Ok(BulkCreateResult {
            created_count: created.len(),
            failed_count: failed.len(),
            created,
            failed,
        })
    }

    async fn upsert(&self, entity: &T) -> AppResult<UpsertResult<T>> {
        let id = entity.id();
        let exists = self.exists(id).await?;

        if exists {
            let updated = self.update(id, entity).await?;
            Ok(UpsertResult {
                entity: updated.unwrap_or_else(|| entity.clone()),
                was_created: false,
            })
        } else {
            let created = self.create(entity).await?;
            Ok(UpsertResult {
                entity: created,
                was_created: true,
            })
        }
    }

    async fn find_deleted(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>> {
        let table = T::table_name();
        let offset = (params.page.saturating_sub(1)) * params.page_size;

        let sort_field = params
            .sort_by
            .as_ref()
            .filter(|f| Self::validate_sort_field(f, &T::sortable_fields()))
            .map(|s| s.as_str())
            .unwrap_or("deleted_at");

        let sort_order = if params.sort_order.to_lowercase() == "desc" {
            "DESC"
        } else {
            "ASC"
        };

        let total: i64 = sqlx::query(&format!(
            "SELECT COUNT(*) as count FROM {} WHERE deleted_at IS NOT NULL",
            table
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?
        .try_get("count")
        .map_err(AppError::Database)?;

        let rows = sqlx::query(&format!(
            "SELECT data FROM {} WHERE deleted_at IS NOT NULL ORDER BY {} {} LIMIT $1 OFFSET $2",
            table, sort_field, sort_order
        ))
        .bind(params.page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        let data: Vec<T> = rows
            .into_iter()
            .filter_map(|row| {
                let json: serde_json::Value = row.try_get("data").ok()?;
                serde_json::from_value(json).ok()
            })
            .collect();

        let total_pages = ((total as f64) / (params.page_size as f64)).ceil() as u32;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationInfo {
                page: params.page,
                page_size: params.page_size,
                total,
                total_pages,
                has_next: params.page < total_pages,
                has_previous: params.page > 1,
            },
        })
    }

    async fn restore(&self, id: Uuid, restored_by: Option<Uuid>) -> AppResult<Option<T>> {
        let table = T::table_name();

        let result = sqlx::query(&format!(
            r#"
            UPDATE {}
            SET deleted_at = NULL, deleted_by = NULL, updated_at = NOW(), updated_by = $1
            WHERE id = $2 AND deleted_at IS NOT NULL
            RETURNING data
            "#,
            table
        ))
        .bind(restored_by)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match result {
            Some(row) => {
                let data: serde_json::Value = row.try_get("data").map_err(AppError::Database)?;
                let entity =
                    serde_json::from_value(data).map_err(|e| AppError::Serialization(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn empty_trash(&self) -> AppResult<u64> {
        let table = T::table_name();

        let result = sqlx::query(&format!(
            "DELETE FROM {} WHERE deleted_at IS NOT NULL",
            table
        ))
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    async fn count(&self, include_deleted: bool) -> AppResult<i64> {
        let table = T::table_name();

        let where_clause = if include_deleted {
            ""
        } else {
            "WHERE deleted_at IS NULL"
        };

        let count: i64 = sqlx::query(&format!("SELECT COUNT(*) as count FROM {} {}", table, where_clause))
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::Database)?
            .try_get("count")
            .map_err(AppError::Database)?;

        Ok(count)
    }

    async fn exists(&self, id: Uuid) -> AppResult<bool> {
        let table = T::table_name();

        let result: Option<i64> = sqlx::query(&format!(
            "SELECT 1 as exists FROM {} WHERE id = $1 LIMIT 1",
            table
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?
        .map(|row| row.try_get("exists").unwrap_or(0));

        Ok(result.is_some())
    }

    async fn hard_delete(&self, id: Uuid) -> AppResult<bool> {
        let table = T::table_name();

        let result = sqlx::query(&format!("DELETE FROM {} WHERE id = $1", table))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert_eq!(params.sort_order, "asc");
    }

    #[test]
    fn test_search_clause_builder() {
        let clause = PostgresCrudRepository::<TestEntity>::build_search_clause(
            "test",
            &["name", "email"],
        );
        assert!(clause.contains("name"));
        assert!(clause.contains("email"));
        assert!(clause.contains("ILIKE"));
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEntity {
        id: Uuid,
        name: String,
    }

    impl CrudEntity for TestEntity {
        fn id(&self) -> Uuid {
            self.id
        }
        fn table_name() -> &'static str {
            "test_entities"
        }
        fn searchable_fields() -> Vec<&'static str> {
            vec!["name"]
        }
    }
}
