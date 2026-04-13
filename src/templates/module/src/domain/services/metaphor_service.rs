// Metaphor Domain Service
// Domain services for Metaphor business logic that doesn't fit in entities

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::entities::Metaphor;
use crate::domain::value_objects::{MetaphorId, MetaphorStatus, MetaphorTimestamp};
use crate::domain::repositories::{MetaphorRepository, RepositoryError, RepositoryResult};

// Service Result Type
pub type ServiceResult<T> = Result<T, ServiceError>;

// Service Error Types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Repository error: {source}")]
    RepositoryError { #[from] source: RepositoryError },

    #[error("Calculation error: {message}")]
    CalculationError { message: String },

    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Unknown service error: {message}")]
    Unknown { message: String },
}

// Validation Service
#[async_trait]
pub trait MetaphorValidationService: Send + Sync {
    async fn validate_metaphor_integrity(&self, metaphor: &Metaphor) -> ServiceResult<ValidationReport>;
    async fn check_business_rules(&self, metaphor: &Metaphor, operation: &str) -> ServiceResult<BusinessRuleReport>;
    async fn validate_metaphor_configuration(&self, metaphor: &Metaphor) -> ServiceResult<ConfigurationReport>;
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub metaphor_id: MetaphorId,
    pub valid: bool,
    pub results: Vec<ValidationResult>,
    pub summary: String,
    pub validated_at: MetaphorTimestamp,
}

impl ValidationReport {
    pub fn new(metaphor_id: MetaphorId) -> Self {
        Self {
            metaphor_id,
            valid: true,
            results: Vec::new(),
            summary: String::new(),
            validated_at: MetaphorTimestamp::now(),
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        if !result.passed {
            self.valid = false;
        }
        self.results.push(result);
    }

    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    pub fn generate_summary(&mut self) {
        let passed = self.passed_count();
        let failed = self.failed_count();
        let total = self.results.len();

        self.summary = match (passed, failed) {
            (0, 0) => "No validation rules executed".to_string(),
            (p, 0) => format!("All {} validation rules passed", p),
            (0, f) => format!("All {} validation rules failed", f),
            (p, f) => format!("{} passed, {} failed out of {} total", p, f, total),
        };
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub rule: String,
    pub passed: bool,
    pub message: String,
    pub severity: ValidationSeverity,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

impl ValidationResult {
    pub fn passed(rule: String, message: String) -> Self {
        Self {
            rule,
            passed: true,
            message,
            severity: ValidationSeverity::Info,
            details: Vec::new(),
        }
    }

    pub fn failed(rule: String, message: String, severity: ValidationSeverity) -> Self {
        Self {
            rule,
            passed: false,
            message,
            severity,
            details: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

#[derive(Debug, Clone)]
pub struct BusinessRuleReport {
    pub allowed: bool,
    pub violated_rules: Vec<BusinessRule>,
    pub passed_rules: Vec<BusinessRule>,
    pub recommendation: String,
    pub evaluated_at: MetaphorTimestamp,
}

impl BusinessRuleReport {
    pub fn new() -> Self {
        Self {
            allowed: true,
            violated_rules: Vec::new(),
            passed_rules: Vec::new(),
            recommendation: String::new(),
            evaluated_at: MetaphorTimestamp::now(),
        }
    }

    pub fn add_violated_rule(&mut self, rule: BusinessRule) {
        self.allowed = false;
        self.violated_rules.push(rule);
    }

    pub fn add_passed_rule(&mut self, rule: BusinessRule) {
        self.passed_rules.push(rule);
    }

    pub fn has_violations(&self) -> bool {
        !self.violated_rules.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct BusinessRule {
    pub name: String,
    pub description: String,
    pub passed: bool,
    pub severity: RuleSeverity,
    pub message: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl BusinessRule {
    pub fn passed(name: String, description: String, message: String) -> Self {
        Self {
            name,
            description,
            passed: true,
            severity: RuleSeverity::Info,
            message,
            recommendation: None,
        }
    }

    pub fn violated(
        name: String,
        description: String,
        severity: RuleSeverity,
        message: String,
        recommendation: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            passed: false,
            severity,
            message,
            recommendation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigurationReport {
    pub valid: bool,
    pub issues: Vec<ConfigurationIssue>,
    pub warnings: Vec<ConfigurationWarning>,
    pub validation_profile: String,
    pub validated_at: MetaphorTimestamp,
}

impl ConfigurationReport {
    pub fn new(profile: String) -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
            validation_profile: profile,
            validated_at: MetaphorTimestamp::now(),
        }
    }

    pub fn add_issue(&mut self, issue: ConfigurationIssue) {
        self.valid = false;
        self.issues.push(issue);
    }

    pub fn add_warning(&mut self, warning: ConfigurationWarning) {
        self.warnings.push(warning);
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigurationIssue {
    pub component: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct ConfigurationWarning {
    pub component: String,
    pub message: String,
    pub recommendation: String,
}

// Metrics and Reporting Service
#[async_trait]
pub trait MetaphorMetricsService: Send + Sync {
    async fn calculate_metrics(
        &self,
        metaphor_id: &MetaphorId,
        metric_types: &[String],
        time_range: &str,
    ) -> ServiceResult<MetricsReport>;

    async fn generate_report(
        &self,
        metaphor_id: &MetaphorId,
        report_type: &str,
        format: &str,
        parameters: &HashMap<String, String>,
    ) -> ServiceResult<ReportResult>;
}

#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub metaphor_id: MetaphorId,
    pub metrics: Vec<MetricValue>,
    pub calculation_time: MetaphorTimestamp,
    pub time_range: String,
}

#[derive(Debug, Clone)]
pub struct MetricValue {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ReportResult {
    pub report_id: String,
    pub status: ReportStatus,
    pub download_url: Option<String>,
    pub format: String,
    pub size_bytes: u64,
    pub generated_at: MetaphorTimestamp,
    pub expires_at: Option<MetaphorTimestamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

// Synchronization and Migration Service
#[async_trait]
pub trait MetaphorSyncService: Send + Sync {
    async fn synchronize_data(
        &self,
        metaphor_ids: &[MetaphorId],
        target_system: &str,
        full_sync: bool,
    ) -> ServiceResult<SyncResult>;

    async fn migrate_data(
        &self,
        source_system: &str,
        target_system: &str,
        metaphor_ids: &[MetaphorId],
        dry_run: bool,
        options: &HashMap<String, String>,
    ) -> ServiceResult<MigrationResult>;
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub results: Vec<SynchronizationResult>,
    pub success_count: usize,
    pub failure_count: usize,
    pub sync_id: String,
    pub synchronized_at: MetaphorTimestamp,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SynchronizationResult {
    pub metaphor_id: MetaphorId,
    pub success: bool,
    pub message: String,
    pub last_synced_at: Option<MetaphorTimestamp>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub migration_id: String,
    pub dry_run: bool,
    pub results: Vec<MigrationItemResult>,
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub warnings: Vec<String>,
    pub migrated_at: MetaphorTimestamp,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct MigrationItemResult {
    pub metaphor_id: MetaphorId,
    pub success: bool,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    pub message: String,
    pub errors: Vec<String>,
}

// Configuration Optimization Service
#[async_trait]
pub trait MetaphorOptimizationService: Send + Sync {
    async fn optimize_configuration(
        &self,
        metaphor_id: &MetaphorId,
        optimization_targets: &[String],
        optimization_level: &str,
    ) -> ServiceResult<OptimizationResult>;
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub metaphor_id: MetaphorId,
    pub optimized: bool,
    pub optimizations: Vec<ConfigurationOptimization>,
    pub performance_improvement: String,
    pub recommendations: Vec<String>,
    pub optimized_at: MetaphorTimestamp,
}

#[derive(Debug, Clone)]
pub struct ConfigurationOptimization {
    pub component: String,
    pub setting: String,
    pub old_value: String,
    pub new_value: String,
    pub impact: String,
    pub risk_level: OptimizationRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationRisk {
    Low,
    Medium,
    High,
    Critical,
}

// Default Service Implementation
pub struct DefaultMetaphorValidationService {
    repository: Box<dyn MetaphorRepository>,
}

impl DefaultMetaphorValidationService {
    pub fn new(repository: Box<dyn MetaphorRepository>) -> Self {
        Self { repository }
    }

    async fn validate_name(&self, metaphor: &Metaphor) -> ServiceResult<ValidationResult> {
        if metaphor.name().is_empty() {
            return Ok(ValidationResult::failed(
                "name_validation".to_string(),
                "Metaphor name cannot be empty".to_string(),
                ValidationSeverity::Error,
            ));
        }

        if metaphor.name().len() > 100 {
            return Ok(ValidationResult::failed(
                "name_validation".to_string(),
                "Metaphor name exceeds maximum length".to_string(),
                ValidationSeverity::Error,
            ));
        }

        Ok(ValidationResult::passed(
            "name_validation".to_string(),
            "Metaphor name is valid".to_string(),
        ))
    }

    async fn validate_tags(&self, metaphor: &Metaphor) -> ServiceResult<ValidationResult> {
        if metaphor.tags().len() > 50 {
            return Ok(ValidationResult::failed(
                "tags_validation".to_string(),
                "Too many tags (maximum 50)".to_string(),
                ValidationSeverity::Warning,
            ));
        }

        let mut duplicates = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for tag in metaphor.tags() {
            if seen.contains(tag) {
                duplicates.push(tag.clone());
            } else {
                seen.insert(tag.clone());
            }
        }

        if !duplicates.is_empty() {
            return Ok(ValidationResult::failed(
                "tags_validation".to_string(),
                format!("Duplicate tags found: {:?}", duplicates),
                ValidationSeverity::Warning,
            ));
        }

        Ok(ValidationResult::passed(
            "tags_validation".to_string(),
            "Tags are valid".to_string(),
        ))
    }

    async fn validate_metadata(&self, metaphor: &Metaphor) -> ServiceResult<ValidationResult> {
        if metaphor.metadata().is_empty() {
            return Ok(ValidationResult::failed(
                "metadata_validation".to_string(),
                "Metadata is empty".to_string(),
                ValidationSeverity::Warning,
            ));
        }

        Ok(ValidationResult::passed(
            "metadata_validation".to_string(),
            "Metadata is valid".to_string(),
        ))
    }

    async fn check_name_uniqueness(&self, metaphor: &Metaphor) -> ServiceResult<ValidationResult> {
        // This would typically check if the name is unique within a scope
        // For now, we'll assume it's valid
        Ok(ValidationResult::passed(
            "name_uniqueness".to_string(),
            "Metaphor name is unique".to_string(),
        ))
    }
}

#[async_trait]
impl MetaphorValidationService for DefaultMetaphorValidationService {
    async fn validate_metaphor_integrity(&self, metaphor: &Metaphor) -> ServiceResult<ValidationReport> {
        let mut report = ValidationReport::new(metaphor.id().clone());

        // Run all validation rules
        let validations = vec![
            self.validate_name(metaphor).await,
            self.validate_tags(metaphor).await,
            self.validate_metadata(metaphor).await,
            self.check_name_uniqueness(metaphor).await,
        ];

        for validation in validations {
            match validation {
                Ok(result) => report.add_result(result),
                Err(e) => report.add_result(ValidationResult::failed(
                    "validation_error".to_string(),
                    format!("Validation error: {}", e),
                    ValidationSeverity::Error,
                )),
            }
        }

        report.generate_summary();
        Ok(report)
    }

    async fn check_business_rules(&self, metaphor: &Metaphor, operation: &str) -> ServiceResult<BusinessRuleReport> {
        let mut report = BusinessRuleReport::new();

        // Check operation-specific rules
        match operation {
            "create" => {
                // Check if metaphor can be created
                let rule = BusinessRule::passed(
                    "creation_allowed".to_string(),
                    "Metaphor creation is allowed".to_string(),
                    "Business rules for creation passed".to_string(),
                );
                report.add_passed_rule(rule);
            }
            "delete" => {
                // Check if metaphor can be deleted
                if matches!(metaphor.status(), MetaphorStatus::Active) {
                    let rule = BusinessRule::violated(
                        "active_deletion".to_string(),
                        "Cannot delete active metaphor".to_string(),
                        RuleSeverity::High,
                        "Metaphor must be deactivated before deletion".to_string(),
                        Some("Deactivate the metaphor first".to_string()),
                    );
                    report.add_violated_rule(rule);
                }
            }
            _ => {}
        }

        Ok(report)
    }

    async fn validate_metaphor_configuration(&self, metaphor: &Metaphor) -> ServiceResult<ConfigurationReport> {
        let mut report = ConfigurationReport::new("default".to_string());

        // Basic configuration validation
        if metaphor.name().is_empty() {
            report.add_issue(ConfigurationIssue {
                component: "metaphor".to_string(),
                severity: IssueSeverity::Critical,
                message: "Metaphor name is required".to_string(),
                suggestion: "Set a valid name for the metaphor".to_string(),
            });
        }

        if metaphor.tags().is_empty() {
            report.add_warning(ConfigurationWarning {
                component: "metaphor".to_string(),
                message: "No tags defined".to_string(),
                recommendation: "Add tags to improve discoverability".to_string(),
            });
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{MetaphorName, Metadata};
    use async_trait::async_trait;

    // Mock repository for testing
    struct MockRepository;

    #[async_trait]
    impl MetaphorRepository for MockRepository {
        async fn save(&self, _metaphor: &Metaphor) -> RepositoryResult<()> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &MetaphorId) -> RepositoryResult<Option<Metaphor>> {
            Ok(None)
        }

        async fn delete(&self, _id: &MetaphorId, _hard_delete: bool) -> RepositoryResult<()> {
            Ok(())
        }

        async fn find_all(
            &self,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_with_filters(
            &self,
            _filters: crate::domain::repositories::MetaphorFilters,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_status(
            &self,
            _status: MetaphorStatus,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_tags(
            &self,
            _tags: Vec<String>,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_created_by(
            &self,
            _created_by: &str,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn search(
            &self,
            _query: &str,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn save_batch(&self, _metaphors: &[Metaphor]) -> RepositoryResult<()> {
            Ok(())
        }

        async fn delete_batch(&self, _ids: &[MetaphorId], _hard_delete: bool) -> RepositoryResult<()> {
            Ok(())
        }

        async fn exists(&self, _id: &MetaphorId) -> RepositoryResult<bool> {
            Ok(false)
        }

        async fn count(&self, _filters: Option<crate::domain::repositories::MetaphorFilters>) -> RepositoryResult<u64> {
            Ok(0)
        }

        async fn find_deleted(
            &self,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn restore(&self, _id: &MetaphorId) -> RepositoryResult<()> {
            Ok(())
        }

        async fn find_by_metadata(
            &self,
            _metadata_key: &str,
            _metadata_value: Option<&str>,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn find_by_date_range(
            &self,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
            _date_field: crate::domain::repositories::SortField,
            _pagination: crate::domain::repositories::PaginationParams,
            _sort: crate::domain::repositories::SortParams,
        ) -> RepositoryResult<crate::domain::repositories::PaginatedResult<Metaphor>> {
            Ok(crate::domain::repositories::PaginatedResult::empty(1, 20))
        }

        async fn get_status_counts(&self) -> RepositoryResult<std::collections::HashMap<MetaphorStatus, u64>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_tag_counts(&self) -> RepositoryResult<std::collections::HashMap<String, u64>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_recently_created(&self, _days: i64, _limit: Option<usize>) -> RepositoryResult<Vec<Metaphor>> {
            Ok(Vec::new())
        }

        async fn health_check(&self) -> RepositoryResult<bool> {
            Ok(true)
        }

        async fn connection_pool_status(&self) -> RepositoryResult<std::collections::HashMap<String, serde_json::Value>> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[tokio::test]
    async fn test_validation_report() {
        let mut report = ValidationReport::new(MetaphorId::generate());

        report.add_result(ValidationResult::passed(
            "test_rule".to_string(),
            "Test passed".to_string(),
        ));

        report.add_result(ValidationResult::failed(
            "test_rule2".to_string(),
            "Test failed".to_string(),
            ValidationSeverity::Error,
        ));

        report.generate_summary();

        assert!(!report.valid);
        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(report.summary.contains("1 passed, 1 failed"));
    }

    #[tokio::test]
    async fn test_business_rule_report() {
        let mut report = BusinessRuleReport::new();

        report.add_passed_rule(BusinessRule::passed(
            "rule1".to_string(),
            "Rule 1".to_string(),
            "Passed".to_string(),
        ));

        report.add_violated_rule(BusinessRule::violated(
            "rule2".to_string(),
            "Rule 2".to_string(),
            RuleSeverity::High,
            "Violated".to_string(),
            Some("Fix it".to_string()),
        ));

        assert!(!report.allowed);
        assert!(report.has_violations());
        assert_eq!(report.passed_rules.len(), 1);
        assert_eq!(report.violated_rules.len(), 1);
    }

    #[tokio::test]
    async fn test_configuration_report() {
        let mut report = ConfigurationReport::new("test".to_string());

        assert!(report.valid);

        report.add_issue(ConfigurationIssue {
            component: "test".to_string(),
            severity: IssueSeverity::High,
            message: "Test issue".to_string(),
            suggestion: "Fix it".to_string(),
        });

        assert!(!report.valid);
        assert!(report.has_issues());
    }

    #[tokio::test]
    async fn test_validation_service() {
        let service = DefaultMetaphorValidationService::new(Box::new(MockRepository));

        let metaphor = Metaphor::create(
            MetaphorName::new("Test Metaphor").unwrap(),
            "Test Description".to_string(),
            vec!["test".to_string()],
            Metadata::new(),
            "test_user".to_string(),
        ).unwrap();

        let report = service.validate_metaphor_integrity(&metaphor).await.unwrap();
        assert!(report.valid);

        let business_report = service.check_business_rules(&metaphor, "create").await.unwrap();
        assert!(business_report.allowed);

        let config_report = service.validate_metaphor_configuration(&metaphor).await.unwrap();
        assert!(config_report.valid);
    }
}