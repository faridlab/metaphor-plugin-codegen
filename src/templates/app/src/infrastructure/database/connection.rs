//! Database Connection Management

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use crate::config::DatabaseConfig;
use crate::shared::error::{AppError, AppResult};

/// Database connection manager
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with the given configuration
    pub async fn new(config: &DatabaseConfig) -> AppResult<Self> {
        tracing::info!("🗄️ Initializing database connection pool");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect(&config.url)
            .await
            .map_err(|e| {
                AppError::Database(e)
            })?;

        // Test the connection
        Self::test_connection(&pool).await?;

        tracing::info!("✅ Database connection pool initialized successfully");
        tracing::info!("🔢 Pool configuration: min={}, max={}", config.min_connections, config.max_connections);

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get database connection pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        tracing::info!("🔌 Closing database connection pool");
        self.pool.close().await;
    }

    /// Test database connection
    async fn test_connection(pool: &PgPool) -> AppResult<()> {
        tracing::debug!("🔍 Testing database connection");

        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        tracing::debug!("✅ Database connection test successful");
        Ok(())
    }

    /// Run database health check
    pub async fn health_check(&self) -> DatabaseHealth {
        let start = std::time::Instant::now();

        match self.test_connection(&self.pool).await {
            Ok(_) => DatabaseHealth {
                status: DatabaseStatus::Healthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                error: None,
                stats: self.pool_stats(),
            },
            Err(e) => DatabaseHealth {
                status: DatabaseStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                stats: self.pool_stats(),
            },
        }
    }

    /// Get connection for a specific module (if using multiple databases)
    pub async fn get_module_connection(&self, _module: &str) -> AppResult<sqlx::PgPool> {
        // In a real implementation, this might return different pools
        // for different modules if they have separate databases
        Ok(self.pool.clone())
    }
}

/// Database connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
}

impl PoolStats {
    pub fn active(&self) -> u32 {
        self.size - self.idle
    }

    pub fn utilization_percent(&self) -> f64 {
        if self.size == 0 {
            return 0.0;
        }
        (self.active() as f64 / self.size as f64) * 100.0
    }
}

/// Database health status
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub status: DatabaseStatus,
    pub response_time_ms: u64,
    pub error: Option<String>,
    pub stats: PoolStats,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Database transaction helper
pub struct TransactionManager;

impl TransactionManager {
    /// Execute a function within a database transaction
    pub async fn execute_in_transaction<F, R>(
        pool: &PgPool,
        f: F,
    ) -> AppResult<R>
    where
        F: for<'tx> FnOnce(&'tx mut sqlx::Transaction<'tx, sqlx::Postgres>) -> futures::future::BoxFuture<'tx, AppResult<R>>,
        R: Send + 'static,
    {
        let mut tx = pool.begin().await
            .map_err(|e| AppError::Database(e))?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await
                    .map_err(|e| AppError::Database(e))?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|rollback_err| {
                        AppError::Database(rollback_err)
                    })?;
                Err(e)
            }
        }
    }

    /// Execute multiple operations in a transaction with automatic rollback on error
    pub async fn execute_operations<F>(
        pool: &PgPool,
        operations: Vec<Operation>,
    ) -> AppResult<Vec<OperationResult>> {
        Self::execute_in_transaction(pool, |tx| {
            Box::pin(async move {
                let mut results = Vec::new();

                for operation in operations {
                    let result = match operation {
                        Operation::Query { query, params } => {
                            Self::execute_query(tx, &query, params).await
                        }
                        Operation::Execute { sql, params } => {
                            Self::execute_sql(tx, &sql, params).await
                        }
                    };

                    results.push(result);
                }

                Ok(results)
            })
        }).await
    }

    async fn execute_query(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        query: &str,
        _params: Vec<serde_json::Value>,
    ) -> OperationResult {
        sqlx::query(query)
            .fetch_all(&mut **tx)
            .await
            .map(|rows| {
                OperationResult::Success {
                    rows_affected: rows.len(),
                    data: Some(serde_json::to_value(&rows).unwrap_or_default()),
                }
            })
            .map_err(|e| OperationResult::Error {
                error: e.to_string(),
                sql: Some(query.to_string()),
            })
    }

    async fn execute_sql(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        sql: &str,
        _params: Vec<serde_json::Value>,
    ) -> OperationResult {
        sqlx::query(sql)
            .execute(&mut **tx)
            .await
            .map(|result| {
                OperationResult::Success {
                    rows_affected: result.rows_affected() as usize,
                    data: None,
                }
            })
            .map_err(|e| OperationResult::Error {
                error: e.to_string(),
                sql: Some(sql.to_string()),
            })
    }
}

/// Database operation for transaction management
#[derive(Debug, Clone)]
pub enum Operation {
    Query {
        query: String,
        params: Vec<serde_json::Value>,
    },
    Execute {
        sql: String,
        params: Vec<serde_json::Value>,
    },
}

/// Result of a database operation
#[derive(Debug, Clone)]
pub enum OperationResult {
    Success {
        rows_affected: usize,
        data: Option<serde_json::Value>,
    },
    Error {
        error: String,
        sql: Option<String>,
    },
}

impl OperationResult {
    pub fn is_success(&self) -> bool {
        matches!(self, OperationResult::Success { .. })
    }

    pub fn rows_affected(&self) -> usize {
        match self {
            OperationResult::Success { rows_affected, .. } => *rows_affected,
            OperationResult::Error { .. } => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            size: 10,
            idle: 3,
        };

        assert_eq!(stats.active(), 7);
        assert_eq!(stats.utilization_percent(), 70.0);
    }

    #[test]
    fn test_database_health() {
        let health = DatabaseHealth {
            status: DatabaseStatus::Healthy,
            response_time_ms: 45,
            error: None,
            stats: PoolStats { size: 10, idle: 5 },
        };

        assert_eq!(health.status, DatabaseStatus::Healthy);
        assert_eq!(health.response_time_ms, 45);
        assert!(health.error.is_none());
    }

    #[test]
    fn test_operation_result() {
        let success = OperationResult::Success {
            rows_affected: 5,
            data: None,
        };

        let error = OperationResult::Error {
            error: "Connection failed".to_string(),
            sql: Some("SELECT * FROM users".to_string()),
        };

        assert!(success.is_success());
        assert!(!error.is_success());
        assert_eq!(success.rows_affected(), 5);
        assert_eq!(error.rows_affected(), 0);
    }

    // Note: Integration tests would require a test database
    // These would test actual database connections and transactions
}