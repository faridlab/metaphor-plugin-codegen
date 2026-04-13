//! External Services
//!
//! Infrastructure for integrating with external services and APIs.
//! This includes HTTP clients, messaging systems, and third-party integrations.

use crate::shared::error::AppResult;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub headers: HashMap<String, String>,
    pub auth: Option<AuthConfig>,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub enum AuthConfig {
    Bearer { token: String },
    ApiKey { key: String, header: String },
    Basic { username: String, password: String },
    Custom { headers: HashMap<String, String> },
}

/// HTTP client wrapper for external services
pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> AppResult<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent("Metaphor-Framework/2.0");

        let client = builder.build()?;

        Ok(Self { client, config })
    }

    /// Make GET request
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: Option<HashMap<String, String>>,
    ) -> AppResult<T> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut request = self.client.get(&url);

        // Add query parameters
        if let Some(params) = query_params {
            request = request.query(&params);
        }

        // Add headers
        request = self.add_headers(request);

        let response = request.send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    /// Make POST request
    pub async fn post<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Req,
    ) -> AppResult<Resp> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut request = self.client.post(&url).json(body);

        request = self.add_headers(request);

        let response = request.send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    /// Make PUT request
    pub async fn put<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Req,
    ) -> AppResult<Resp> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut request = self.client.put(&url).json(body);

        request = self.add_headers(request);

        let response = request.send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    /// Make DELETE request
    pub async fn delete<Resp: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> AppResult<Resp> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut request = self.client.delete(&url);

        request = self.add_headers(request);

        let response = request.send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    fn add_headers(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // Add configured headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Add authentication headers
        if let Some(auth) = &self.config.auth {
            match auth {
                AuthConfig::Bearer { token } => {
                    request = request.header("Authorization", format!("Bearer {}", token));
                }
                AuthConfig::ApiKey { key, header } => {
                    request = request.header(header, key);
                }
                AuthConfig::Basic { username, password } => {
                    let credentials = format!("{}:{}", username, password);
                    let encoded = base64::encode(credentials);
                    request = request.header("Authorization", format!("Basic {}", encoded));
                }
                AuthConfig::Custom { headers } => {
                    for (key, value) in headers {
                        request = request.header(key, value);
                    }
                }
            }
        }

        request
    }
}

/// Generic external service client
#[async_trait]
pub trait ExternalServiceClient {
    type Config;
    type Error;

    async fn health_check(&self) -> AppResult<bool>;
    fn service_name(&self) -> &str;
}

/// Notification service client
pub struct NotificationServiceClient {
    http_client: HttpClient,
}

impl NotificationServiceClient {
    pub fn new(config: HttpClientConfig) -> AppResult<Self> {
        let http_client = HttpClient::new(config)?;
        Ok(Self { http_client })
    }

    /// Send email notification
    pub async fn send_email(&self, request: EmailRequest) -> AppResult<EmailResponse> {
        self.http_client
            .post::<EmailRequest, EmailResponse>("/api/v1/email", &request)
            .await
    }

    /// Send SMS notification
    pub async fn send_sms(&self, request: SmsRequest) -> AppResult<SmsResponse> {
        self.http_client
            .post::<SmsRequest, SmsResponse>("/api/v1/sms", &request)
            .await
    }

    /// Send push notification
    pub async fn send_push(&self, request: PushRequest) -> AppResult<PushResponse> {
        self.http_client
            .post::<PushRequest, PushResponse>("/api/v1/push", &request)
            .await
    }
}

#[async_trait]
impl ExternalServiceClient for NotificationServiceClient {
    type Config = HttpClientConfig;
    type Error = crate::shared::error::AppError;

    async fn health_check(&self) -> AppResult<bool> {
        #[derive(Debug, Deserialize)]
        struct HealthResponse {
            status: String,
        }

        let response: HealthResponse = self
            .http_client
            .get("/health", None)
            .await
            .unwrap_or(HealthResponse {
                status: "unhealthy".to_string(),
            });

        Ok(response.status == "healthy")
    }

    fn service_name(&self) -> &str {
        "notification-service"
    }
}

/// Storage service client
pub struct StorageServiceClient {
    http_client: HttpClient,
}

impl StorageServiceClient {
    pub fn new(config: HttpClientConfig) -> AppResult<Self> {
        let http_client = HttpClient::new(config)?;
        Ok(Self { http_client })
    }

    /// Upload file
    pub async fn upload_file(&self, request: FileUploadRequest) -> AppResult<FileUploadResponse> {
        self.http_client
            .post::<FileUploadRequest, FileUploadResponse>("/api/v1/files", &request)
            .await
    }

    /// Download file
    pub async fn download_file(&self, file_id: &str) -> AppResult<FileDownloadResponse> {
        let endpoint = format!("/api/v1/files/{}", file_id);
        self.http_client
            .get::<FileDownloadResponse>(&endpoint, None)
            .await
    }

    /// Delete file
    pub async fn delete_file(&self, file_id: &str) -> AppResult<FileDeleteResponse> {
        let endpoint = format!("/api/v1/files/{}", file_id);
        self.http_client
            .delete::<FileDeleteResponse>(&endpoint)
            .await
    }

    /// List files
    pub async fn list_files(&self, request: FileListRequest) -> AppResult<FileListResponse> {
        self.http_client
            .post::<FileListRequest, FileListResponse>("/api/v1/files/search", &request)
            .await
    }
}

#[async_trait]
impl ExternalServiceClient for StorageServiceClient {
    type Config = HttpClientConfig;
    type Error = crate::shared::error::AppError;

    async fn health_check(&self) -> AppResult<bool> {
        #[derive(Debug, Deserialize)]
        struct HealthResponse {
            status: String,
        }

        let response: HealthResponse = self
            .http_client
            .get("/health", None)
            .await
            .unwrap_or(HealthResponse {
                status: "unhealthy".to_string(),
            });

        Ok(response.status == "healthy")
    }

    fn service_name(&self) -> &str {
        "storage-service"
    }
}

// Request/Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailRequest {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body: String,
    pub is_html: bool,
    pub attachments: Vec<EmailAttachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: String, // Base64 encoded
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailResponse {
    pub message_id: String,
    pub status: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsRequest {
    pub to: Vec<String>,
    pub message: String,
    pub sender_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsResponse {
    pub message_id: String,
    pub status: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushRequest {
    pub to: Vec<String>,
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushResponse {
    pub message_id: String,
    pub status: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub filename: String,
    pub content_type: String,
    pub data: String, // Base64 encoded
    pub folder: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: String,
    pub url: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDownloadResponse {
    pub file_id: String,
    pub url: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDeleteResponse {
    pub file_id: String,
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListRequest {
    pub folder: Option<String>,
    pub tags: Vec<String>,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<FileMetadata>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub folder: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// External service registry
pub struct ExternalServiceRegistry {
    services: HashMap<String, Box<dyn ExternalServiceClient<Config = HttpClientConfig, Error = crate::shared::error::AppError> + Send + Sync>>,
}

impl ExternalServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register_service(&mut self, name: &str, client: Box<dyn ExternalServiceClient<Config = HttpClientConfig, Error = crate::shared::error::AppError> + Send + Sync>) {
        self.services.insert(name.to_string(), client);
    }

    pub fn get_service(&self, name: &str) -> Option<&dyn ExternalServiceClient<Config = HttpClientConfig, Error = crate::shared::error::AppError>> {
        self.services.get(name).map(|s| s.as_ref())
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        for (name, service) in &self.services {
            let health = service.health_check().await.unwrap_or(false);
            results.insert(name.clone(), health);
        }

        results
    }
}

impl Default for ExternalServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_config() {
        let config = HttpClientConfig {
            base_url: "https://api.example.com".to_string(),
            timeout_ms: 5000,
            retry_attempts: 3,
            headers: HashMap::new(),
            auth: Some(AuthConfig::Bearer {
                token: "test-token".to_string(),
            }),
        };

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_email_request() {
        let request = EmailRequest {
            to: vec!["test@example.com".to_string()],
            cc: None,
            bcc: None,
            subject: "Test".to_string(),
            body: "Test email".to_string(),
            is_html: false,
            attachments: vec![],
        };

        assert_eq!(request.to.len(), 1);
        assert_eq!(request.subject, "Test");
        assert!(!request.is_html);
    }

    #[test]
    fn test_external_service_registry() {
        let mut registry = ExternalServiceRegistry::new();

        // Test empty registry
        assert_eq!(registry.get_service("test"), None);

        // Test health check on empty registry
        let results = tokio::runtime::Runtime::new().unwrap().block_on(
            registry.health_check_all()
        );
        assert!(results.is_empty());
    }
}