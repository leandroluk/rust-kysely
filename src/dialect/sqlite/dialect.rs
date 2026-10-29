//! SQLite dialect implementation.

use std::sync::Arc;

use crate::dialect::adapter::DialectAdapter;
use crate::dialect::sqlite::adapter::SqliteAdapter;
use crate::dialect::sqlite::query_compiler::SqliteQueryCompiler;
use crate::dialect::Dialect;
use crate::driver::Driver;
use crate::query_compiler::QueryCompiler;

/// Dialect adapter for SQLite databases.
// @spec REQ-DIA-003, REQ-DIA-005
pub struct SqliteDialect {
    driver: Arc<dyn Driver>,
}

impl SqliteDialect {
    /// Creates a new `SqliteDialect` wrapping an existing driver.
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }
}

impl Dialect for SqliteDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::new(SqliteQueryCompiler::new())
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(SqliteAdapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::sqlite::introspector::SqliteIntrospector;
    use crate::driver::{DatabaseConnection, DriverError};
    use crate::introspection::DatabaseIntrospector;
    use crate::kysely::Kysely;
    use crate::operation_node::Value;
    use crate::query_compiler::CompiledQuery;
    use crate::query_executor::{QueryResult, Row};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockSqliteConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for MockSqliteConnection {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());

            if query.sql.contains("sqlite_master") {
                let rows = vec![Row::new(
                    ["name", "type"],
                    [Value::Text("users".into()), Value::Text("table".into())],
                )];
                return Ok(QueryResult::with_rows(rows));
            }

            if query.sql.contains("PRAGMA table_info") {
                let rows = vec![
                    Row::new(
                        ["cid", "name", "type", "notnull", "dflt_value", "pk"],
                        [
                            Value::I64(0),
                            Value::Text("id".into()),
                            Value::Text("INTEGER".into()),
                            Value::I64(1),
                            Value::Null,
                            Value::I64(1),
                        ],
                    ),
                    Row::new(
                        ["cid", "name", "type", "notnull", "dflt_value", "pk"],
                        [
                            Value::I64(1),
                            Value::Text("email".into()),
                            Value::Text("TEXT".into()),
                            Value::I64(0),
                            Value::Null,
                            Value::I64(0),
                        ],
                    ),
                ];
                return Ok(QueryResult::with_rows(rows));
            }

            Ok(QueryResult::default())
        }
    }

    struct MockSqliteDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for MockSqliteDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockSqliteConnection {
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
    async fn test_sqlite_dialect() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(MockSqliteDriver {
            queries: Arc::clone(&queries),
        });
        let dialect = SqliteDialect::new(driver);
        let db = Kysely::new(dialect);

        // 1. Query compilation: parameters should be ? and identifiers double quoted
        let _ = db
            .select_from("users")
            .select(["id", "email"])
            .where_("email", "=", "user@example.com")
            .execute()
            .await;

        {
            let recorded = queries.lock().unwrap();
            assert_eq!(recorded.len(), 1);
            let sql = &recorded[0];
            assert_eq!(
                sql,
                "SELECT \"id\", \"email\" FROM \"users\" WHERE \"email\" = ?"
            );
        }

        // 2. Introspection via SqliteIntrospector
        let introspector = SqliteIntrospector::new(db.executor().as_ref());
        let tables = introspector.get_tables().await.unwrap();

        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "users");
        assert_eq!(tables[0].columns.len(), 2);
        assert_eq!(tables[0].columns[0].name, "id");
        assert!(tables[0].columns[0].is_auto_incrementing);
        assert_eq!(tables[0].columns[1].name, "email");
        assert!(tables[0].columns[1].is_nullable);
    }
}
