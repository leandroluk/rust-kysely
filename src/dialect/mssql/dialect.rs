//! Microsoft SQL Server (MSSQL) dialect implementation.

use std::sync::Arc;

use crate::dialect::adapter::DialectAdapter;
use crate::dialect::mssql::adapter::MssqlAdapter;
use crate::dialect::mssql::query_compiler::MssqlQueryCompiler;
use crate::dialect::Dialect;
use crate::driver::Driver;
use crate::query_compiler::QueryCompiler;

/// Dialect adapter for Microsoft SQL Server (MSSQL) databases.
// @spec REQ-DIA-002, REQ-DIA-005
pub struct MssqlDialect {
    driver: Arc<dyn Driver>,
}

impl MssqlDialect {
    /// Creates a new `MssqlDialect` wrapping an existing driver.
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }
}

impl Dialect for MssqlDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::new(MssqlQueryCompiler::new())
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(MssqlAdapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::mssql::introspector::MssqlIntrospector;
    use crate::driver::{DatabaseConnection, DriverError};
    use crate::introspection::DatabaseIntrospector;
    use crate::kysely::Kysely;
    use crate::operation_node::Value;
    use crate::query_compiler::CompiledQuery;
    use crate::query_executor::{QueryResult, Row};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockMssqlConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for MockMssqlConnection {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());

            if query.sql.contains("INFORMATION_SCHEMA.TABLES") {
                let rows = vec![Row::new(
                    ["table_name", "table_schema", "table_type"],
                    [
                        Value::Text("customers".into()),
                        Value::Text("dbo".into()),
                        Value::Text("BASE TABLE".into()),
                    ],
                )];
                return Ok(QueryResult::with_rows(rows));
            }

            if query.sql.contains("INFORMATION_SCHEMA.COLUMNS") {
                let rows = vec![
                    Row::new(
                        [
                            "table_name",
                            "column_name",
                            "data_type",
                            "is_nullable",
                            "column_default",
                        ],
                        [
                            Value::Text("customers".into()),
                            Value::Text("id".into()),
                            Value::Text("int".into()),
                            Value::Text("NO".into()),
                            Value::Null,
                        ],
                    ),
                    Row::new(
                        [
                            "table_name",
                            "column_name",
                            "data_type",
                            "is_nullable",
                            "column_default",
                        ],
                        [
                            Value::Text("customers".into()),
                            Value::Text("company".into()),
                            Value::Text("nvarchar".into()),
                            Value::Text("YES".into()),
                            Value::Null,
                        ],
                    ),
                ];
                return Ok(QueryResult::with_rows(rows));
            }

            Ok(QueryResult::default())
        }
    }

    struct MockMssqlDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for MockMssqlDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockMssqlConnection {
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
    async fn test_mssql_dialect() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(MockMssqlDriver {
            queries: Arc::clone(&queries),
        });
        let dialect = MssqlDialect::new(driver);
        let db = Kysely::new(dialect);

        // 1. SELECT query: parameters @p1, @p2 and identifiers [bracketed]
        let _ = db
            .select_from("customers")
            .select(["id", "company"])
            .where_("company", "=", "Acme Corp")
            .where_("id", ">", Value::I64(10))
            .execute()
            .await;

        {
            let recorded = queries.lock().unwrap();
            assert_eq!(recorded.len(), 1);
            let sql = &recorded[0];
            assert_eq!(
                sql,
                "SELECT [id], [company] FROM [customers] WHERE [company] = @p1 AND [id] > @p2"
            );
        }

        // 2. Introspection via MssqlIntrospector
        let introspector = MssqlIntrospector::new(db.executor().as_ref());
        let tables = introspector.get_tables().await.unwrap();

        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "customers");
        assert_eq!(tables[0].schema.as_deref(), Some("dbo"));
        assert_eq!(tables[0].columns.len(), 2);
        assert_eq!(tables[0].columns[0].name, "id");
        assert_eq!(tables[0].columns[1].name, "company");

        // 3. Advisory lock test via sp_getapplock
        let adapter = MssqlAdapter;
        adapter
            .acquire_migration_lock(db.executor().as_ref())
            .await
            .unwrap();
        adapter
            .release_migration_lock(db.executor().as_ref())
            .await
            .unwrap();

        {
            let recorded = queries.lock().unwrap();
            assert_eq!(recorded.len(), 5);
            assert!(recorded[3].contains("sp_getapplock"));
            assert!(recorded[4].contains("sp_releaseapplock"));
        }
    }
}
