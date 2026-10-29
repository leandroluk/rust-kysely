//! SQLite database introspector querying PRAGMA and sqlite_master.

use async_trait::async_trait;

use crate::driver::DriverError;
use crate::introspection::{ColumnMetadata, DatabaseIntrospector, TableMetadata};
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;

/// Database introspector for SQLite schemas using `PRAGMA table_info` and `sqlite_master`.
// @spec REQ-DIA-003, REQ-DIA-007
pub struct SqliteIntrospector<'a> {
    executor: &'a dyn QueryExecutor,
}

impl<'a> SqliteIntrospector<'a> {
    /// Creates a new `SqliteIntrospector` bound to an executor.
    pub fn new(executor: &'a dyn QueryExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<'a> DatabaseIntrospector for SqliteIntrospector<'a> {
    async fn get_schemas(&self) -> Result<Vec<String>, DriverError> {
        Ok(vec!["main".to_string()])
    }

    async fn get_tables(&self) -> Result<Vec<TableMetadata>, DriverError> {
        let sql = "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' ORDER BY name ASC";
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        let res = self.executor.execute_query(&query).await?;

        let mut tables = Vec::with_capacity(res.rows.len());

        for row in &res.rows {
            let table_name: String = row.get("name")?;
            let obj_type: String = row.get("type")?;
            let is_view = obj_type.eq_ignore_ascii_case("view");

            let pragma_sql = format!("PRAGMA table_info(\"{}\")", table_name.replace('"', "\"\""));
            let pragma_query = CompiledQuery::new(pragma_sql, vec![], QueryId::generate());
            let pragma_res = self.executor.execute_query(&pragma_query).await?;

            let mut columns = Vec::with_capacity(pragma_res.rows.len());

            for col_row in &pragma_res.rows {
                let col_name: String = col_row.get("name")?;
                let col_type: String = col_row.get("type")?;
                let notnull: i64 = col_row.get("notnull").unwrap_or(0);
                let dflt_value: Option<String> = col_row.get_opt("dflt_value").unwrap_or(None);
                let pk: i64 = col_row.get("pk").unwrap_or(0);

                let is_nullable = notnull == 0;
                let has_default_value = dflt_value.is_some();
                let is_auto_incrementing = pk == 1 && col_type.eq_ignore_ascii_case("integer");

                columns.push(ColumnMetadata {
                    name: col_name,
                    data_type: col_type,
                    is_nullable,
                    has_default_value,
                    is_auto_incrementing,
                });
            }

            tables.push(TableMetadata {
                name: table_name,
                schema: Some("main".to_string()),
                is_view,
                columns,
            });
        }

        Ok(tables)
    }
}
