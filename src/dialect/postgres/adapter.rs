//! PostgreSQL adapter managing session-level advisory locks for migration concurrency.

use crate::dialect::adapter::DialectAdapter;
use crate::driver::DriverError;
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;
use async_trait::async_trait;

/// Kysely upstream standard 64-bit advisory lock ID for migrations.
// @spec REQ-MIG-005
pub const KYSELY_MIGRATION_LOCK_ID: i64 = 3853314791062309107;

/// PostgreSQL adapter managing session-level advisory locks for migration concurrency.
// @spec REQ-MIG-005, REQ-MIG-006
#[derive(Debug, Clone, Copy, Default)]
pub struct PostgresAdapter;

#[async_trait]
impl DialectAdapter for PostgresAdapter {
    async fn acquire_migration_lock(
        &self,
        executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        let sql = format!("SELECT pg_advisory_lock({})", KYSELY_MIGRATION_LOCK_ID);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }

    async fn release_migration_lock(
        &self,
        executor: &dyn QueryExecutor,
    ) -> Result<(), DriverError> {
        let sql = format!("SELECT pg_advisory_unlock({})", KYSELY_MIGRATION_LOCK_ID);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        executor.execute_query(&query).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::kysely::Kysely;
    use crate::query_compiler::DefaultQueryCompiler;
    use crate::query_executor::QueryResult;
    use std::sync::{Arc, Mutex};

    struct LockRecordingConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for LockRecordingConnection {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct LockRecordingDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for LockRecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(LockRecordingConnection {
                queries: Arc::clone(&self.queries),
            }))
        }

        async fn release_connection(
            &self,
            _connection: Box<dyn DatabaseConnection>,
        ) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_postgres_advisory_lock() {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(LockRecordingDriver {
            queries: Arc::clone(&recorded),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        let adapter = PostgresAdapter;

        // 1. Acquire
        adapter
            .acquire_migration_lock(db.executor().as_ref())
            .await
            .unwrap();

        // 2. Release
        adapter
            .release_migration_lock(db.executor().as_ref())
            .await
            .unwrap();

        let queries = recorded.lock().unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0], "SELECT pg_advisory_lock(3853314791062309107)");
        assert_eq!(queries[1], "SELECT pg_advisory_unlock(3853314791062309107)");
    }
}
