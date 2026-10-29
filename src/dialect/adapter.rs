//! Dialect-specific adapter traits for migration locking and database capabilities.

use crate::driver::DriverError;
use crate::query_executor::QueryExecutor;
use async_trait::async_trait;

/// Async trait defining dialect-specific capabilities such as distributed advisory locking.
// @spec REQ-MIG-005, REQ-MIG-006
#[async_trait]
pub trait DialectAdapter: Send + Sync {
    /// Acquires the migration lock across application instances.
    async fn acquire_migration_lock(&self, executor: &dyn QueryExecutor)
        -> Result<(), DriverError>;

    /// Releases the acquired migration lock.
    async fn release_migration_lock(&self, executor: &dyn QueryExecutor)
        -> Result<(), DriverError>;
}

/// Default no-op adapter for dialects that do not support or require advisory locking.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultAdapter;

#[async_trait]
impl DialectAdapter for DefaultAdapter {
    async fn acquire_migration_lock(
        &self,
        _executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        Ok(())
    }

    async fn release_migration_lock(
        &self,
        _executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        Ok(())
    }
}
