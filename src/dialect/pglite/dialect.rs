//! PGlite (embedded PostgreSQL) dialect implementation.

use std::sync::Arc;

use crate::dialect::adapter::DialectAdapter;
use crate::dialect::postgres::adapter::PostgresAdapter;
use crate::dialect::postgres::query_compiler::PostgresQueryCompiler;
use crate::dialect::Dialect;
use crate::driver::Driver;
use crate::query_compiler::QueryCompiler;

/// Dialect adapter for PGlite (embedded PostgreSQL engine), reusing PostgreSQL compiler and adapters.
// @spec REQ-DIA-004, REQ-DIA-005
#[derive(Clone)]
pub struct PGliteDialect {
    driver: Arc<dyn Driver>,
}

impl PGliteDialect {
    /// Creates a new `PGliteDialect` wrapping an embedded or custom driver.
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }
}

impl Dialect for PGliteDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::new(PostgresQueryCompiler::new())
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(PostgresAdapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use crate::driver::{DatabaseConnection, DriverError};
    use crate::kysely::Kysely;
    use crate::query_compiler::CompiledQuery;
    use crate::query_executor::QueryResult;

    struct MockPGliteConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for MockPGliteConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct MockPGliteDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for MockPGliteDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockPGliteConnection {
                queries: Arc::clone(&self.queries),
            }))
        }

        async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_pglite_dialect() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(MockPGliteDriver {
            queries: Arc::clone(&queries),
        });
        let dialect = PGliteDialect::new(driver);
        let db = Kysely::new(dialect);

        // 1. SELECT query: verify PostgreSQL $1 parameter style and double quoting
        let _ = db
            .select_from("settings")
            .select(["key", "val"])
            .where_("key", "=", "theme")
            .execute()
            .await;

        let recorded = queries.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        let sql = &recorded[0];
        assert_eq!(sql, "SELECT \"key\", \"val\" FROM \"settings\" WHERE \"key\" = $1");

        // 2. Adapter: verify PostgreSQL advisory lock ID 3853314791062309107
        drop(recorded);
        let adapter = db.adapter();
        adapter.acquire_migration_lock(db.executor().as_ref()).await.unwrap();
        adapter.release_migration_lock(db.executor().as_ref()).await.unwrap();

        let recorded = queries.lock().unwrap();
        assert_eq!(recorded.len(), 3);
        assert!(recorded[1].contains("pg_advisory_lock(3853314791062309107)"));
        assert!(recorded[2].contains("pg_advisory_unlock(3853314791062309107)"));
    }
}
