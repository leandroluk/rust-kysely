//! Microsoft SQL Server (MSSQL) adapter managing sp_getapplock migration concurrency.

use async_trait::async_trait;
use crate::dialect::adapter::DialectAdapter;
use crate::driver::DriverError;
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;

/// Default MSSQL migration lock resource name.
pub const MSSQL_MIGRATION_LOCK_NAME: &str = "kysely_migration_lock";

/// MSSQL adapter managing distributed session locks via `sp_getapplock` and `sp_releaseapplock`.
// @spec REQ-DIA-002, REQ-DIA-006
#[derive(Debug, Clone, Copy, Default)]
pub struct MssqlAdapter;

#[async_trait]
impl DialectAdapter for MssqlAdapter {
    async fn acquire_migration_lock(&self, executor: &dyn QueryExecutor) -> Result<(), DriverError> {
        let sql = format!(
            "EXEC sp_getapplock @Resource = '{}', @LockMode = 'Exclusive', @LockOwner = 'Session', @LockTimeout = 60000",
            MSSQL_MIGRATION_LOCK_NAME
        );
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }

    async fn release_migration_lock(&self, executor: &dyn QueryExecutor) -> Result<(), DriverError> {
        let sql = format!(
            "EXEC sp_releaseapplock @Resource = '{}', @LockOwner = 'Session'",
            MSSQL_MIGRATION_LOCK_NAME
        );
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }
}
