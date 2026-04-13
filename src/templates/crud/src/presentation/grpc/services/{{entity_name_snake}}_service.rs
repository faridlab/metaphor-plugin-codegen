// {{PascalCaseEntity}} gRPC Service
// Complete gRPC service implementation for {{PascalCaseEntity}} CRUD operations

use anyhow::Result;
use tonic::{Request, Response, Status};

use crate::application::{
    commands::{
        Create{{PascalCaseEntity}}Command, Update{{PascalCaseEntity}}Command, Delete{{PascalCaseEntity}}Command,
        BulkCreate{{PascalCaseEntity}}Command, Upsert{{PascalCaseEntity}}Command, Restore{{PascalCaseEntity}}Command,
        EmptyTrashCommand, {{PascalCaseEntity}}Dto, {{PascalCaseEntity}}Filters,
    },
    queries::{
        Get{{PascalCaseEntity}}Query, List{{PascalCaseEntity}}Query, ListDeleted{{PascalCaseEntity}}Query,
    },
    {{entity_name_snake}}_services::{{PascalCaseEntity}}ApplicationServices,
};

// Generated gRPC types - these should be imported from your proto-generated code
// use your_protos::{{entity_name_snake}}_service_server::{{PascalCaseEntity}}Service;
// use your_protos::{
//     Create{{PascalCaseEntity}}Request, Create{{PascalCaseEntity}}Response,
//     Update{{PascalCaseEntity}}Request, Update{{PascalCaseEntity}}Response,
//     Delete{{PascalCaseEntity}}Request, Delete{{PascalCaseEntity}}Response,
//     Get{{PascalCaseEntity}}Request, Get{{PascalCaseEntity}}Response,
//     List{{PascalCaseEntity}}Request, List{{PascalCaseEntity}}Response,
//     BulkCreate{{PascalCaseEntity}}Request, BulkCreate{{PascalCaseEntity}}Response,
//     Upsert{{PascalCaseEntity}}Request, Upsert{{PascalCaseEntity}}Response,
//     Restore{{PascalCaseEntity}}Request, Restore{{PascalCaseEntity}}Response,
//     EmptyTrashRequest, EmptyTrashResponse,
// };

pub struct {{PascalCaseEntity}}GrpcService {
    services: {{PascalCaseEntity}}ApplicationServices,
}

impl {{PascalCaseEntity}}GrpcService {
    pub fn new(services: {{PascalCaseEntity}}ApplicationServices) -> Self {
        Self { services }
    }
}

// Example gRPC service implementation - uncomment and adapt to your proto definitions
/*
#[tonic::async_trait]
impl {{PascalCaseEntity}}Service for {{PascalCaseEntity}}GrpcService {
    async fn create_{{entity_name_snake}}(
        &self,
        request: Request<Create{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Create{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        // Convert gRPC request to application command
        let command = Create{{PascalCaseEntity}}Command {
            // TODO: Map gRPC request fields to command fields
            custom_fields: std::collections::HashMap::new(),
            created_by: req.user_id.unwrap_or_default(),
        };

        match self.services.create_{{entity_name_snake}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_response = Create{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_snake}}: response.{{entity_name_snake}}
                        .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto)),
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_{{entity_name_snake}}(
        &self,
        request: Request<Get{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Get{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let query = Get{{PascalCaseEntity}}Query {
            id: req.id,
        };

        match self.services.get_{{entity_name_snake}}_handler().handle(query).await {
            Ok(response) => {
                let grpc_response = Get{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_snake}}: response.{{entity_name_snake}}
                        .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto)),
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_{{entity_name_plural}}(
        &self,
        request: Request<List{{PascalCaseEntity}}Request>,
    ) -> Result<Response<List{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        // Build filters from gRPC request
        let filters = if req.has_filters() {
            Some(convert_grpc_filters_to_application(req.get_filters()))
        } else {
            None
        };

        let query = List{{PascalCaseEntity}}Query {
            page: req.page as usize,
            page_size: req.page_size as usize,
            sort_by: req.sort_by,
            sort_direction: req.sort_direction,
            filters,
        };

        match self.services.list_{{entity_name_plural}}_handler().handle(query).await {
            Ok(response) => {
                let grpc_{{entity_name_plural}}: Vec<_> = response.{{entity_name_plural}}
                    .into_iter()
                    .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto))
                    .collect();

                let grpc_response = List{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_plural}}: grpc_{{entity_name_plural}},
                    page: response.page as u32,
                    page_size: response.page_size as u32,
                    total: response.total,
                    total_pages: response.total_pages as u32,
                    has_next: response.has_next,
                    has_previous: response.has_previous,
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_{{entity_name_snake}}(
        &self,
        request: Request<Update{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Update{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let command = Update{{PascalCaseEntity}}Command {
            id: req.id,
            // TODO: Map gRPC request fields to command fields
            custom_fields: std::collections::HashMap::new(),
            updated_by: req.user_id.unwrap_or_default(),
        };

        match self.services.update_{{entity_name_snake}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_response = Update{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_snake}}: response.{{entity_name_snake}}
                        .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto)),
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_{{entity_name_snake}}(
        &self,
        request: Request<Delete{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Delete{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let command = Delete{{PascalCaseEntity}}Command {
            id: req.id,
            deleted_by: req.user_id.unwrap_or_default(),
        };

        match self.services.delete_{{entity_name_snake}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_response = Delete{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn bulk_create_{{entity_name_plural}}(
        &self,
        request: Request<BulkCreate{{PascalCaseEntity}}Request>,
    ) -> Result<Response<BulkCreate{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let commands: Vec<Create{{PascalCaseEntity}}Command> = req.{{entity_name_plural}}
            .into_iter()
            .map(|grpc_item| {
                // TODO: Convert gRPC items to commands
                Create{{PascalCaseEntity}}Command {
                    custom_fields: std::collections::HashMap::new(),
                    created_by: req.user_id.unwrap_or_default(),
                }
            })
            .collect();

        let command = BulkCreate{{PascalCaseEntity}}Command {
            {{entity_name_plural}}: commands,
        };

        match self.services.bulk_create_{{entity_name_plural}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_{{entity_name_plural}}: Vec<_> = response.{{entity_name_plural}}
                    .into_iter()
                    .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto))
                    .collect();

                let grpc_response = BulkCreate{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_plural}}: grpc_{{entity_name_plural}},
                    created_count: response.created_count as u32,
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn upsert_{{entity_name_snake}}(
        &self,
        request: Request<Upsert{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Upsert{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let command = Upsert{{PascalCaseEntity}}Command {
            // TODO: Map gRPC request fields to command fields
            custom_fields: std::collections::HashMap::new(),
            user_id: req.user_id.unwrap_or_default(),
        };

        match self.services.upsert_{{entity_name_snake}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_{{entity_name_plural}}: Vec<_> = response.{{entity_name_plural}}
                    .into_iter()
                    .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto))
                    .collect();

                let grpc_response = Upsert{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_plural}}: grpc_{{entity_name_plural}},
                    created: response.created,
                    updated: response.updated,
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn restore_{{entity_name_snake}}(
        &self,
        request: Request<Restore{{PascalCaseEntity}}Request>,
    ) -> Result<Response<Restore{{PascalCaseEntity}}Response>, Status> {
        let req = request.into_inner();

        let command = Restore{{PascalCaseEntity}}Command {
            id: req.id,
            restored_by: req.user_id.unwrap_or_default(),
        };

        match self.services.restore_{{entity_name_snake}}_handler().handle(command).await {
            Ok(response) => {
                let grpc_response = Restore{{PascalCaseEntity}}Response {
                    success: response.success,
                    message: response.message,
                    {{entity_name_snake}}: response.{{entity_name_snake}}
                        .map(|dto| convert_{{entity_name_snake}}_dto_to_grpc(dto)),
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn empty_{{entity_name_snake}}_trash(
        &self,
        request: Request<EmptyTrashRequest>,
    ) -> Result<Response<EmptyTrashResponse>, Status> {
        let req = request.into_inner();

        let command = EmptyTrashCommand {
            user_id: req.user_id.unwrap_or_default(),
        };

        match self.services.empty_{{entity_name_snake}}_trash_handler().handle(command).await {
            Ok(response) => {
                let grpc_response = EmptyTrashResponse {
                    success: response.success,
                    message: response.message,
                    deleted_count: response.deleted_count as u32,
                };
                Ok(Response::new(grpc_response))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
*/

// Helper functions to convert between application DTOs and gRPC messages
// TODO: Implement these conversion functions based on your proto definitions

/*
fn convert_{{entity_name_snake}}_dto_to_grpc(dto: {{PascalCaseEntity}}Dto) -> {{PascalCaseEntity}}Grpc {
    {{PascalCaseEntity}}Grpc {
        id: dto.id,
        // TODO: Map all DTO fields to gRPC fields
    }
}

fn convert_grpc_filters_to_application(filters: &{{PascalCaseEntity}}GrpcFilters) -> {{PascalCaseEntity}}Filters {
    {{PascalCaseEntity}}Filters::new()
        // TODO: Map gRPC filters to application filters
        .with_search(filters.search.clone())
        // Add other filter mappings
}
*/