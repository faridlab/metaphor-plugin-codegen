//! Module Health Service
//!
//! Monitors and manages health status of all modules in the application

use crate::domain::value_objects::{ModuleHealth, HealthStatus};
use crate::shared::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// Domain service for monitoring module health
pub struct ModuleHealthService {
    modules: HashMap<String, ModuleHealth>,
    health_check_timeout: Duration,
    unhealthy_threshold: u32,
}

impl ModuleHealthService {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            health_check_timeout: Duration::from_secs(10),
            unhealthy_threshold: 3,
        }
    }

    pub fn with_config(timeout_seconds: u64, unhealthy_threshold: u32) -> Self {
        Self {
            modules: HashMap::new(),
            health_check_timeout: Duration::from_secs(timeout_seconds),
            unhealthy_threshold,
        }
    }

    /// Register a module for health monitoring
    pub fn register_module(&mut self, module_name: String) {
        let module_health = ModuleHealth::healthy(module_name.clone());
        self.modules.insert(module_name, module_health);
    }

    /// Check health of a specific module
    pub async fn check_module_health(&mut self, module_name: &str) -> AppResult<ModuleHealth> {
        if !self.modules.contains_key(module_name) {
            return Err(AppError::not_found(format!("Module '{}' not registered", module_name)));
        }

        let health_result = self.perform_health_check(module_name).await;
        let now = Utc::now();

        let module_health = self.modules.get_mut(module_name).unwrap();
        module_health.last_check = now;

        match health_result {
            Ok((response_time, metrics)) => {
                // Health check successful
                module_health.status = HealthStatus::Healthy;
                module_health.response_time_ms = Some(response_time);
                module_health.error_message = None;
                module_health.metrics = metrics;

                // Reset failure count
                // (In a real implementation, you'd track this separately)
            }
            Err(error) => {
                // Health check failed
                module_health.status = HealthStatus::Unhealthy;
                module_health.response_time_ms = None;
                module_health.error_message = Some(error.to_string());

                // Increment failure count and mark as unhealthy if threshold exceeded
                // (In a real implementation, you'd track this)
            }
        }

        Ok(module_health.clone())
    }

    /// Check health of all registered modules
    pub async fn check_all_modules_health(&mut self) -> Vec<ModuleHealth> {
        let module_names: Vec<String> = self.modules.keys().cloned().collect();
        let mut health_results = Vec::new();

        for module_name in module_names {
            match self.check_module_health(&module_name).await {
                Ok(health) => health_results.push(health),
                Err(_) => {
                    // Create unhealthy status for failed health checks
                    let unhealthy_health = ModuleHealth::unhealthy(
                        module_name.clone(),
                        "Health check failed".to_string(),
                    );
                    health_results.push(unhealthy_health);
                }
            }
        }

        health_results
    }

    /// Get current health status of a module
    pub fn get_module_health(&self, module_name: &str) -> Option<&ModuleHealth> {
        self.modules.get(module_name)
    }

    /// Get health status of all modules
    pub fn get_all_modules_health(&self) -> Vec<ModuleHealth> {
        self.modules.values().cloned().collect()
    }

    /// Get overall application health status
    pub fn get_overall_health(&self) -> HealthStatus {
        if self.modules.is_empty() {
            return HealthStatus::Degraded;
        }

        let mut unhealthy_count = 0;
        let mut degraded_count = 0;

        for module_health in self.modules.values() {
            match module_health.status {
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Healthy => {}
            }
        }

        // If any module is unhealthy, overall status is unhealthy
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        }
        // If any module is degraded and none are unhealthy, overall status is degraded
        else if degraded_count > 0 {
            HealthStatus::Degraded
        }
        // All modules are healthy
        else {
            HealthStatus::Healthy
        }
    }

    /// Get modules by health status
    pub fn get_modules_by_status(&self, status: HealthStatus) -> Vec<&ModuleHealth> {
        self.modules
            .values()
            .filter(|health| health.status == status)
            .collect()
    }

    /// Get modules with slow response times
    pub fn get_slow_modules(&self, threshold_ms: u64) -> Vec<&ModuleHealth> {
        self.modules
            .values()
            .filter(|health| {
                health
                    .response_time_ms
                    .map_or(false, |rt| rt > threshold_ms)
            })
            .collect()
    }

    /// Manually set module health status (for external health checks)
    pub fn set_module_health(&mut self, module_name: &str, health: ModuleHealth) -> AppResult<()> {
        if !self.modules.contains_key(module_name) {
            return Err(AppError::not_found(format!("Module '{}' not registered", module_name)));
        }

        self.modules.insert(module_name.to_string(), health);
        Ok(())
    }

    /// Update module metrics
    pub fn update_module_metrics(
        &mut self,
        module_name: &str,
        metrics: HashMap<String, f64>,
    ) -> AppResult<()> {
        let module_health = self.modules.get_mut(module_name)
            .ok_or_else(|| AppError::not_found(format!("Module '{}' not found", module_name)))?;

        module_health.metrics = metrics;
        module_health.last_check = Utc::now();

        Ok(())
    }

    /// Perform actual health check on a module
    async fn perform_health_check(&self, module_name: &str) -> AppResult<(u64, HashMap<String, f64>)> {
        // In a real implementation, this would make HTTP requests or other
        // checks to the module's health endpoint
        let start_time = std::time::Instant::now();

        // Simulate health check
        let health_check_result = timeout(
            self.health_check_timeout,
            self.simulate_module_health_check(module_name),
        )
        .await;

        let elapsed = start_time.elapsed();

        match health_check_result {
            Ok(Ok(metrics)) => Ok((elapsed.as_millis() as u64, metrics)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::internal(format!(
                "Health check timeout for module '{}'",
                module_name
            ))),
        }
    }

    /// Simulate module health check (placeholder implementation)
    async fn simulate_module_health_check(&self, module_name: &str) -> AppResult<HashMap<String, f64>> {
        // Simulate some delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Simulate different health statuses based on module name
        match module_name {
            "sapiens" => Ok(HashMap::from([
                ("database_connections".to_string(), 15.0),
                ("active_sessions".to_string(), 127.0),
                ("memory_usage_mb".to_string(), 256.0),
            ])),
            "postman" => Ok(HashMap::from([
                ("emails_sent_today".to_string(), 1234.0),
                ("queue_size".to_string(), 23.0),
                ("smtp_response_time_ms".to_string(), 45.0),
            ])),
            "bucket" => Ok(HashMap::from([
                ("files_stored".to_string(), 56789.0),
                ("storage_used_gb".to_string(), 123.45),
                ("active_uploads".to_string(), 5.0),
            ])),
            _ => Err(AppError::not_found(format!("Module '{}' not found", module_name))),
        }
    }

    /// Get health summary for dashboard
    pub fn get_health_summary(&self) -> HealthSummary {
        let total_modules = self.modules.len();
        let healthy_modules = self.get_modules_by_status(HealthStatus::Healthy).len();
        let degraded_modules = self.get_modules_by_status(HealthStatus::Degraded).len();
        let unhealthy_modules = self.get_modules_by_status(HealthStatus::Unhealthy).len();

        let overall_status = self.get_overall_health();

        HealthSummary {
            total_modules,
            healthy_modules,
            degraded_modules,
            unhealthy_modules,
            overall_status,
            last_check: Utc::now(),
        }
    }

    /// Enable/disable a module (affects health monitoring)
    pub fn set_module_enabled(&mut self, module_name: &str, enabled: bool) -> AppResult<()> {
        let module_health = self.modules.get_mut(module_name)
            .ok_or_else(|| AppError::not_found(format!("Module '{}' not found", module_name)))?;

        if enabled {
            if module_health.status == HealthStatus::Unhealthy {
                module_health.status = HealthStatus::Degraded;
            }
        } else {
            module_health.status = HealthStatus::Unhealthy;
            module_health.error_message = Some("Module disabled".to_string());
        }

        Ok(())
    }

    /// Get average response time across all modules
    pub fn get_average_response_time(&self) -> Option<f64> {
        let response_times: Vec<u64> = self
            .modules
            .values()
            .filter_map(|health| health.response_time_ms)
            .collect();

        if response_times.is_empty() {
            None
        } else {
            let sum: u64 = response_times.iter().sum();
            Some(sum as f64 / response_times.len() as f64)
        }
    }
}

impl Default for ModuleHealthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Health summary for dashboards and monitoring
#[derive(Debug, Clone, Serialize)]
pub struct HealthSummary {
    pub total_modules: usize,
    pub healthy_modules: usize,
    pub degraded_modules: usize,
    pub unhealthy_modules: usize,
    pub overall_status: HealthStatus,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_module_registration() {
        let mut service = ModuleHealthService::new();
        service.register_module("test_module".to_string());

        let health = service.get_module_health("test_module");
        assert!(health.is_some());
        assert!(matches!(health.unwrap().status, HealthStatus::Healthy));
    }

    #[test]
    fn test_overall_health() {
        let mut service = ModuleHealthService::new();

        // No modules -> Degraded
        assert_eq!(service.get_overall_health(), HealthStatus::Degraded);

        service.register_module("module1".to_string());
        assert_eq!(service.get_overall_health(), HealthStatus::Healthy);

        // Mark one as unhealthy
        let unhealthy_health = ModuleHealth::unhealthy("module1".to_string(), "Test error".to_string());
        service.set_module_health("module1", unhealthy_health).unwrap();
        assert_eq!(service.get_overall_health(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_get_modules_by_status() {
        let mut service = ModuleHealthService::new();
        service.register_module("healthy".to_string());
        service.register_module("unhealthy".to_string());

        let unhealthy_health = ModuleHealth::unhealthy("unhealthy".to_string(), "Test error".to_string());
        service.set_module_health("unhealthy", unhealthy_health).unwrap();

        let healthy_modules = service.get_modules_by_status(HealthStatus::Healthy);
        let unhealthy_modules = service.get_modules_by_status(HealthStatus::Unhealthy);

        assert_eq!(healthy_modules.len(), 1);
        assert_eq!(unhealthy_modules.len(), 1);
    }

    #[test]
    fn test_health_summary() {
        let mut service = ModuleHealthService::new();
        service.register_module("module1".to_string());
        service.register_module("module2".to_string());

        let summary = service.get_health_summary();
        assert_eq!(summary.total_modules, 2);
        assert_eq!(summary.healthy_modules, 2);
        assert_eq!(summary.degraded_modules, 0);
        assert_eq!(summary.unhealthy_modules, 0);
        assert!(matches!(summary.overall_status, HealthStatus::Healthy));
    }

    #[test]
    fn test_update_module_metrics() {
        let mut service = ModuleHealthService::new();
        service.register_module("test_module".to_string());

        let mut metrics = HashMap::new();
        metrics.insert("test_metric".to_string(), 42.0);

        service.update_module_metrics("test_module", metrics).unwrap();

        let health = service.get_module_health("test_module").unwrap();
        assert_eq!(health.metrics.get("test_metric"), Some(&42.0));
    }

    #[tokio::test]
    async fn test_module_health_check() {
        let mut service = ModuleHealthService::new();
        service.register_module("sapiens".to_string());

        let health = service.check_module_health("sapiens").await.unwrap();
        assert!(matches!(health.status, HealthStatus::Healthy));
        assert!(health.response_time_ms.is_some());
        assert!(!health.metrics.is_empty());
    }

    #[test]
    fn test_slow_modules() {
        let mut service = ModuleHealthService::new();
        service.register_module("slow_module".to_string());

        // Manually create a module with slow response time
        let mut slow_health = ModuleHealth::healthy("slow_module".to_string());
        slow_health.response_time_ms = Some(5000); // 5 seconds

        service.set_module_health("slow_module", slow_health).unwrap();

        let slow_modules = service.get_slow_modules(1000); // 1 second threshold
        assert_eq!(slow_modules.len(), 1);
    }

    #[test]
    fn test_average_response_time() {
        let mut service = ModuleHealthService::new();

        // No modules -> None
        assert!(service.get_average_response_time().is_none());

        service.register_module("module1".to_string());
        service.register_module("module2".to_string());

        // Manually set response times
        let mut health1 = ModuleHealth::healthy("module1".to_string());
        health1.response_time_ms = Some(100);
        service.set_module_health("module1", health1).unwrap();

        let mut health2 = ModuleHealth::healthy("module2".to_string());
        health2.response_time_ms = Some(200);
        service.set_module_health("module2", health2).unwrap();

        let avg_time = service.get_average_response_time().unwrap();
        assert_eq!(avg_time, 150.0);
    }
}