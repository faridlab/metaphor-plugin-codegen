//! In-Memory CRUD Repository Implementation
//!
//! A simple in-memory repository for testing and development.
//! Data is stored in memory and lost when the application restarts.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::infrastructure::database::crud::{
    BulkCreateResult, BulkCreateError, CrudEntity, CrudRepository, PaginatedResponse,
    PaginationInfo, PaginationParams, UpsertResult,
};
use crate::shared::error::{AppError, AppResult};

/// In-memory storage for entities
struct EntityStorage<T> {
    data: HashMap<Uuid, T>,
    deleted: HashMap<Uuid, T>,
}

impl<T> Default for EntityStorage<T> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            deleted: HashMap::new(),
        }
    }
}

/// In-memory CRUD Repository Implementation
pub struct InMemoryCrudRepository<T: CrudEntity> {
    storage: RwLock<EntityStorage<T>>,
}

impl<T: CrudEntity> InMemoryCrudRepository<T> {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(EntityStorage::default()),
        }
    }
}

impl<T: CrudEntity> Default for InMemoryCrudRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T: CrudEntity + Send + Sync + Clone + 'static> CrudRepository<T> for InMemoryCrudRepository<T> {
    async fn create(&self, entity: &T) -> AppResult<T> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let id = entity.id();
        if storage.data.contains_key(&id) {
            return Err(AppError::Conflict(format!("Entity with id {} already exists", id)));
        }

        storage.data.insert(id, entity.clone());
        Ok(entity.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<T>> {
        let storage = self.storage.read().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        Ok(storage.data.get(&id).cloned())
    }

    async fn find_all(&self, params: &PaginationParams) -> AppResult<PaginatedResponse<T>> {
        let storage = self.storage.read().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let mut data: Vec<T> = storage.data.values().cloned().collect();

        // Simple search filter (by serializing to JSON and checking)
        if let Some(ref search) = params.search {
            let search_lower = search.to_lowercase();
            data.retain(|entity| {
                if let Ok(json) = serde_json::to_string(entity) {
                    json.to_lowercase().contains(&search_lower)
                } else {
                    false
                }
            });
        }

        let total = data.len() as i64;
        let page_size = params.page_size.max(1);
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        // Pagination
        let offset = (params.page.saturating_sub(1) * page_size) as usize;
        let limit = page_size as usize;

        let paginated_data: Vec<T> = data.into_iter().skip(offset).take(limit).collect();

        Ok(PaginatedResponse {
            data: paginated_data,
            pagination: PaginationInfo {
                page: params.page,
                page_size,
                total,
                total_pages,
                has_next: params.page < total_pages,
                has_previous: params.page > 1,
            },
        })
    }

    async fn update(&self, id: Uuid, entity: &T) -> AppResult<Option<T>> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        if !storage.data.contains_key(&id) {
            return Ok(None);
        }

        storage.data.insert(id, entity.clone());
        Ok(Some(entity.clone()))
    }

    async fn partial_update(
        &self,
        id: Uuid,
        updates: serde_json::Value,
    ) -> AppResult<Option<T>> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let existing = match storage.data.get(&id) {
            Some(e) => e.clone(),
            None => return Ok(None),
        };

        // Merge updates into existing
        let mut existing_json = serde_json::to_value(&existing)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        if let (serde_json::Value::Object(ref mut existing_map), serde_json::Value::Object(updates_map)) =
            (&mut existing_json, updates)
        {
            for (key, value) in updates_map {
                existing_map.insert(key, value);
            }
        }

        let updated: T = serde_json::from_value(existing_json)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        storage.data.insert(id, updated.clone());
        Ok(Some(updated))
    }

    async fn soft_delete(&self, id: Uuid, _deleted_by: Option<Uuid>) -> AppResult<bool> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        if let Some(entity) = storage.data.remove(&id) {
            storage.deleted.insert(id, entity);
            Ok(true)
        } else {
            Ok(false)
        }
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
        let storage = self.storage.read().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let data: Vec<T> = storage.deleted.values().cloned().collect();
        let total = data.len() as i64;
        let page_size = params.page_size.max(1);
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        let offset = (params.page.saturating_sub(1) * page_size) as usize;
        let limit = page_size as usize;

        let paginated_data: Vec<T> = data.into_iter().skip(offset).take(limit).collect();

        Ok(PaginatedResponse {
            data: paginated_data,
            pagination: PaginationInfo {
                page: params.page,
                page_size,
                total,
                total_pages,
                has_next: params.page < total_pages,
                has_previous: params.page > 1,
            },
        })
    }

    async fn restore(&self, id: Uuid, _restored_by: Option<Uuid>) -> AppResult<Option<T>> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        if let Some(entity) = storage.deleted.remove(&id) {
            storage.data.insert(id, entity.clone());
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    async fn empty_trash(&self) -> AppResult<u64> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let count = storage.deleted.len() as u64;
        storage.deleted.clear();
        Ok(count)
    }

    async fn count(&self, include_deleted: bool) -> AppResult<i64> {
        let storage = self.storage.read().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let count = if include_deleted {
            storage.data.len() + storage.deleted.len()
        } else {
            storage.data.len()
        };

        Ok(count as i64)
    }

    async fn exists(&self, id: Uuid) -> AppResult<bool> {
        let storage = self.storage.read().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        Ok(storage.data.contains_key(&id))
    }

    async fn hard_delete(&self, id: Uuid) -> AppResult<bool> {
        let mut storage = self.storage.write().map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Lock error: {}", e))
        })?;

        let removed_from_data = storage.data.remove(&id).is_some();
        let removed_from_deleted = storage.deleted.remove(&id).is_some();

        Ok(removed_from_data || removed_from_deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let repo = InMemoryCrudRepository::<TestEntity>::new();

        let entity = TestEntity {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
        };

        let created = repo.create(&entity).await.unwrap();
        assert_eq!(created.name, "Test");

        let found = repo.find_by_id(entity.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");
    }

    #[tokio::test]
    async fn test_soft_delete_and_restore() {
        let repo = InMemoryCrudRepository::<TestEntity>::new();

        let entity = TestEntity {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
        };

        repo.create(&entity).await.unwrap();

        // Soft delete
        let deleted = repo.soft_delete(entity.id, None).await.unwrap();
        assert!(deleted);

        // Should not be found normally
        let found = repo.find_by_id(entity.id).await.unwrap();
        assert!(found.is_none());

        // Should be in trash
        let trash = repo.find_deleted(&PaginationParams::default()).await.unwrap();
        assert_eq!(trash.pagination.total, 1);

        // Restore
        let restored = repo.restore(entity.id, None).await.unwrap();
        assert!(restored.is_some());

        // Should be found again
        let found = repo.find_by_id(entity.id).await.unwrap();
        assert!(found.is_some());
    }
}
