//! Microsoft SQL Server (MSSQL) database introspector querying INFORMATION_SCHEMA.

use async_trait::async_trait;

use crate::driver::DriverError;
use crate::introspection::{ColumnMetadata, DatabaseIntrospector, TableMetadata};
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryExecutor;
use crate::util::QueryId;

/// Database introspector for MSSQL schemas using `INFORMATION_SCHEMA.TABLES` and `COLUMNS`.
// @spec REQ-DIA-002, REQ-DIA-007
pub struct MssqlIntrospector<'a> {
    executor: &'a dyn QueryExecutor,
}

impl<'a> MssqlIntrospector<'a> {
    /// Creates a new `MssqlIntrospector` bound to an executor.
    pub fn new(executor: &'a dyn QueryExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<'a> DatabaseIntrospector for MssqlIntrospector<'a> {
    async fn get_schemas(&self) -> Result<Vec<String>, DriverError> {
        let sql = "SELECT schema_name FROM INFORMATION_SCHEMA.SCHEMATA WHERE schema_name NOT IN ('guest', 'INFORMATION_SCHEMA', 'sys', 'db_owner', 'db_accessadmin', 'db_securityadmin', 'db_ddladmin', 'db_backupoperator', 'db_datareader', 'db_datawriter', 'db_denydatareader', 'db_denydatawriter') ORDER BY schema_name ASC";
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        let res = self.executor.execute_query(&query).await?;

        let mut schemas = Vec::with_capacity(res.rows.len());
        for row in &res.rows {
            schemas.push(row.get("schema_name")?);
        }
        Ok(schemas)
    }

    async fn get_tables(&self) -> Result<Vec<TableMetadata>, DriverError> {
        let tables_sql = "SELECT TABLE_NAME as table_name, TABLE_SCHEMA as table_schema, TABLE_TYPE as table_type FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE IN ('BASE TABLE', 'VIEW') ORDER BY TABLE_NAME ASC";
        let tables_query = CompiledQuery::new(tables_sql, vec![], QueryId::generate());
        let tables_res = self.executor.execute_query(&tables_query).await?;

        let cols_sql = "SELECT TABLE_NAME as table_name, COLUMN_NAME as column_name, DATA_TYPE as data_type, IS_NULLABLE as is_nullable, COLUMN_DEFAULT as column_default FROM INFORMATION_SCHEMA.COLUMNS ORDER BY TABLE_NAME ASC, ORDINAL_POSITION ASC";
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

                    let is_nullable = is_null_str.eq_ignore_ascii_case("yes");
                    let has_default_value = col_default.is_some();

                    columns.push(ColumnMetadata {
                        name: col_name,
                        data_type,
                        is_nullable,
                        has_default_value,
                        is_auto_incrementing: false,
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
