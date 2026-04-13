// Metaphor HTTP Handlers
// HTTP REST API handlers for Metaphor operations

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::application::{
    ApplicationServices, CreateMetaphorCommand, CreateMetaphorResponse, GetMetaphorQuery,
    GetMetaphorResponse, ListMetaphorsQuery, ListMetaphorsResponse, SearchMetaphorsQuery,
};
use crate::domain::repositories::{PaginationParams, SortParams};
use crate::domain::value_objects::MetaphorStatus;

// HTTP Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateMetaphorRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl From<CreateMetaphorRequest> for CreateMetaphorCommand {
    fn from(request: CreateMetaphorRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            tags: request.tags,
            metadata: request.metadata,
            created_by: "system".to_string(), // This should come from authentication context
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMetaphorsRequest {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub search: Option<String>,
}

impl From<ListMetaphorsRequest> for ListMetaphorsQuery {
    fn from(request: ListMetaphorsRequest) -> Self {
        let mut filters = None;

        if request.status.is_some() || request.tags.is_some() || request.created_by.is_some() {
            let mut metaphor_filters = crate::application::MetaphorFilters::new();

            if let Some(status) = request.status {
                metaphor_filters = metaphor_filters.with_status(status);
            }

            if let Some(tags) = request.tags {
                metaphor_filters = metaphor_filters.with_tags(tags);
            }

            if let Some(created_by) = request.created_by {
                metaphor_filters = metaphor_filters.with_created_by(created_by);
            }

            filters = Some(metaphor_filters);
        }

        let mut query = Self::new()
            .with_pagination(
                request.page.unwrap_or(1),
                request.page_size.unwrap_or(20),
            );

        if let Some(sort_by) = request.sort_by {
            if let Some(sort_direction) = request.sort_direction {
                query = query.with_sort(sort_by, sort_direction);
            }
        }

        if let Some(filters) = filters {
            query = query.with_filters(filters);
        }

        query
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchMetaphorsRequest {
    pub query: String,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl From<SearchMetaphorsRequest> for SearchMetaphorsQuery {
    fn from(request: SearchMetaphorsRequest) -> Self {
        let mut query = Self::new(request.query);

        query = query.with_pagination(
            request.page.unwrap_or(1),
            request.page_size.unwrap_or(20),
        );

        if let Some(sort_by) = request.sort_by {
            if let Some(sort_direction) = request.sort_direction {
                query = query.with_sort(sort_by, sort_direction);
            }
        }

        query
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMetaphorRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaphorStatusRequest {
    pub status: String,
    pub reason: Option<String>,
}

// HTTP Response DTOs
#[derive(Debug, Serialize)]
pub struct MetaphorListResponse {
    pub data: Vec<crate::application::MetaphorDto>,
    pub pagination: PaginationInfo,
    pub filters: Option<AppliedFilters>,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total: u64,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Debug, Serialize)]
pub struct AppliedFilters {
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApiError {
    pub fn new(error: String, message: String) -> Self {
        Self {
            error,
            message,
            details: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl Responder for ApiError {
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::InternalServerError().json(self)
    }
}

// HTTP Handlers
pub struct MetaphorHttpHandler {
    services: ApplicationServices,
}

impl MetaphorHttpHandler {
    pub fn new(services: ApplicationServices) -> Self {
        Self { services }
    }

    // Create a new Metaphor
    pub async fn create_metaphor(
        &self,
        request: web::Json<CreateMetaphorRequest>,
    ) -> Result<HttpResponse, ApiError> {
        let command = CreateMetaphorCommand::from(request.into_inner());

        let handler = self.services.create_metaphor_handler();

        match self.services.execute_command(command, handler).await {
            Ok(response) => {
                if response.success {
                    Ok(HttpResponse::Created().json(response))
                } else {
                    Err(ApiError::new(
                        "CREATION_FAILED".to_string(),
                        response.message,
                    ))
                }
            }
            Err(e) => Err(ApiError::new(
                "COMMAND_ERROR".to_string(),
                e.to_string(),
            )),
        }
    }

    // Get a Metaphor by ID
    pub async fn get_metaphor(
        &self,
        path: web::Path<String>,
    ) -> Result<HttpResponse, ApiError> {
        let query = GetMetaphorQuery::new(path.into_inner());

        let handler = self.services.get_metaphor_handler();

        match self.services.execute_query(query, handler).await {
            Ok(response) => {
                if response.success {
                    Ok(HttpResponse::Ok().json(response))
                } else {
                    Ok(HttpResponse::NotFound().json(response))
                }
            }
            Err(e) => Err(ApiError::new(
                "QUERY_ERROR".to_string(),
                e.to_string(),
            )),
        }
    }

    // List Metaphors with optional filters
    pub async fn list_metaphors(
        &self,
        query: web::Query<ListMetaphorsRequest>,
    ) -> Result<HttpResponse, ApiError> {
        let list_query = ListMetaphorsQuery::from(query.into_inner());

        let handler = self.services.list_metaphors_handler();

        match self.services.execute_query(list_query, handler).await {
            Ok(response) => {
                let metaphor_response = MetaphorListResponse {
                    data: response.metaphors,
                    pagination: PaginationInfo {
                        page: response.page,
                        page_size: response.page_size,
                        total: response.total,
                        total_pages: response.total_pages,
                        has_next: response.has_next,
                        has_previous: response.has_previous,
                    },
                    filters: None, // TODO: Extract filters from query
                };

                Ok(HttpResponse::Ok().json(metaphor_response))
            }
            Err(e) => Err(ApiError::new(
                "QUERY_ERROR".to_string(),
                e.to_string(),
            )),
        }
    }

    // Search Metaphors
    pub async fn search_metaphors(
        &self,
        query: web::Query<SearchMetaphorsRequest>,
    ) -> Result<HttpResponse, ApiError> {
        let search_query = SearchMetaphorsQuery::from(query.into_inner());

        let handler = self.services.search_metaphors_handler();

        match self.services.execute_query(search_query, handler).await {
            Ok(response) => {
                let metaphor_response = MetaphorListResponse {
                    data: response.metaphors,
                    pagination: PaginationInfo {
                        page: response.page,
                        page_size: response.page_size,
                        total: response.total,
                        total_pages: response.total_pages,
                        has_next: response.has_next,
                        has_previous: response.has_previous,
                    },
                    filters: None,
                };

                Ok(HttpResponse::Ok().json(metaphor_response))
            }
            Err(e) => Err(ApiError::new(
                "SEARCH_ERROR".to_string(),
                e.to_string(),
            )),
        }
    }

    // Update a Metaphor
    pub async fn update_metaphor(
        &self,
        path: web::Path<String>,
        request: web::Json<UpdateMetaphorRequest>,
    ) -> Result<HttpResponse, ApiError> {
        let id = path.into_inner();

        // For now, this is a placeholder implementation
        // In a complete implementation, you would create UpdateMetaphorCommand and UpdateMetaphorHandler

        Err(ApiError::new(
            "NOT_IMPLEMENTED".to_string(),
            "Update functionality not yet implemented".to_string(),
        ))
    }

    // Delete a Metaphor
    pub async fn delete_metaphor(
        &self,
        path: web::Path<String>,
    ) -> Result<HttpResponse, ApiError> {
        let id = path.into_inner();

        // For now, this is a placeholder implementation
        // In a complete implementation, you would create DeleteMetaphorCommand and DeleteMetaphorHandler

        Ok(HttpResponse::NoContent().finish())
    }

    // Update Metaphor status
    pub async fn update_metaphor_status(
        &self,
        path: web::Path<String>,
        request: web::Json<MetaphorStatusRequest>,
    ) -> Result<HttpResponse, ApiError> {
        let id = path.into_inner();
        let new_status = request.status.clone();

        // For now, this is a placeholder implementation
        // In a complete implementation, you would create UpdateStatusCommand and handler

        Err(ApiError::new(
            "NOT_IMPLEMENTED".to_string(),
            format!("Update status functionality not yet implemented for status: {}", new_status),
        ))
    }

    // Get Metaphor statistics
    pub async fn get_metaphor_stats(&self) -> Result<HttpResponse, ApiError> {
        // For now, this is a placeholder implementation
        // In a complete implementation, you would create StatsQuery and handler

        let stats = serde_json::json!({
            "total_metaphors": 0,
            "by_status": {
                "ACTIVE": 0,
                "INACTIVE": 0,
                "SUSPENDED": 0,
                "ARCHIVED": 0
            },
            "by_tags": {},
            "recently_created": []
        });

        Ok(HttpResponse::Ok().json(stats))
    }

    // Health check endpoint
    pub async fn health_check(&self) -> Result<HttpResponse, ApiError> {
        match self.services.health_check().await {
            Ok(health_status) => {
                let status_code = match health_status.status {
                    crate::infrastructure::health::HealthStatus::Healthy => actix_web::http::StatusCode::OK,
                    crate::infrastructure::health::HealthStatus::Degraded => {
                        actix_web::http::StatusCode::OK // Still return 200 but with degraded status
                    }
                    crate::infrastructure::health::HealthStatus::Unhealthy => {
                        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
                    }
                    crate::infrastructure::health::HealthStatus::Unknown => {
                        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
                    }
                };

                Ok(HttpResponse::build(status_code).json(health_status))
            }
            Err(e) => Err(ApiError::new(
                "HEALTH_CHECK_ERROR".to_string(),
                e.to_string(),
            )),
        }
    }

    // Get API information
    pub async fn api_info(&self) -> Result<HttpResponse, ApiError> {
        let info = serde_json::json!({
            "name": "Metaphor API",
            "version": "1.0.0",
            "description": "Metaphor bounded context REST API",
            "endpoints": {
                "create": "POST /api/v1/metaphors",
                "get": "GET /api/v1/metaphors/{id}",
                "list": "GET /api/v1/metaphors",
                "search": "GET /api/v1/metaphors/search",
                "update": "PUT /api/v1/metaphors/{id}",
                "delete": "DELETE /api/v1/metaphors/{id}",
                "status": "PATCH /api/v1/metaphors/{id}/status",
                "stats": "GET /api/v1/metaphors/stats",
                "health": "GET /health",
                "info": "GET /api/v1/info"
            },
            "documentation": "/swagger-ui",
            "features": [
                "CQRS pattern",
                "Domain-driven design",
                "Clean architecture",
                "Event sourcing",
                "PostgreSQL persistence"
            ]
        });

        Ok(HttpResponse::Ok().json(info))
    }
}

// Helper functions for routing
pub fn configure_metaphor_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/metaphors")
            .route("", web::post().to(create_metaphor))
            .route("", web::get().to(list_metaphors))
            .route("/search", web::get().to(search_metaphors))
            .route("/stats", web::get().to(get_metaphor_stats))
            .route("/{id}", web::get().to(get_metaphor))
            .route("/{id}", web::put().to(update_metaphor))
            .route("/{id}", web::delete().to(delete_metaphor))
            .route("/{id}/status", web::patch().to(update_metaphor_status)),
    );

    cfg.route("/health", web::get().to(health_check));
    cfg.route("/api/v1/info", web::get().to(api_info));
}

// Actix-web handler functions that use the application services
async fn create_metaphor(
    handler: web::Data<MetaphorHttpHandler>,
    request: web::Json<CreateMetaphorRequest>,
) -> Result<HttpResponse, ApiError> {
    handler.create_metaphor(request).await
}

async fn get_metaphor(
    handler: web::Data<MetaphorHttpHandler>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    handler.get_metaphor(path).await
}

async fn list_metaphors(
    handler: web::Data<MetaphorHttpHandler>,
    query: web::Query<ListMetaphorsRequest>,
) -> Result<HttpResponse, ApiError> {
    handler.list_metaphors(query).await
}

async fn search_metaphors(
    handler: web::Data<MetaphorHttpHandler>,
    query: web::Query<SearchMetaphorsRequest>,
) -> Result<HttpResponse, ApiError> {
    handler.search_metaphors(query).await
}

async fn update_metaphor(
    handler: web::Data<MetaphorHttpHandler>,
    path: web::Path<String>,
    request: web::Json<UpdateMetaphorRequest>,
) -> Result<HttpResponse, ApiError> {
    handler.update_metaphor(path, request).await
}

async fn delete_metaphor(
    handler: web::Data<MetaphorHttpHandler>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    handler.delete_metaphor(path).await
}

async fn update_metaphor_status(
    handler: web::Data<MetaphorHttpHandler>,
    path: web::Path<String>,
    request: web::Json<MetaphorStatusRequest>,
) -> Result<HttpResponse, ApiError> {
    handler.update_metaphor_status(path, request).await
}

async fn get_metaphor_stats(
    handler: web::Data<MetaphorHttpHandler>,
) -> Result<HttpResponse, ApiError> {
    handler.get_metaphor_stats().await
}

async fn health_check(
    handler: web::Data<MetaphorHttpHandler>,
) -> Result<HttpResponse, ApiError> {
    handler.health_check().await
}

async fn api_info(
    handler: web::Data<MetaphorHttpHandler>,
) -> Result<HttpResponse, ApiError> {
    handler.api_info().await
}

// Middleware for error handling
pub async fn error_handler(
    err: actix_web::Error,
    _req: &actix_web::HttpRequest,
) -> HttpResponse {
    let api_error = ApiError::new(
        "INTERNAL_ERROR".to_string(),
        err.to_string(),
    );

    HttpResponse::InternalServerError().json(api_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;

    fn create_test_handler() -> MetaphorHttpHandler {
        // This would normally use real ApplicationServices
        // For testing, we'll create a mock implementation
        use crate::domain::repositories::MetaphorRepository;
        use async_trait::async_trait;

        struct MockRepository;

        #[async_trait]
        impl MetaphorRepository for MockRepository {
            async fn save(&self, _metaphor: &crate::domain::entities::Metaphor) -> crate::domain::repositories::RepositoryResult<()> {
                Ok(())
            }

            async fn find_by_id(&self, _id: &crate::domain::value_objects::MetaphorId) -> crate::domain::repositories::RepositoryResult<Option<crate::domain::entities::Metaphor>> {
                Ok(None)
            }

            async fn delete(&self, _id: &crate::domain::value_objects::MetaphorId, _hard_delete: bool) -> crate::domain::repositories::RepositoryResult<()> {
                Ok(())
            }

            async fn find_all(
                &self,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn find_with_filters(
                &self,
                _filters: crate::domain::repositories::MetaphorFilters,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn find_by_status(
                &self,
                _status: crate::domain::value_objects::MetaphorStatus,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn find_by_tags(
                &self,
                _tags: Vec<String>,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn find_by_created_by(
                &self,
                _created_by: &str,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn search(
                &self,
                _query: &str,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn save_batch(&self, _metaphors: &[crate::domain::entities::Metaphor]) -> crate::domain::repositories::RepositoryResult<()> {
                Ok(())
            }

            async fn delete_batch(&self, _ids: &[crate::domain::value_objects::MetaphorId], _hard_delete: bool) -> crate::domain::repositories::RepositoryResult<()> {
                Ok(())
            }

            async fn exists(&self, _id: &crate::domain::value_objects::MetaphorId) -> crate::domain::repositories::RepositoryResult<bool> {
                Ok(false)
            }

            async fn count(&self, _filters: Option<crate::domain::repositories::MetaphorFilters>) -> crate::domain::repositories::RepositoryResult<u64> {
                Ok(0)
            }

            async fn find_deleted(
                &self,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn restore(&self, _id: &crate::domain::value_objects::MetaphorId) -> crate::domain::repositories::RepositoryResult<()> {
                Ok(())
            }

            async fn find_by_metadata(
                &self,
                _metadata_key: &str,
                _metadata_value: Option<&str>,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn find_by_date_range(
                &self,
                _start_date: chrono::DateTime<chrono::Utc>,
                _end_date: chrono::DateTime<chrono::Utc>,
                _date_field: crate::domain::repositories::SortField,
                _pagination: crate::domain::repositories::PaginationParams,
                _sort: crate::domain::repositories::SortParams,
            ) -> crate::domain::repositories::RepositoryResult<crate::domain::repositories::PaginatedResult<crate::domain::entities::Metaphor>> {
                Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
            }

            async fn get_status_counts(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<crate::domain::value_objects::MetaphorStatus, u64>> {
                Ok(std::collections::HashMap::new())
            }

            async fn get_tag_counts(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<String, u64>> {
                Ok(std::collections::HashMap::new())
            }

            async fn get_recently_created(&self, _days: i64, _limit: Option<usize>) -> crate::domain::repositories::RepositoryResult<Vec<crate::domain::entities::Metaphor>> {
                Ok(Vec::new())
            }

            async fn health_check(&self) -> crate::domain::repositories::RepositoryResult<bool> {
                Ok(true)
            }

            async fn connection_pool_status(&self) -> crate::domain::repositories::RepositoryResult<std::collections::HashMap<String, serde_json::Value>> {
                Ok(std::collections::HashMap::new())
            }
        }

        // Create minimal services for testing
        let repository = Box::new(MockRepository);
        let services = ApplicationServices::builder()
            .with_repository(repository)
            .build()
            .unwrap();

        MetaphorHttpHandler::new(services)
    }

    #[actix_web::test]
    async fn test_create_metaphor_request_conversion() {
        let request = CreateMetaphorRequest {
            name: "Test Metaphor".to_string(),
            description: "Test Description".to_string(),
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
        };

        let command = CreateMetaphorCommand::from(request);

        assert_eq!(command.name, "Test Metaphor");
        assert_eq!(command.description, "Test Description");
        assert_eq!(command.tags, vec!["test"]);
        assert!(command.metadata.is_empty());
    }

    #[actix_web::test]
    async fn test_api_error_creation() {
        let error = ApiError::new(
            "TEST_ERROR".to_string(),
            "Test error message".to_string(),
        );

        assert_eq!(error.error, "TEST_ERROR");
        assert_eq!(error.message, "Test error message");
        assert!(error.details.is_none());
    }

    #[actix_web::test]
    async fn test_health_check_endpoint() {
        let handler = create_test_handler();
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/health", web::get().to(health_check)),
        )
        .await;

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_api_info_endpoint() {
        let handler = create_test_handler();
        let req = test::TestRequest::get()
            .uri("/api/v1/info")
            .to_request();

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/api/v1/info", web::get().to(api_info)),
        )
        .await;

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_list_metaphors_endpoint() {
        let handler = create_test_handler();
        let req = test::TestRequest::get()
            .uri("/api/v1/metaphors?page=1&page_size=10")
            .to_request();

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/api/v1/metaphors", web::get().to(list_metaphors)),
        )
        .await;

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_search_metaphors_endpoint() {
        let handler = create_test_handler();
        let req = test::TestRequest::get()
            .uri("/api/v1/metaphors/search?query=test&page=1&page_size=10")
            .to_request();

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/api/v1/metaphors/search", web::get().to(search_metaphors)),
        )
        .await;

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_metaphor_endpoint_not_found() {
        let handler = create_test_handler();
        let req = test::TestRequest::get()
            .uri("/api/v1/metaphors/non-existent-id")
            .to_request();

        let app = test::init_service(
            App::new()
                .app_data(handler)
                .route("/api/v1/metaphors/{id}", web::get().to(get_metaphor)),
        )
        .await;

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }
}