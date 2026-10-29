//! MySQL database introspector querying information_schema.

use async_trait::async_trait;

use crate::driver::DriverError;
use crate::introspection::{ColumnMetadata, DatabaseIntrospector, TableMetadata};
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;

/// Database introspector for MySQL schemas using `information_schema.tables` and `columns`.
// @spec REQ-DIA-001, REQ-DIA-007
pub struct MysqlIntrospector<'a> {
    executor: &'a dyn QueryExecutor,
}

impl<'a> MysqlIntrospector<'a> {
    /// Creates a new `MysqlIntrospector` bound to an executor.
    pub fn new(executor: &'a dyn QueryExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<'a> DatabaseIntrospector for MysqlIntrospector<'a> {
    async fn get_schemas(&self) -> Result<Vec<String>, DriverError> {
        let sql = "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys') ORDER BY schema_name ASC";
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        let res = self.executor.execute_query(&query).await?;

        let mut schemas = Vec::with_capacity(res.rows.len());
        for row in &res.rows {
            schemas.push(row.get("schema_name")?);
        }
        Ok(schemas)
    }

    async fn get_tables(&self) -> Result<Vec<TableMetadata>, DriverError> {
        let tables_sql = "SELECT table_name, table_schema, table_type FROM information_schema.tables WHERE table_schema = DATABASE() AND table_type IN ('BASE TABLE', 'VIEW') ORDER BY table_name ASC";
        let tables_query = CompiledQuery::new(tables_sql, vec![], QueryId::generate());
        let tables_res = self.executor.execute_query(&tables_query).await?;

        let cols_sql = "SELECT table_name, column_name, data_type, is_nullable, column_default, extra FROM information_schema.columns WHERE table_schema = DATABASE() ORDER BY table_name ASC, ordinal_position ASC";
        let cols_query = CompiledQuery::new(cols_sql, vec![], QueryId::generate());
        let cols_res = self.executor.execute_query(&cols_query).await?;

        let mut tables = Vec::with_capacity(tables_res.rows.len());

        for row in &tables_res.rows {
            let table_name: String = row.get("table_name")?;
            let schema_name: Option<String> = row.get_opt("table_schema")?;
            let table_type: String = row.get("table_type")?;
            let is_view = table_type.eq_ignore_ascii_case("view");

            let mut columns = Vec::new();
            for col_row in &cols_res.rows {
                let col_table: String = col_row.get("table_name")?;
                if col_table == table_name {
                    let col_name: String = col_row.get("column_name")?;
                    let data_type: String = col_row.get("data_type")?;
                    let is_null_str: String = col_row.get("is_nullable")?;
                    let col_default: Option<String> = col_row.get_opt("column_default")?;
                    let extra: String = col_row.get_opt("extra")?.unwrap_or_default();

                    let is_nullable = is_null_str.eq_ignore_ascii_case("yes");
                    let has_default_value = col_default.is_some();
                    let is_auto_incrementing = extra.to_lowercase().contains("auto_increment");

                    columns.push(ColumnMetadata {
                        name: col_name,
                        data_type,
                        is_nullable,
                        has_default_value,
                        is_auto_incrementing,
                    });
                }
            }

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
