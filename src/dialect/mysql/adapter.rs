//! MySQL adapter managing GET_LOCK / RELEASE_LOCK migration concurrency.

use crate::dialect::adapter::DialectAdapter;
use crate::driver::DriverError;
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;
use async_trait::async_trait;

/// Default MySQL migration lock name.
pub const MYSQL_MIGRATION_LOCK_NAME: &str = "kysely_migration_lock";

/// MySQL adapter managing distributed migration locks using `GET_LOCK` and `RELEASE_LOCK`.
// @spec REQ-DIA-001, REQ-DIA-006
#[derive(Debug, Clone, Copy, Default)]
pub struct MysqlAdapter;

#[async_trait]
impl DialectAdapter for MysqlAdapter {
    async fn acquire_migration_lock(
        &self,
        executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        let sql = format!("SELECT GET_LOCK('{}', 60)", MYSQL_MIGRATION_LOCK_NAME);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }

    async fn release_migration_lock(
        &self,
        executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        let sql = format!("SELECT RELEASE_LOCK('{}')", MYSQL_MIGRATION_LOCK_NAME);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }
}
