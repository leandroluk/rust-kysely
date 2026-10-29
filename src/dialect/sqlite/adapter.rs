//! SQLite adapter for migration locking.

use async_trait::async_trait;
use crate::dialect::adapter::DialectAdapter;
use crate::driver::DriverError;
use crate::query_executor::QueryExecutor;

/// SQLite adapter for concurrency and migration locking (no-op as SQLite utilizes file-level locking).
// @spec REQ-DIA-003, REQ-DIA-006
#[derive(Debug, Clone, Copy, Default)]
pub struct SqliteAdapter;

#[async_trait]
impl DialectAdapter for SqliteAdapter {
    async fn acquire_migration_lock(&self, _executor: &dyn QueryExecutor) -> Result<(), DriverError> {
        Ok(())
    }

    async fn release_migration_lock(&self, _executor: &dyn QueryExecutor) -> Result<(), DriverError> {
        Ok(())
    }
}
