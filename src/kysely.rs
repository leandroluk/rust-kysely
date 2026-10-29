//! Main Kysely database client and builder orchestration facade.

use std::sync::Arc;
use compact_str::CompactString;

use crate::dialect::Dialect;
use crate::driver::DriverError;
use crate::parser::IntoTableSource;
use crate::query_builder::{
    delete_from, insert_into, select_from, DeleteQueryBuilder, InsertQueryBuilder,
    SelectQueryBuilder, UpdateQueryBuilder, update_table,
};
use crate::query_compiler::CompiledQuery;
use crate::query_executor::{DefaultQueryExecutor, QueryExecutor, QueryResult};

/// Main Kysely database client orchestrating query building and execution.
// @spec REQ-EXEC-003, REQ-EXEC-004
#[derive(Clone)]
pub struct Kysely {
    executor: Arc<dyn QueryExecutor>,
}

impl Kysely {
    /// Creates a new `Kysely` instance from a configured SQL dialect.
    pub fn new<D: Dialect>(dialect: D) -> Self {
        let driver = dialect.create_driver();
        let compiler = dialect.create_query_compiler();
        let executor = Arc::new(DefaultQueryExecutor::new(driver, compiler));
        Self { executor }
    }

    /// Creates a new `Kysely` instance wrapping an explicit query executor.
    pub fn with_executor(executor: Arc<dyn QueryExecutor>) -> Self {
        Self { executor }
    }

    /// Returns a reference to the active query executor.
    pub fn executor(&self) -> &Arc<dyn QueryExecutor> {
        &self.executor
    }

    /// Starts building a SELECT query with a FROM table source.
    pub fn select_from(&self, table: impl IntoTableSource) -> SelectQueryBuilder {
        select_from(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a SELECT query selecting all columns (`*`) from a table.
    pub fn select_all_from(&self, table: impl Into<CompactString>) -> SelectQueryBuilder {
        let tbl = table.into();
        select_from(tbl.as_str())
            .select_all()
            .with_executor(Arc::clone(&self.executor))
    }

    /// Starts building an INSERT statement.
    pub fn insert_into(&self, table: impl IntoTableSource) -> InsertQueryBuilder {
        insert_into(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building an UPDATE statement.
    pub fn update_table(&self, table: impl IntoTableSource) -> UpdateQueryBuilder {
        update_table(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a DELETE statement.
    pub fn delete_from(&self, table: impl IntoTableSource) -> DeleteQueryBuilder {
        delete_from(table).with_executor(Arc::clone(&self.executor))
    }

    /// Executes an already compiled query directly.
    pub async fn execute_query(&self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
        self.executor.execute_query(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::operation_node::Value;
    use crate::query_compiler::DefaultQueryCompiler;
    use crate::query_executor::Row;

    struct MockConnection {
        rows: Vec<Row>,
    }

    #[async_trait]
    impl DatabaseConnection for MockConnection {
        async fn execute_query(&mut self, _query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            Ok(QueryResult::with_rows(self.rows.clone()))
        }
    }

    struct MockDriver {
        rows: Vec<Row>,
    }

    #[async_trait]
    impl Driver for MockDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockConnection {
                rows: self.rows.clone(),
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
    async fn test_kysely_builder_execution() {
        let rows = vec![
            Row::new(
                vec!["id", "name", "active"],
                vec![Value::I64(1), Value::Text("Alice".into()), Value::Bool(true)],
            ),
            Row::new(
                vec!["id", "name", "active"],
                vec![Value::I64(2), Value::Text("Bob".into()), Value::Bool(false)],
            ),
        ];

        let driver = Arc::new(MockDriver { rows });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        // 1. SELECT execution
        let res = db.select_from("users").select(["id", "name"]).execute().await.unwrap();
        assert_eq!(res.len(), 2);

        // 2. SELECT execute_take_first
        let first = db
            .select_from("users")
            .select(["id", "name"])
            .execute_take_first()
            .await
            .unwrap();
        assert!(first.is_some());
        assert_eq!(first.unwrap().get::<String>("name").unwrap(), "Alice");

        // 3. SELECT execute_into
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct User {
            id: i64,
            name: String,
            active: bool,
        }

        let users: Vec<User> = db
            .select_from("users")
            .select(["id", "name", "active"])
            .execute_into()
            .await
            .unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[1].id, 2);

        // 4. SELECT execute_take_first_into
        let first_user: Option<User> = db
            .select_from("users")
            .select(["id", "name", "active"])
            .execute_take_first_into()
            .await
            .unwrap();
        assert_eq!(first_user.unwrap().name, "Alice");

        // 5. INSERT execution
        let insert_res = db
            .insert_into("users")
            .set("name", "Charlie")
            .execute()
            .await
            .unwrap();
        assert_eq!(insert_res.len(), 2);

        // 6. UPDATE execution
        let update_res = db
            .update_table("users")
            .set("active", true)
            .where_("id", "=", 1)
            .execute()
            .await
            .unwrap();
        assert_eq!(update_res.len(), 2);

        // 7. DELETE execution
        let delete_res = db
            .delete_from("users")
            .where_("id", "=", 2)
            .execute()
            .await
            .unwrap();
        assert_eq!(delete_res.len(), 2);
    }
}
