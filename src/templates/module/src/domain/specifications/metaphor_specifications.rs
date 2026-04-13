// Metaphor Specifications
// Business rules that can be combined and reused for Metaphor validation

use std::collections::HashMap;
use std::fmt;

use crate::domain::entities::Metaphor;
use crate::domain::value_objects::{MetaphorStatus, MetaphorTimestamp};

// Specification Trait
pub trait Specification {
    type Error;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error>;
    fn and<S>(self, other: S) -> AndSpecification<Self, S>
    where
        Self: Sized,
        S: Specification,
    {
        AndSpecification::new(self, other)
    }

    fn or<S>(self, other: S) -> OrSpecification<Self, S>
    where
        Self: Sized,
        S: Specification,
    {
        OrSpecification::new(self, other)
    }

    fn not(self) -> NotSpecification<Self>
    where
        Self: Sized,
    {
        NotSpecification::new(self)
    }
}

// Specification Result
#[derive(Debug, Clone)]
pub struct SpecificationResult {
    pub satisfied: bool,
    pub specification_name: String,
    pub message: String,
    pub details: HashMap<String, String>,
    pub evaluated_at: MetaphorTimestamp,
}

impl SpecificationResult {
    pub fn satisfied(name: String, message: String) -> Self {
        Self {
            satisfied: true,
            specification_name: name,
            message,
            details: HashMap::new(),
            evaluated_at: MetaphorTimestamp::now(),
        }
    }

    pub fn unsatisfied(name: String, message: String) -> Self {
        Self {
            satisfied: false,
            specification_name: name,
            message,
            details: HashMap::new(),
            evaluated_at: MetaphorTimestamp::now(),
        }
    }

    pub fn with_details(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }
}

// Composite Specification Operators
#[derive(Debug, Clone)]
pub struct AndSpecification<T, U> {
    left: T,
    right: U,
}

impl<T, U> AndSpecification<T, U> {
    pub fn new(left: T, right: U) -> Self {
        Self { left, right }
    }
}

impl<T, U> Specification for AndSpecification<T, U>
where
    T: Specification,
    U: Specification,
{
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let left_result = self.left.is_satisfied_by(candidate)
            .map_err(|e| format!("Left specification failed: {}", e))?;
        let right_result = self.right.is_satisfied_by(candidate)
            .map_err(|e| format!("Right specification failed: {}", e))?;

        Ok(left_result && right_result)
    }
}

#[derive(Debug, Clone)]
pub struct OrSpecification<T, U> {
    left: T,
    right: U,
}

impl<T, U> OrSpecification<T, U> {
    pub fn new(left: T, right: U) -> Self {
        Self { left, right }
    }
}

impl<T, U> Specification for OrSpecification<T, U>
where
    T: Specification,
    U: Specification,
{
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let left_result = self.left.is_satisfied_by(candidate)
            .map_err(|e| format!("Left specification failed: {}", e))?;

        if left_result {
            return Ok(true);
        }

        self.right.is_satisfied_by(candidate)
            .map_err(|e| format!("Right specification failed: {}", e))
    }
}

#[derive(Debug, Clone)]
pub struct NotSpecification<T> {
    spec: T,
}

impl<T> NotSpecification<T> {
    pub fn new(spec: T) -> Self {
        Self { spec }
    }
}

impl<T> Specification for NotSpecification<T>
where
    T: Specification,
{
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let result = self.spec.is_satisfied_by(candidate)
            .map_err(|e| format!("Inner specification failed: {}", e))?;
        Ok(!result)
    }
}

// Simple Specifications

#[derive(Debug, Clone)]
pub struct MetaphorNameMustBeValidSpecification;

impl MetaphorNameMustBeValidSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorNameMustBeValidSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let name = candidate.name();

        // Name must be between 1 and 100 characters
        if name.is_empty() {
            return Ok(false);
        }

        if name.len() > 100 {
            return Ok(false);
        }

        // Name must contain only alphanumeric characters, spaces, hyphens, and underscores
        let valid_chars = name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_');
        if !valid_chars {
            return Ok(false);
        }

        // Name must not be empty or contain only whitespace
        if name.trim().is_empty() {
            return Ok(false);
        }

        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorStatusMustBeValidSpecification;

impl MetaphorStatusMustBeValidSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorStatusMustBeValidSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        // Status must be one of the defined enum values (this is always true with Rust enums)
        // Status transitions must follow valid state machine
        matches!(
            candidate.status(),
            MetaphorStatus::Active | MetaphorStatus::Inactive | MetaphorStatus::Suspended | MetaphorStatus::Archived
        )
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorTagsMustBeUniqueSpecification;

impl MetaphorTagsMustBeUniqueSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorTagsMustBeUniqueSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let tags = candidate.tags();
        let mut seen = std::collections::HashSet::new();

        for tag in tags {
            if seen.contains(tag) {
                return Ok(false); // Duplicate found
            }
            seen.insert(tag);
        }

        // Each tag must be between 1 and 50 characters
        for tag in tags {
            if tag.is_empty() || tag.len() > 50 {
                return Ok(false);
            }
        }

        // Maximum 50 tags per metaphor
        if tags.len() > 50 {
            return Ok(false);
        }

        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorMustHaveMetadataSpecification;

impl MetaphorMustHaveMetadataSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorMustHaveMetadataSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let metadata = candidate.metadata();

        // Metadata must not be empty
        if metadata.is_empty() {
            return Ok(false);
        }

        // All metadata keys must be strings (always true with Rust)
        // All metadata values must be strings (always true with Rust)

        Ok(true)
    }
}

// Composite Specifications

#[derive(Debug, Clone)]
pub struct MetaphorIsActiveSpecification;

impl MetaphorIsActiveSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorIsActiveSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        // Metaphor status must be ACTIVE
        if !candidate.status().is_active() {
            return Ok(false);
        }

        // Metaphor must not be deleted
        if candidate.is_deleted() {
            return Ok(false);
        }

        // Metaphor must be valid (combine with other specifications)
        let name_spec = MetaphorNameMustBeValidSpecification::new();
        name_spec.is_satisfied_by(candidate)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorCanDeactivateSpecification;

impl MetaphorCanDeactivateSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorCanDeactivateSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        // Metaphor must currently be ACTIVE
        if !candidate.status().is_active() {
            return Ok(false);
        }

        // Metaphor must not be in SUSPENDED state
        if candidate.status().is_suspended() {
            return Ok(false);
        }

        // Note: Deactivation reason should be provided at the application layer
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorCanSuspendSpecification;

impl MetaphorCanSuspendSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorCanSuspendSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        // Metaphor must be ACTIVE or INACTIVE
        if !candidate.status().is_active() && !candidate.status().is_inactive() {
            return Ok(false);
        }

        // Note: Suspension reason should be provided at the application layer
        // Note: Suspension period should be reasonable (check at application layer)
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorCanArchiveSpecification;

impl MetaphorCanArchiveSpecification {
    pub fn new() -> Self {
        Self
    }
}

impl Specification for MetaphorCanArchiveSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        // Metaphor must be INACTIVE
        if !candidate.status().is_inactive() {
            return Ok(false);
        }

        // Must be inactive for at least 30 days (simplified check - actual implementation would check timestamps)
        let thirty_days_ago = MetaphorTimestamp::now().add_days(-30);
        if candidate.updated_at() > &thirty_days_ago {
            return Ok(false);
        }

        // No pending operations (simplified - actual implementation would check operation status)
        Ok(true)
    }
}

// Temporal Specifications

#[derive(Debug, Clone)]
pub struct MetaphorMustBeRecentSpecification {
    days: i64,
}

impl MetaphorMustBeRecentSpecification {
    pub fn new(days: i64) -> Self {
        Self { days }
    }
}

impl Specification for MetaphorMustBeRecentSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let cutoff = MetaphorTimestamp::now().add_days(-self.days);
        Ok(candidate.created_at() >= &cutoff)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorMustNotBeOlderThanSpecification {
    max_age_days: i64,
}

impl MetaphorMustNotBeOlderThanSpecification {
    pub fn new(max_age_days: i64) -> Self {
        Self { max_age_days }
    }
}

impl Specification for MetaphorMustNotBeOlderThanSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let cutoff = MetaphorTimestamp::now().add_days(-self.max_age_days);
        Ok(candidate.created_at() >= &cutoff)
    }
}

// Parameterized Specifications

#[derive(Debug, Clone)]
pub struct MetaphorTaggedWithSpecification {
    required_tags: Vec<String>,
    match_all: bool,
}

impl MetaphorTaggedWithSpecification {
    pub fn new(required_tags: Vec<String>, match_all: bool) -> Self {
        Self {
            required_tags,
            match_all,
        }
    }
}

impl Specification for MetaphorTaggedWithSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        if self.required_tags.is_empty() {
            return Ok(true);
        }

        let metaphor_tags = candidate.tags();

        if self.match_all {
            // All required tags must be present
            Ok(self.required_tags.iter().all(|tag| metaphor_tags.contains(tag)))
        } else {
            // At least one required tag must be present
            Ok(self.required_tags.iter().any(|tag| metaphor_tags.contains(tag)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorInDateRangeSpecification {
    start_date: MetaphorTimestamp,
    end_date: MetaphorTimestamp,
    include_start: bool,
    include_end: bool,
}

impl MetaphorInDateRangeSpecification {
    pub fn new(
        start_date: MetaphorTimestamp,
        end_date: MetaphorTimestamp,
        include_start: bool,
        include_end: bool,
    ) -> Self {
        Self {
            start_date,
            end_date,
            include_start,
            include_end,
        }
    }
}

impl Specification for MetaphorInDateRangeSpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let created_at = candidate.created_at();

        let after_start = if self.include_start {
            created_at >= &self.start_date
        } else {
            created_at > &self.start_date
        };

        let before_end = if self.include_end {
            created_at <= &self.end_date
        } else {
            created_at < &self.end_date
        };

        Ok(after_start && before_end)
    }
}

#[derive(Debug, Clone)]
pub struct MetaphorWithMetadataKeySpecification {
    key: String,
    value: Option<String>,
}

impl MetaphorWithMetadataKeySpecification {
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
    }
}

impl Specification for MetaphorWithMetadataKeySpecification {
    type Error = String;

    fn is_satisfied_by(&self, candidate: &Metaphor) -> Result<bool, Self::Error> {
        let metadata = candidate.metadata();

        match &self.value {
            Some(expected_value) => {
                // Check if key exists and has specific value
                metadata.get(&self.key).map_or(false, |v| v == expected_value)
            }
            None => {
                // Just check if key exists
                metadata.contains_key(&self.key)
            }
        }
    }
}

// Specification Evaluator
pub struct SpecificationEvaluator;

impl SpecificationEvaluator {
    pub fn evaluate<S: Specification>(
        specification: &S,
        candidate: &Metaphor,
    ) -> Result<SpecificationResult, S::Error> {
        let satisfied = specification.is_satisfied_by(candidate)?;
        let spec_name = std::any::type_name::<S>().split("::").last().unwrap_or("Unknown");

        let result = if satisfied {
            SpecificationResult::satisfied(
                spec_name.to_string(),
                format!("Specification '{}' is satisfied", spec_name),
            )
        } else {
            SpecificationResult::unsatisfied(
                spec_name.to_string(),
                format!("Specification '{}' is not satisfied", spec_name),
            )
        };

        Ok(result)
    }

    pub fn evaluate_batch<S: Specification>(
        specification: &S,
        candidates: &[Metaphor],
    ) -> Vec<Result<SpecificationResult, S::Error>> {
        candidates
            .iter()
            .map(|candidate| Self::evaluate(specification, candidate))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{MetaphorName, Metadata};

    fn create_test_metaphor() -> Metaphor {
        Metaphor::create(
            MetaphorName::new("Test Metaphor").unwrap(),
            "Test Description".to_string(),
            vec!["test".to_string(), "production".to_string()],
            {
                let mut metadata = Metadata::new();
                metadata.insert("env".to_string(), "production".to_string()).unwrap();
                metadata
            },
            "test_user".to_string(),
        ).unwrap()
    }

    #[test]
    fn test_metaphor_name_specification() {
        let spec = MetaphorNameMustBeValidSpecification::new();
        let valid_metaphor = create_test_metaphor();

        assert!(spec.is_satisfied_by(&valid_metaphor).unwrap());

        // Test with invalid name (empty)
        let invalid_metaphor = Metaphor::create(
            MetaphorName::new("").unwrap(), // This would normally fail at creation
            "Test".to_string(),
            vec![],
            Metadata::new(),
            "user".to_string(),
        ).unwrap();

        // This test is conceptual - in practice, name validation happens at creation
    }

    #[test]
    fn test_tags_unique_specification() {
        let spec = MetaphorTagsMustBeUniqueSpecification::new();
        let valid_metaphor = create_test_metaphor();

        assert!(spec.is_satisfied_by(&valid_metaphor).unwrap());
    }

    #[test]
    fn test_is_active_specification() {
        let spec = MetaphorIsActiveSpecification::new();
        let active_metaphor = create_test_metaphor();

        assert!(spec.is_satisfied_by(&active_metaphor).unwrap());

        // Create inactive metaphor
        let mut inactive_metaphor = create_test_metaphor();
        // Note: In a real implementation, you'd need to be able to change status
        // This is just for testing the specification logic
    }

    #[test]
    fn test_tagged_with_specification() {
        let spec_match_all = MetaphorTaggedWithSpecification::new(
            vec!["test".to_string(), "production".to_string()],
            true,
        );

        let spec_match_any = MetaphorTaggedWithSpecification::new(
            vec!["test".to_string(), "nonexistent".to_string()],
            false,
        );

        let metaphor = create_test_metaphor();

        assert!(spec_match_all.is_satisfied_by(&metaphor).unwrap());
        assert!(spec_match_any.is_satisfied_by(&metaphor).unwrap());
    }

    #[test]
    fn test_composite_specifications() {
        let name_spec = MetaphorNameMustBeValidSpecification::new();
        let tags_spec = MetaphorTagsMustBeUniqueSpecification::new();

        let combined_and = name_spec.and(tags_spec);
        let metaphor = create_test_metaphor();

        assert!(combined_and.is_satisfied_by(&metaphor).unwrap());
    }

    #[test]
    fn test_not_specification() {
        let active_spec = MetaphorIsActiveSpecification::new();
        let not_active = active_spec.not();

        // Create an inactive metaphor (conceptual test)
        let active_metaphor = create_test_metaphor();
        assert!(active_metaphor.status().is_active());

        // The not specification should return false for an active metaphor
        assert!(!not_active.is_satisfied_by(&active_metaphor).unwrap());
    }

    #[test]
    fn test_specification_evaluator() {
        let spec = MetaphorNameMustBeValidSpecification::new();
        let metaphor = create_test_metaphor();

        let result = SpecificationEvaluator::evaluate(&spec, &metaphor).unwrap();
        assert!(result.satisfied);
        assert!(result.specification_name.contains("MetaphorNameMustBeValidSpecification"));
    }

    #[test]
    fn test_metadata_specification() {
        let spec_has_key = MetaphorWithMetadataKeySpecification::new(
            "env".to_string(),
            None,
        );

        let spec_has_key_value = MetaphorWithMetadataKeySpecification::new(
            "env".to_string(),
            Some("production".to_string()),
        );

        let spec_wrong_value = MetaphorWithMetadataKeySpecification::new(
            "env".to_string(),
            Some("development".to_string()),
        );

        let metaphor = create_test_metaphor();

        assert!(spec_has_key.is_satisfied_by(&metaphor).unwrap());
        assert!(spec_has_key_value.is_satisfied_by(&metaphor).unwrap());
        assert!(!spec_wrong_value.is_satisfied_by(&metaphor).unwrap());
    }

    #[test]
    fn test_temporal_specifications() {
        let recent_spec = MetaphorMustBeRecentSpecification::new(30);
        let metaphor = create_test_metaphor();

        assert!(recent_spec.is_satisfied_by(&metaphor).unwrap());

        let old_spec = MetaphorMustNotBeOlderThanSpecification::new(1);
        assert!(old_spec.is_satisfied_by(&metaphor).unwrap());
    }
}