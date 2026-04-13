// Get Metaphor Query
// Query handler for retrieving Metaphor entities

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::entities::Metaphor;
use crate::domain::repositories::{MetaphorRepository, PaginationParams, SortParams};
use crate::domain::value_objects::{MetaphorId, MetaphorStatus};
use crate::domain::{DomainError, DomainResult};

// Query DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMetaphorQuery {
    pub id: String,
}

impl GetMetaphorQuery {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn validate(&self) -> DomainResult<()> {
        if self.id.trim().is_empty() {
            return Err(DomainError::ValidationError {
                message: "Metaphor ID cannot be empty".to_string(),
            });
        }

        // Basic UUID format validation
        let id = MetaphorId::new(&self.id).map_err(|_| DomainError::ValidationError {
            message: "Invalid Metaphor ID format".to_string(),
        })?;

        // If we get here, the ID is valid
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMetaphorsQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub filters: Option<MetaphorFilters>,
}

impl ListMetaphorsQuery {
    pub fn new() -> Self {
        Self {
            page: None,
            page_size: None,
            sort_by: None,
            sort_direction: None,
            filters: None,
        }
    }

    pub fn with_pagination(mut self, page: usize, page_size: usize) -> Self {
        self.page = Some(page);
        self.page_size = Some(page_size);
        self
    }

    pub fn with_sort(mut self, sort_by: String, sort_direction: String) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_direction = Some(sort_direction);
        self
    }

    pub fn with_filters(mut self, filters: MetaphorFilters) -> Self {
        self.filters = Some(filters);
        self
    }

    pub fn validate(&self) -> DomainResult<()> {
        if let Some(page) = self.page {
            if page == 0 {
                return Err(DomainError::ValidationError {
                    message: "Page number must be greater than 0".to_string(),
                });
            }
        }

        if let Some(page_size) = self.page_size {
            if page_size == 0 {
                return Err(DomainError::ValidationError {
                    message: "Page size must be greater than 0".to_string(),
                });
            }
            if page_size > 100 {
                return Err(DomainError::ValidationError {
                    message: "Page size cannot exceed 100".to_string(),
                });
            }
        }

        if let Some(ref sort_by) = self.sort_by {
            let valid_sort_fields = vec!["id", "name", "status", "created_at", "updated_at", "created_by"];
            if !valid_sort_fields.contains(&sort_by.as_str()) {
                return Err(DomainError::ValidationError {
                    message: format!("Invalid sort field: {}", sort_by),
                });
            }
        }

        if let Some(ref sort_direction) = self.sort_direction {
            let valid_directions = vec!["asc", "ascending", "desc", "descending"];
            if !valid_directions.contains(&sort_direction.to_lowercase().as_str()) {
                return Err(DomainError::ValidationError {
                    message: format!("Invalid sort direction: {}", sort_direction),
                });
            }
        }

        Ok(())
    }

    pub fn get_pagination_params(&self) -> PaginationParams {
        PaginationParams::new(
            self.page.unwrap_or(1),
            self.page_size.unwrap_or(20),
        )
    }

    pub fn get_sort_params(&self) -> SortParams {
        use crate::domain::repositories::{SortDirection, SortField};

        let field = match self.sort_by.as_ref().map(|s| s.as_str()) {
            Some("id") => SortField::Id,
            Some("name") => SortField::Name,
            Some("status") => SortField::Status,
            Some("updated_at") => SortField::UpdatedAt,
            Some("created_by") => SortField::CreatedBy,
            _ => SortField::CreatedAt, // Default
        };

        let direction = match self.sort_direction.as_ref().map(|s| s.to_lowercase().as_str()) {
            Some("desc") | Some("descending") => SortDirection::Descending,
            _ => SortDirection::Ascending, // Default
        };

        SortParams::new(field, direction)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetaphorsQuery {
    pub query: String,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl SearchMetaphorsQuery {
    pub fn new(query: String) -> Self {
        Self {
            query,
            page: None,
            page_size: None,
            sort_by: None,
            sort_direction: None,
        }
    }

    pub fn with_pagination(mut self, page: usize, page_size: usize) -> Self {
        self.page = Some(page);
        self.page_size = Some(page_size);
        self
    }

    pub fn with_sort(mut self, sort_by: String, sort_direction: String) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_direction = Some(sort_direction);
        self
    }

    pub fn validate(&self) -> DomainResult<()> {
        if self.query.trim().is_empty() {
            return Err(DomainError::ValidationError {
                message: "Search query cannot be empty".to_string(),
            });
        }

        if self.query.len() > 1000 {
            return Err(DomainError::ValidationError {
                message: "Search query cannot exceed 1000 characters".to_string(),
            });
        }

        // Validate pagination and sorting (reuse validation from ListMetaphorsQuery)
        let list_query = ListMetaphorsQuery {
            page: self.page,
            page_size: self.page_size,
            sort_by: self.sort_by.clone(),
            sort_direction: self.sort_direction.clone(),
            filters: None,
        };

        list_query.validate()
    }

    pub fn get_pagination_params(&self) -> PaginationParams {
        PaginationParams::new(
            self.page.unwrap_or(1),
            self.page_size.unwrap_or(20),
        )
    }

    pub fn get_sort_params(&self) -> SortParams {
        let list_query = ListMetaphorsQuery {
            page: None,
            page_size: None,
            sort_by: self.sort_by.clone(),
            sort_direction: self.sort_direction.clone(),
            filters: None,
        };

        list_query.get_sort_params()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaphorFilters {
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub created_after: Option<String>, // ISO 8601 datetime
    pub created_before: Option<String>, // ISO 8601 datetime
    pub updated_after: Option<String>, // ISO 8601 datetime
    pub updated_before: Option<String>, // ISO 8601 datetime
    pub metadata: Option<HashMap<String, String>>,
}

impl MetaphorFilters {
    pub fn new() -> Self {
        Self {
            status: None,
            tags: None,
            created_by: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            metadata: None,
        }
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_created_by(mut self, created_by: String) -> Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn with_date_range(
        mut self,
        created_after: Option<String>,
        created_before: Option<String>,
    ) -> Self {
        self.created_after = created_after;
        self.created_before = created_before;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn validate(&self) -> DomainResult<()> {
        // Validate status
        if let Some(ref status) = self.status {
            let valid_statuses = vec!["ACTIVE", "INACTIVE", "SUSPENDED", "ARCHIVED"];
            if !valid_statuses.contains(&status.to_uppercase().as_str()) {
                return Err(DomainError::ValidationError {
                    message: format!("Invalid status: {}", status),
                });
            }
        }

        // Validate tags
        if let Some(ref tags) = self.tags {
            if tags.len() > 50 {
                return Err(DomainError::ValidationError {
                    message: "Cannot filter by more than 50 tags".to_string(),
                });
            }

            for tag in tags {
                if tag.trim().is_empty() {
                    return Err(DomainError::ValidationError {
                        message: "Filter tags cannot be empty".to_string(),
                    });
                }
                if tag.len() > 50 {
                    return Err(DomainError::ValidationError {
                        message: "Filter tag cannot exceed 50 characters".to_string(),
                    });
                }
            }
        }

        // Validate date formats (basic ISO 8601 validation)
        for date_field in [
            &self.created_after,
            &self.created_before,
            &self.updated_after,
            &self.updated_before,
        ] {
            if let Some(date_str) = date_field {
                if let Err(_) = chrono::DateTime::parse_from_rfc3339(date_str) {
                    return Err(DomainError::ValidationError {
                        message: format!("Invalid date format: {}. Expected ISO 8601 format", date_str),
                    });
                }
            }
        }

        // Validate metadata
        if let Some(ref metadata) = self.metadata {
            if metadata.len() > 20 {
                return Err(DomainError::ValidationError {
                    message: "Cannot filter by more than 20 metadata key-value pairs".to_string(),
                });
            }

            for (key, value) in metadata {
                if key.is_empty() {
                    return Err(DomainError::ValidationError {
                        message: "Metadata filter keys cannot be empty".to_string(),
                    });
                }
                if key.len() > 50 {
                    return Err(DomainError::ValidationError {
                        message: "Metadata filter key cannot exceed 50 characters".to_string(),
                    });
                }
                if value.len() > 500 {
                    return Err(DomainError::ValidationError {
                        message: "Metadata filter value cannot exceed 500 characters".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn to_repository_filters(&self) -> crate::domain::repositories::MetaphorFilters {
        use crate::domain::repositories::MetaphorFilters as RepoFilters;

        let mut filters = RepoFilters::new();

        if let Some(ref status) = self.status {
            filters.status = Some(match status.to_uppercase().as_str() {
                "ACTIVE" => MetaphorStatus::Active,
                "INACTIVE" => MetaphorStatus::Inactive,
                "SUSPENDED" => MetaphorStatus::Suspended,
                "ARCHIVED" => MetaphorStatus::Archived,
                _ => MetaphorStatus::Active, // Default fallback
            });
        }

        if let Some(ref tags) = self.tags {
            filters.tags = Some(tags.clone());
        }

        if let Some(ref created_by) = self.created_by {
            filters.created_by = Some(created_by.clone());
        }

        if let Some(ref created_after) = self.created_after {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_after) {
                filters.created_after = Some(dt.with_timezone(&chrono::Utc));
            }
        }

        if let Some(ref created_before) = self.created_before {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_before) {
                filters.created_before = Some(dt.with_timezone(&chrono::Utc));
            }
        }

        if let Some(ref updated_after) = self.updated_after {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_after) {
                filters.updated_after = Some(dt.with_timezone(&chrono::Utc));
            }
        }

        if let Some(ref updated_before) = self.updated_before {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_before) {
                filters.updated_before = Some(dt.with_timezone(&chrono::Utc));
            }
        }

        if let Some(ref metadata) = self.metadata {
            filters.metadata = Some(metadata.clone());
        }

        filters
    }
}

impl Default for MetaphorFilters {
    fn default() -> Self {
        Self::new()
    }
}

// Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMetaphorResponse {
    pub success: bool,
    pub metaphor: Option<MetaphorDto>,
    pub message: String,
}

impl GetMetaphorResponse {
    pub fn success(metaphor: Metaphor) -> Self {
        Self {
            success: true,
            metaphor: Some(MetaphorDto::from(metaphor)),
            message: "Metaphor retrieved successfully".to_string(),
        }
    }

    pub fn not_found(id: String) -> Self {
        Self {
            success: false,
            metaphor: None,
            message: format!("Metaphor with ID '{}' not found", id),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            metaphor: None,
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMetaphorsResponse {
    pub success: bool,
    pub metaphors: Vec<MetaphorDto>,
    pub total: u64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
    pub message: String,
}

impl ListMetaphorsResponse {
    pub fn success(
        paginated_result: crate::domain::repositories::PaginatedResult<Metaphor>,
    ) -> Self {
        let metaphors: Vec<MetaphorDto> = paginated_result
            .items
            .into_iter()
            .map(MetaphorDto::from)
            .collect();

        Self {
            success: true,
            metaphors,
            total: paginated_result.total,
            page: paginated_result.page,
            page_size: paginated_result.page_size,
            total_pages: paginated_result.total_pages,
            has_next: paginated_result.has_next,
            has_previous: paginated_result.has_previous,
            message: "Metaphors retrieved successfully".to_string(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            metaphors: Vec::new(),
            total: 0,
            page: 1,
            page_size: 20,
            total_pages: 0,
            has_next: false,
            has_previous: false,
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaphorDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub version: i64,
}

impl From<Metaphor> for MetaphorDto {
    fn from(metaphor: Metaphor) -> Self {
        Self {
            id: metaphor.id().value().to_string(),
            name: metaphor.name().to_string(),
            description: metaphor.description().to_string(),
            status: metaphor.status().to_string(),
            tags: metaphor.tags().clone(),
            metadata: metaphor.metadata().to_map(),
            created_by: metaphor.created_by().to_string(),
            created_at: *metaphor.created_at(),
            updated_at: *metaphor.updated_at(),
            deleted_at: metaphor.deleted_at().map(|dt| *dt),
            version: metaphor.version().value(),
        }
    }
}

// Query Handler Traits
#[async_trait]
pub trait GetMetaphorHandler: Send + Sync {
    async fn handle(&self, query: GetMetaphorQuery) -> DomainResult<GetMetaphorResponse>;
}

#[async_trait]
pub trait ListMetaphorsHandler: Send + Sync {
    async fn handle(&self, query: ListMetaphorsQuery) -> DomainResult<ListMetaphorsResponse>;
}

#[async_trait]
pub trait SearchMetaphorsHandler: Send + Sync {
    async fn handle(&self, query: SearchMetaphorsQuery) -> DomainResult<ListMetaphorsResponse>;
}

// Default Query Handler Implementations
pub struct DefaultGetMetaphorHandler {
    repository: Box<dyn MetaphorRepository>,
}

impl DefaultGetMetaphorHandler {
    pub fn new(repository: Box<dyn MetaphorRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl GetMetaphorHandler for DefaultGetMetaphorHandler {
    async fn handle(&self, query: GetMetaphorQuery) -> DomainResult<GetMetaphorResponse> {
        // Validate query
        query.validate()?;

        let metaphor_id = MetaphorId::new(&query.id)
            .map_err(|e| DomainError::ValidationError { message: e.to_string() })?;

        // Fetch from repository
        match self.repository.find_by_id(&metaphor_id).await {
            Ok(Some(metaphor)) => Ok(GetMetaphorResponse::success(metaphor)),
            Ok(None) => Ok(GetMetaphorResponse::not_found(query.id)),
            Err(e) => Err(DomainError::from(e)),
        }
    }
}

pub struct DefaultListMetaphorsHandler {
    repository: Box<dyn MetaphorRepository>,
}

impl DefaultListMetaphorsHandler {
    pub fn new(repository: Box<dyn MetaphorRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ListMetaphorsHandler for DefaultListMetaphorsHandler {
    async fn handle(&self, query: ListMetaphorsQuery) -> DomainResult<ListMetaphorsResponse> {
        // Validate query
        query.validate()?;

        let pagination = query.get_pagination_params();
        let sort = query.get_sort_params();

        // Fetch from repository
        let result = if let Some(filters) = &query.filters {
            filters.validate()?;
            let repo_filters = filters.to_repository_filters();
            self.repository
                .find_with_filters(repo_filters, pagination, sort)
                .await
        } else {
            self.repository.find_all(pagination, sort).await
        };

        match result {
            Ok(paginated_result) => Ok(ListMetaphorsResponse::success(paginated_result)),
            Err(e) => Err(DomainError::from(e)),
        }
    }
}

pub struct DefaultSearchMetaphorsHandler {
    repository: Box<dyn MetaphorRepository>,
}

impl DefaultSearchMetaphorsHandler {
    pub fn new(repository: Box<dyn MetaphorRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SearchMetaphorsHandler for DefaultSearchMetaphorsHandler {
    async fn handle(&self, query: SearchMetaphorsQuery) -> DomainResult<ListMetaphorsResponse> {
        // Validate query
        query.validate()?;

        let pagination = query.get_pagination_params();
        let sort = query.get_sort_params();

        // Search in repository
        match self.repository.search(&query.query, pagination, sort).await {
            Ok(paginated_result) => Ok(ListMetaphorsResponse::success(paginated_result)),
            Err(e) => Err(DomainError::from(e)),
        }
    }
}

// Handler Factory
pub struct MetaphorQueryHandlerFactory;

impl MetaphorQueryHandlerFactory {
    pub fn create_get_handler(
        repository: Box<dyn MetaphorRepository>,
    ) -> Box<dyn GetMetaphorHandler> {
        Box::new(DefaultGetMetaphorHandler::new(repository))
    }

    pub fn create_list_handler(
        repository: Box<dyn MetaphorRepository>,
    ) -> Box<dyn ListMetaphorsHandler> {
        Box::new(DefaultListMetaphorsHandler::new(repository))
    }

    pub fn create_search_handler(
        repository: Box<dyn MetaphorRepository>,
    ) -> Box<dyn SearchMetaphorsHandler> {
        Box::new(DefaultSearchMetaphorsHandler::new(repository))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::PaginationParams;
    use crate::domain::value_objects::{MetaphorName, Metadata};
    use async_trait::async_trait;

    // Mock repository for testing
    struct MockRepository {
        should_fail: bool,
        should_return_none: bool,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                should_fail: false,
                should_return_none: false,
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn with_none(mut self) -> Self {
            self.should_return_none = true;
            self
        }
    }

    #[async_trait]
    impl MetaphorRepository for MockRepository {
        async fn save(&self, _metaphor: &Metaphor) -> crate::domain::repositories::RepositoryResult<()> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &MetaphorId) -> crate::domain::repositories::RepositoryResult<Option<Metaphor>> {
            if self.should_fail {
                Err(crate::domain::repositories::RepositoryError::DatabaseError {
                    message: "Database error".to_string(),
                })
            } else if self.should_return_none {
                Ok(None)
            } else {
                // Return a test metaphor
                let metaphor = Metaphor::create(
                    MetaphorName::new("Test Metaphor").unwrap(),
                    "Test Description".to_string(),
                    vec!["test".to_string()],
                    Metadata::new(),
                    "test_user".to_string(),
                ).unwrap();
                Ok(Some(metaphor))
            }
        }

        async fn delete(&self, _id: &MetaphorId, _hard_delete: bool) -> crate::domain::repositories::RepositoryResult<()> {
            Ok(())
        }

        async fn find_all(
            &self,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_with_filters(
            &self,
            _filters: crate::domain::repositories::MetaphorFilters,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_status(
            &self,
            _status: MetaphorStatus,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_tags(
            &self,
            _tags: Vec<String>,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_created_by(
            &self,
            _created_by: &str,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn search(
            &self,
            _query: &str,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn save_batch(&self, _metaphors: &[Metaphor]) -> crate::domain::repositories::RepositoryResult<()> {
            Ok(())
        }

        async fn delete_batch(&self, _ids: &[MetaphorId], _hard_delete: bool) -> crate::domain::repositories::RepositoryResult<()> {
            Ok(())
        }

        async fn exists(&self, _id: &MetaphorId) -> crate::domain::repositories::RepositoryResult<bool> {
            Ok(false)
        }

        async fn count(&self, _filters: Option<crate::domain::repositories::MetaphorFilters>) -> crate::domain::repositories::RepositoryResult<u64> {
            Ok(0)
        }

        async fn find_deleted(
            &self,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn restore(&self, _id: &MetaphorId) -> crate::domain::repositories::RepositoryResult<()> {
            Ok(())
        }

        async fn find_by_metadata(
            &self,
            _metadata_key: &str,
            _metadata_value: Option<&str>,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_date_range(
            &self,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
            _date_field: crate::domain::repositories::SortField,
            _pagination: PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn get_status_counts(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<MetaphorStatus, u64>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_tag_counts(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<String, u64>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_recently_created(&self, _days: i64, _limit: Option<usize>) -> crate::domain::repositories::RepositoryResult<Vec<Metaphor>> {
            Ok(Vec::new())
        }

        async fn health_check(&self) -> crate::domain::repositories::RepositoryResult<bool> {
            Ok(true)
        }

        async fn connection_pool_status(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<String, serde_json::Value>> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[tokio::test]
    async fn test_get_metaphor_query_validation() {
        // Valid query
        let valid_query = GetMetaphorQuery::new("123e4567-e89b-12d3-a456-426614174000".to_string());
        assert!(valid_query.validate().is_ok());

        // Invalid query - empty ID
        let invalid_query = GetMetaphorQuery::new("".to_string());
        assert!(invalid_query.validate().is_err());

        // Invalid query - bad format
        let invalid_query = GetMetaphorQuery::new("invalid-uuid".to_string());
        assert!(invalid_query.validate().is_err());
    }

    #[tokio::test]
    async fn test_get_metaphor_handler_success() {
        let repository = Box::new(MockRepository::new());
        let handler = DefaultGetMetaphorHandler::new(repository);

        let query = GetMetaphorQuery::new("123e4567-e89b-12d3-a456-426614174000".to_string());
        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert!(response.metaphor.is_some());
    }

    #[tokio::test]
    async fn test_get_metaphor_handler_not_found() {
        let repository = Box::new(MockRepository::new().with_none());
        let handler = DefaultGetMetaphorHandler::new(repository);

        let query = GetMetaphorQuery::new("123e4567-e89b-12d3-a456-426614174000".to_string());
        let result = handler.handle(query).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.metaphor.is_none());
        assert!(response.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_list_metaphors_query_validation() {
        // Valid query
        let valid_query = ListMetaphorsQuery::new()
            .with_pagination(1, 20)
            .with_sort("name".to_string(), "asc".to_string());
        assert!(valid_query.validate().is_ok());

        // Invalid query - page 0
        let invalid_query = ListMetaphorsQuery::new().with_pagination(0, 20);
        assert!(invalid_query.validate().is_err());

        // Invalid query - page size 0
        let invalid_query = ListMetaphorsQuery::new().with_pagination(1, 0);
        assert!(invalid_query.validate().is_err());

        // Invalid query - page size too large
        let invalid_query = ListMetaphorsQuery::new().with_pagination(1, 101);
        assert!(invalid_query.validate().is_err());

        // Invalid sort field
        let invalid_query = ListMetaphorsQuery::new()
            .with_sort("invalid_field".to_string(), "asc".to_string());
        assert!(invalid_query.validate().is_err());

        // Invalid sort direction
        let invalid_query = ListMetaphorsQuery::new()
            .with_sort("name".to_string(), "invalid".to_string());
        assert!(invalid_query.validate().is_err());
    }

    #[tokio::test]
    async fn test_metaphor_filters_validation() {
        // Valid filters
        let valid_filters = MetaphorFilters::new()
            .with_status("ACTIVE".to_string())
            .with_tags(vec!["test".to_string()])
            .with_created_by("user".to_string());
        assert!(valid_filters.validate().is_ok());

        // Invalid status
        let invalid_filters = MetaphorFilters::new().with_status("INVALID".to_string());
        assert!(invalid_filters.validate().is_err());

        // Too many tags
        let too_many_tags = (0..51).map(|i| format!("tag{}", i)).collect();
        let invalid_filters = MetaphorFilters::new().with_tags(too_many_tags);
        assert!(invalid_filters.validate().is_err());

        // Empty tag
        let invalid_filters = MetaphorFilters::new().with_tags(vec!["".to_string()]);
        assert!(invalid_filters.validate().is_err());

        // Invalid date format
        let invalid_filters = MetaphorFilters::new()
            .with_date_range(Some("invalid-date".to_string()), None);
        assert!(invalid_filters.validate().is_err());
    }

    #[tokio::test]
    async fn test_search_metaphors_query_validation() {
        // Valid query
        let valid_query = SearchMetaphorsQuery::new("test".to_string())
            .with_pagination(1, 20);
        assert!(valid_query.validate().is_ok());

        // Empty query
        let invalid_query = SearchMetaphorsQuery::new("".to_string());
        assert!(invalid_query.validate().is_err());

        // Query too long
        let long_query = "a".repeat(1001);
        let invalid_query = SearchMetaphorsQuery::new(long_query);
        assert!(invalid_query.validate().is_err());
    }

    #[tokio::test]
    async fn test_metaphor_dto_conversion() {
        let metaphor = Metaphor::create(
            MetaphorName::new("Test Metaphor").unwrap(),
            "Test Description".to_string(),
            vec!["test".to_string()],
            {
                let mut metadata = Metadata::new();
                metadata.insert("env".to_string(), "test".to_string()).unwrap();
                metadata
            },
            "test_user".to_string(),
        ).unwrap();

        let dto = MetaphorDto::from(metaphor);

        assert_eq!(dto.name, "Test Metaphor");
        assert_eq!(dto.description, "Test Description");
        assert_eq!(dto.status, "ACTIVE");
        assert_eq!(dto.tags, vec!["test"]);
        assert_eq!(dto.created_by, "test_user");
        assert!(dto.metadata.contains_key("env"));
    }
}