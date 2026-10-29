//! PostgreSQL schema and catalog introspector.

use std::sync::Arc;
use async_trait::async_trait;

use crate::driver::DriverError;
use crate::introspection::{ColumnMetadata, DatabaseIntrospector, TableMetadata};
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;

/// PostgreSQL introspector querying `information_schema` and `pg_catalog`.
// @spec REQ-DDL-010
pub struct PostgresIntrospector {
    executor: Arc<dyn QueryExecutor>,
}

impl PostgresIntrospector {
    /// Creates a new `PostgresIntrospector` wrapping a query executor.
    pub fn new(executor: Arc<dyn QueryExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl DatabaseIntrospector for PostgresIntrospector {
    async fn get_schemas(&self) -> Result<Vec<String>, DriverError> {
        let sql = "SELECT schema_name FROM information_schema.schemata \
                   WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast') \
                   ORDER BY schema_name";
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        let result = self.executor.execute_query(&query).await?;

        let mut schemas = Vec::with_capacity(result.len());
        for row in result.rows {
            schemas.push(row.get::<String>("schema_name").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?);
        }
        Ok(schemas)
    }

    async fn get_tables(&self) -> Result<Vec<TableMetadata>, DriverError> {
        let tables_sql = "SELECT table_name, table_schema, table_type \
                          FROM information_schema.tables \
                          WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast') \
                          ORDER BY table_name";
        let tables_query = CompiledQuery::new(tables_sql, vec![], QueryId::generate());
        let tables_result = self.executor.execute_query(&tables_query).await?;

        let columns_sql = "SELECT table_name, column_name, data_type, is_nullable, column_default \
                           FROM information_schema.columns \
                           WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast') \
                           ORDER BY table_name, ordinal_position";
        let columns_query = CompiledQuery::new(columns_sql, vec![], QueryId::generate());
        let columns_result = self.executor.execute_query(&columns_query).await?;

        // Group columns by table_name
        let mut columns_by_table: std::collections::HashMap<String, Vec<ColumnMetadata>> =
            std::collections::HashMap::new();

        for row in columns_result.rows {
            let table_name = row.get::<String>("table_name").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let col_name = row.get::<String>("column_name").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let data_type = row.get::<String>("data_type").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let is_nullable_str = row.get::<String>("is_nullable").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let col_default = row.get_opt::<String>("column_default").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;

            let is_nullable = is_nullable_str.eq_ignore_ascii_case("yes");
            let has_default_value = col_default.is_some();
            let is_auto_incrementing = col_default
                .as_ref()
                .map(|d| d.contains("nextval("))
                .unwrap_or(false);

            columns_by_table
                .entry(table_name)
                .or_default()
                .push(ColumnMetadata {
                    name: col_name,
                    data_type,
                    is_nullable,
                    has_default_value,
                    is_auto_incrementing,
                });
        }

        let mut tables = Vec::with_capacity(tables_result.len());
        for row in tables_result.rows {
            let table_name = row.get::<String>("table_name").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let schema_name = row.get_opt::<String>("table_schema").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let table_type = row.get::<String>("table_type").map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            let is_view = table_type.to_uppercase().contains("VIEW");

            let columns = columns_by_table.remove(&table_name).unwrap_or_default();

            tables.push(TableMetadata {
                name: table_name,
                schema: schema_name,
                is_view,
                columns,
            });
        }

        Ok(tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::kysely::Kysely;
    use crate::operation_node::Value;
    use crate::query_compiler::DefaultQueryCompiler;
    use crate::query_executor::{QueryResult, Row};

    struct MockIntrospectionConnection;

    #[async_trait]
    impl DatabaseConnection for MockIntrospectionConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            if query.sql.contains("information_schema.schemata") {
                Ok(QueryResult::with_rows(vec![
                    Row::new(vec!["schema_name"], vec![Value::from("public")]),
                    Row::new(vec!["schema_name"], vec![Value::from("custom")]),
                ]))
            } else if query.sql.contains("information_schema.tables") {
                Ok(QueryResult::with_rows(vec![
                    Row::new(
                        vec!["table_name", "table_schema", "table_type"],
                        vec![Value::from("users"), Value::from("public"), Value::from("BASE TABLE")],
                    ),
                    Row::new(
                        vec!["table_name", "table_schema", "table_type"],
                        vec![Value::from("user_summary"), Value::from("public"), Value::from("VIEW")],
                    ),
                ]))
            } else if query.sql.contains("information_schema.columns") {
                Ok(QueryResult::with_rows(vec![
                    Row::new(
                        vec!["table_name", "column_name", "data_type", "is_nullable", "column_default"],
                        vec![
                            Value::from("users"),
                            Value::from("id"),
                            Value::from("bigint"),
                            Value::from("NO"),
                            Value::from("nextval('users_id_seq'::regclass)"),
                        ],
                    ),
                    Row::new(
                        vec!["table_name", "column_name", "data_type", "is_nullable", "column_default"],
                        vec![
                            Value::from("users"),
                            Value::from("name"),
                            Value::from("varchar"),
                            Value::from("YES"),
                            Value::Null,
                        ],
                    ),
                ]))
            } else {
                Ok(QueryResult::default())
            }
        }
    }

    struct MockDriver;

    #[async_trait]
    impl Driver for MockDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockIntrospectionConnection))
        }

        async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_postgres_introspection() {
        let driver = Arc::new(MockDriver);
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        let introspector = PostgresIntrospector::new(Arc::clone(db.executor()));

        // 1. Schemas
        let schemas = introspector.get_schemas().await.unwrap();
        assert_eq!(schemas, vec!["public", "custom"]);

        // 2. Tables and columns
        let tables = introspector.get_tables().await.unwrap();
        assert_eq!(tables.len(), 2);

        let users_table = &tables[0];
        assert_eq!(users_table.name, "users");
        assert_eq!(users_table.is_view, false);
        assert_eq!(users_table.columns.len(), 2);

        let id_col = &users_table.columns[0];
        assert_eq!(id_col.name, "id");
        assert_eq!(id_col.data_type, "bigint");
        assert_eq!(id_col.is_nullable, false);
        assert_eq!(id_col.has_default_value, true);
        assert_eq!(id_col.is_auto_incrementing, true);

        let name_col = &users_table.columns[1];
        assert_eq!(name_col.name, "name");
        assert_eq!(name_col.is_nullable, true);
        assert_eq!(name_col.has_default_value, false);
        assert_eq!(name_col.is_auto_incrementing, false);

        let view_table = &tables[1];
        assert_eq!(view_table.name, "user_summary");
        assert_eq!(view_table.is_view, true);
    }
}
