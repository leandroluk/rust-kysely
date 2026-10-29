//! MySQL dialect implementation.

use std::sync::Arc;

use crate::dialect::adapter::DialectAdapter;
use crate::dialect::mysql::adapter::MysqlAdapter;
use crate::dialect::mysql::query_compiler::MysqlQueryCompiler;
use crate::dialect::Dialect;
use crate::driver::Driver;
use crate::query_compiler::QueryCompiler;

/// Dialect adapter for MySQL databases.
// @spec REQ-DIA-001, REQ-DIA-005
pub struct MysqlDialect {
    driver: Arc<dyn Driver>,
}

impl MysqlDialect {
    /// Creates a new `MysqlDialect` wrapping an existing driver.
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }
}

impl Dialect for MysqlDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::new(MysqlQueryCompiler::new())
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(MysqlAdapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use crate::dialect::mysql::introspector::MysqlIntrospector;
    use crate::driver::{DatabaseConnection, DriverError};
    use crate::introspection::DatabaseIntrospector;
    use crate::kysely::Kysely;
    use crate::operation_node::Value;
    use crate::query_compiler::CompiledQuery;
    use crate::query_executor::{QueryResult, Row};

    struct MockMysqlConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for MockMysqlConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());

            if query.sql.contains("information_schema.tables") {
                let rows = vec![Row::new(
                    ["table_name", "table_schema", "table_type"],
                    [
                        Value::Text("users".into()),
                        Value::Text("mydb".into()),
                        Value::Text("BASE TABLE".into()),
                    ],
                )];
                return Ok(QueryResult::with_rows(rows));
            }

            if query.sql.contains("information_schema.columns") {
                let rows = vec![
                    Row::new(
                        ["table_name", "column_name", "data_type", "is_nullable", "column_default", "extra"],
                        [
                            Value::Text("users".into()),
                            Value::Text("id".into()),
                            Value::Text("int".into()),
                            Value::Text("NO".into()),
                            Value::Null,
                            Value::Text("auto_increment".into()),
                        ],
                    ),
                    Row::new(
                        ["table_name", "column_name", "data_type", "is_nullable", "column_default", "extra"],
                        [
                            Value::Text("users".into()),
                            Value::Text("email".into()),
                            Value::Text("varchar".into()),
                            Value::Text("YES".into()),
                            Value::Null,
                            Value::Text("".into()),
                        ],
                    ),
                ];
                return Ok(QueryResult::with_rows(rows));
            }

            Ok(QueryResult::default())
        }
    }

    struct MockMysqlDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for MockMysqlDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockMysqlConnection {
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
    async fn test_mysql_dialect() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(MockMysqlDriver {
            queries: Arc::clone(&queries),
        });
        let dialect = MysqlDialect::new(driver);
        let db = Kysely::new(dialect);

        // 1. SELECT query: parameters ? and identifiers backticked
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
            assert_eq!(sql, "SELECT `id`, `email` FROM `users` WHERE `email` = ?");
        }

        // 2. INSERT ON DUPLICATE KEY UPDATE
        let _ = db
            .insert_into("users")
            .values([("id", Value::I64(1)), ("email", Value::Text("alice@mysql.com".into()))])
            .on_conflict(|b| b.column("id").do_update_set(|u| u.set("email", "updated@mysql.com")))
            .execute()
            .await;

        {
            let recorded = queries.lock().unwrap();
            assert_eq!(recorded.len(), 2);
            let sql = &recorded[1];
            assert!(sql.contains("INSERT INTO `users`"), "Must have backtick table: {sql}");
            assert!(sql.contains("ON DUPLICATE KEY UPDATE `email` = ?"), "Must have MySQL upsert: {sql}");
        }

        // 3. Introspection via MysqlIntrospector
        let introspector = MysqlIntrospector::new(db.executor().as_ref());
        let tables = introspector.get_tables().await.unwrap();

        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "users");
        assert_eq!(tables[0].columns.len(), 2);
        assert_eq!(tables[0].columns[0].name, "id");
        assert!(tables[0].columns[0].is_auto_incrementing);
        assert!(!tables[0].columns[0].is_nullable);
        assert_eq!(tables[0].columns[1].name, "email");
        assert!(tables[0].columns[1].is_nullable);

        // 4. Advisory lock test
        let adapter = MysqlAdapter;
        adapter.acquire_migration_lock(db.executor().as_ref()).await.unwrap();
        adapter.release_migration_lock(db.executor().as_ref()).await.unwrap();

        {
            let recorded = queries.lock().unwrap();
            assert_eq!(recorded.len(), 6);
            assert_eq!(recorded[4], "SELECT GET_LOCK('kysely_migration_lock', 60)");
            assert_eq!(recorded[5], "SELECT RELEASE_LOCK('kysely_migration_lock')");
        }
    }
}
