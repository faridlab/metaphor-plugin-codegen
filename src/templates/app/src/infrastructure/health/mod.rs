//! Health Monitoring Infrastructure
//!
//! Infrastructure components for monitoring system health, performance metrics,
//! and providing health check endpoints.

use crate::domain::services::{ModuleHealthService, AuditService};
use crate::shared::error::AppResult;
use metaphor_health::{HealthChecker, HealthConfig, HealthReport, HealthStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Health monitoring service
pub struct HealthMonitoringService {
    health_checker: HealthChecker,
    module_health_service: Arc<ModuleHealthService>,
    audit_service: Option<Arc<AuditService>>,
    metrics: Arc<RwLock<HealthMetrics>>,
    config: HealthMonitoringConfig,
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthMonitoringConfig {
    pub check_interval_secs: u64,
    pub retention_hours: u32,
    pub enable_detailed_checks: bool,
    pub enable_metrics: bool,
    pub alert_thresholds: AlertThresholds,
}

/// Alert thresholds for health monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub error_rate_percent: f64,
    pub response_time_ms: u64,
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            retention_hours: 24,
            enable_detailed_checks: true,
            enable_metrics: true,
            alert_thresholds: AlertThresholds {
                cpu_usage_percent: 80.0,
                memory_usage_percent: 85.0,
                disk_usage_percent: 90.0,
                error_rate_percent: 5.0,
                response_time_ms: 2000,
            },
        }
    }
}

/// Health metrics collection
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub average_response_time_ms: f64,
    pub last_check_time: Option<chrono::DateTime<chrono::Utc>>,
    pub component_health: HashMap<String, ComponentHealthMetrics>,
}

/// Component health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthMetrics {
    pub component_name: String,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub failure_rate_percent: f64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub service_name: String,
    pub overall_status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub components: Vec<ComponentHealth>,
    pub metrics: HealthMetrics,
    pub uptime_seconds: u64,
    pub alerts: Vec<HealthAlert>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub response_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Alert type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    ComponentDown,
    HighErrorRate,
    SlowResponse,
    HighResourceUsage,
    ConfigurationError,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl HealthMonitoringService {
    pub fn new(
        health_checker: HealthChecker,
        module_health_service: Arc<ModuleHealthService>,
        audit_service: Option<Arc<AuditService>>,
        config: HealthMonitoringConfig,
    ) -> Self {
        Self {
            health_checker,
            module_health_service,
            audit_service,
            metrics: Arc::new(RwLock::new(HealthMetrics::default())),
            config,
        }
    }

    /// Perform comprehensive health check
    pub async fn comprehensive_health_check(&self) -> AppResult<HealthCheckResult> {
        let start_time = Instant::now();

        // Get basic health report
        let health_report = self.health_checker.health_report().await;

        // Get module-specific health
        let module_health = self.module_health_service.get_module_overview().await?;

        // Calculate overall status
        let overall_status = self.calculate_overall_status(&health_report, &module_health);

        // Update metrics
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        self.update_metrics(response_time_ms, &health_report).await;

        // Check for alerts
        let alerts = self.check_for_alerts(&health_report, &module_health).await?;

        // Get metrics snapshot
        let metrics = self.metrics.read().await.clone();

        let result = HealthCheckResult {
            service_name: "metaphor-framework".to_string(),
            overall_status,
            timestamp: chrono::Utc::now(),
            components: self.build_component_health(&health_report, &module_health),
            metrics,
            uptime_seconds: self.calculate_uptime(),
            alerts,
        };

        Ok(result)
    }

    /// Get health status for specific component
    pub async fn component_health_check(&self, component_name: &str) -> AppResult<ComponentHealth> {
        let health_report = self.health_checker.health_report().await;

        if let Some(component) = health_report.components.get(component_name) {
            Ok(ComponentHealth {
                name: component_name.to_string(),
                status: component.status,
                message: component.message.clone(),
                response_time_ms: component.response_time_ms.unwrap_or(0),
                metadata: component.details.clone(),
            })
        } else {
            Ok(ComponentHealth {
                name: component_name.to_string(),
                status: HealthStatus::Unhealthy,
                message: "Component not found".to_string(),
                response_time_ms: 0,
                metadata: HashMap::new(),
            })
        }
    }

    /// Acknowledge health alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: Uuid) -> AppResult<()> {
        // In a real implementation, this would persist the acknowledgment
        if let Some(audit_service) = &self.audit_service {
            // Log alert acknowledgment
            audit_service.log_system_event(
                format!("Health alert {} acknowledged", alert_id),
                "health".to_string(),
                Some(acknowledged_by.to_string()),
            ).await?;
        }

        Ok(())
    }

    /// Get health metrics
    pub async fn get_health_metrics(&self) -> AppResult<HealthMetrics> {
        let metrics = self.metrics.read().await.clone();
        Ok(metrics)
    }

    /// Reset health metrics
    pub async fn reset_metrics(&self) -> AppResult<()> {
        let mut metrics = self.metrics.write().await;
        *metrics = HealthMetrics::default();
        Ok(())
    }

    /// Start background health monitoring
    pub async fn start_monitoring(&self) -> AppResult<()> {
        let health_service = self.clone();
        let interval = Duration::from_secs(self.config.check_interval_secs);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = health_service.perform_background_check().await {
                    eprintln!("Background health check failed: {:?}", e);
                }
            }
        });

        Ok(())
    }

    async fn perform_background_check(&self) -> AppResult<()> {
        // Perform health check in background
        let _result = self.comprehensive_health_check().await?;

        // Log results if enabled
        if let Some(audit_service) = &self.audit_service {
            audit_service.log_system_event(
                "Background health check completed".to_string(),
                "health".to_string(),
                None,
            ).await?;
        }

        Ok(())
    }

    fn calculate_overall_status(
        &self,
        health_report: &HealthReport,
        module_health: &crate::domain::services::ModuleOverview,
    ) -> HealthStatus {
        // Check if any critical components are down
        if health_report.components.values().any(|c| c.status == HealthStatus::Unhealthy) {
            return HealthStatus::Unhealthy;
        }

        // Check if any components are degraded
        if health_report.components.values().any(|c| c.status == HealthStatus::Degraded) {
            return HealthStatus::Degraded;
        }

        // Check module health
        let total_modules = module_health.active_modules + module_health.inactive_modules;
        if total_modules > 0 && module_health.inactive_modules > 0 {
            let inactive_percentage = (module_health.inactive_modules as f64 / total_modules as f64) * 100.0;
            if inactive_percentage > 20.0 {
                return HealthStatus::Degraded;
            }
        }

        HealthStatus::Healthy
    }

    async fn update_metrics(&self, response_time_ms: u64, health_report: &HealthReport) {
        let mut metrics = self.metrics.write().await;

        metrics.total_checks += 1;
        metrics.last_check_time = Some(chrono::Utc::now());

        if health_report.status == HealthStatus::Healthy {
            metrics.successful_checks += 1;
        } else {
            metrics.failed_checks += 1;
        }

        // Update average response time
        metrics.average_response_time_ms = (metrics.average_response_time_ms * (metrics.total_checks - 1) as f64 + response_time_ms as f64) / metrics.total_checks as f64;

        // Update component metrics
        for (name, component) in &health_report.components {
            let component_metrics = metrics.component_health.entry(name.clone()).or_insert_with(|| ComponentHealthMetrics {
                component_name: name.clone(),
                status: component.status,
                last_check: chrono::Utc::now(),
                response_time_ms: component.response_time_ms.unwrap_or(0),
                consecutive_failures: if component.status == HealthStatus::Healthy { 0 } else { 1 },
                total_checks: 1,
                failure_rate_percent: if component.status == HealthStatus::Healthy { 0.0 } else { 100.0 },
            });

            component_metrics.status = component.status;
            component_metrics.last_check = chrono::Utc::now();
            component_metrics.response_time_ms = component.response_time_ms.unwrap_or(0);
            component_metrics.total_checks += 1;

            if component.status == HealthStatus::Healthy {
                component_metrics.consecutive_failures = 0;
            } else {
                component_metrics.consecutive_failures += 1;
            }

            component_metrics.failure_rate_percent = (component_metrics.consecutive_failures as f64 / component_metrics.total_checks as f64) * 100.0;
        }
    }

    async fn check_for_alerts(
        &self,
        health_report: &HealthReport,
        _module_health: &crate::domain::services::ModuleOverview,
    ) -> AppResult<Vec<HealthAlert>> {
        let mut alerts = Vec::new();

        // Check for component down alerts
        for (name, component) in &health_report.components {
            if component.status == HealthStatus::Unhealthy {
                alerts.push(HealthAlert {
                    id: Uuid::new_v4(),
                    alert_type: AlertType::ComponentDown,
                    severity: AlertSeverity::Critical,
                    component: name.clone(),
                    message: format!("Component {} is unhealthy: {}", name, component.message),
                    triggered_at: chrono::Utc::now(),
                    acknowledged_at: None,
                    resolved_at: None,
                });
            }

            // Check for slow response alerts
            if let Some(response_time) = component.response_time_ms {
                if response_time > self.config.alert_thresholds.response_time_ms {
                    alerts.push(HealthAlert {
                        id: Uuid::new_v4(),
                        alert_type: AlertType::SlowResponse,
                        severity: AlertSeverity::Warning,
                        component: name.clone(),
                        message: format!("Component {} response time is {}ms (threshold: {}ms)", name, response_time, self.config.alert_thresholds.response_time_ms),
                        triggered_at: chrono::Utc::now(),
                        acknowledged_at: None,
                        resolved_at: None,
                    });
                }
            }
        }

        Ok(alerts)
    }

    fn build_component_health(
        &self,
        health_report: &HealthReport,
        module_health: &crate::domain::services::ModuleOverview,
    ) -> Vec<ComponentHealth> {
        let mut components = Vec::new();

        // Add health checker components
        for (name, component) in &health_report.components {
            components.push(ComponentHealth {
                name: name.clone(),
                status: component.status,
                message: component.message.clone(),
                response_time_ms: component.response_time_ms.unwrap_or(0),
                metadata: component.details.clone(),
            });
        }

        // Add module status as a component
        components.push(ComponentHealth {
            name: "modules".to_string(),
            status: if module_health.inactive_modules == 0 {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            },
            message: format!("{} active, {} inactive modules", module_health.active_modules, module_health.inactive_modules),
            response_time_ms: 0,
            metadata: serde_json::json!({
                "total_modules": module_health.active_modules + module_health.inactive_modules,
                "active_modules": module_health.active_modules,
                "inactive_modules": module_health.inactive_modules
            }).as_object().unwrap().iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        });

        components
    }

    fn calculate_uptime(&self) -> u64 {
        // In a real implementation, this would track actual service start time
        // For now, return a placeholder
        3600 // 1 hour
    }
}

// Implement Clone for HealthMonitoringService for background tasks
impl Clone for HealthMonitoringService {
    fn clone(&self) -> Self {
        Self {
            health_checker: self.health_checker.clone(),
            module_health_service: self.module_health_service.clone(),
            audit_service: self.audit_service.clone(),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitoring_config() {
        let config = HealthMonitoringConfig::default();
        assert_eq!(config.check_interval_secs, 60);
        assert_eq!(config.retention_hours, 24);
        assert!(config.enable_detailed_checks);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_health_alert() {
        let alert = HealthAlert {
            id: Uuid::new_v4(),
            alert_type: AlertType::ComponentDown,
            severity: AlertSeverity::Critical,
            component: "database".to_string(),
            message: "Database connection failed".to_string(),
            triggered_at: chrono::Utc::now(),
            acknowledged_at: None,
            resolved_at: None,
        };

        assert_eq!(alert.component, "database");
        assert!(matches!(alert.alert_type, AlertType::ComponentDown));
        assert!(matches!(alert.severity, AlertSeverity::Critical));
        assert!(alert.acknowledged_at.is_none());
    }

    #[test]
    fn test_component_health() {
        let component = ComponentHealth {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            message: "Connection successful".to_string(),
            response_time_ms: 150,
            metadata: HashMap::new(),
        };

        assert_eq!(component.name, "database");
        assert!(matches!(component.status, HealthStatus::Healthy));
        assert_eq!(component.response_time_ms, 150);
    }

    #[tokio::test]
    async fn test_health_metrics_update() {
        let metrics = Arc::new(RwLock::new(HealthMetrics::default()));

        // Simulate metric updates
        {
            let mut m = metrics.write().await;
            m.total_checks = 10;
            m.successful_checks = 9;
            m.average_response_time_ms = 150.0;
        }

        let m = metrics.read().await;
        assert_eq!(m.total_checks, 10);
        assert_eq!(m.successful_checks, 9);
        assert_eq!(m.average_response_time_ms, 150.0);
    }
}