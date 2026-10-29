//! Database introspection traits and structured metadata representations.

use async_trait::async_trait;
use crate::driver::DriverError;

/// Metadata for a single table column.
// @spec REQ-DDL-010
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMetadata {
    /// Column name
    pub name: String,
    /// Data type string as reported by database
    pub data_type: String,
    /// Is the column nullable
    pub is_nullable: bool,
    /// Does the column have a default value defined
    pub has_default_value: bool,
    /// Is the column auto-incrementing (e.g. SERIAL, BIGSERIAL, IDENTITY)
    pub is_auto_incrementing: bool,
}

/// Metadata for a database table or view.
// @spec REQ-DDL-010
#[derive(Debug, Clone, PartialEq)]
pub struct TableMetadata {
    /// Table or view name
    pub name: String,
    /// Schema name (e.g. "public")
    pub schema: Option<String>,
    /// Whether this object is a VIEW or MATERIALIZED VIEW
    pub is_view: bool,
    /// Ordered list of columns
    pub columns: Vec<ColumnMetadata>,
}

/// Metadata container for an entire database schema.
// @spec REQ-DDL-010
#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseMetadata {
    /// List of tables and views
    pub tables: Vec<TableMetadata>,
}

/// Async trait providing introspection into database schemas, tables, and columns.
// @spec REQ-DDL-010
#[async_trait]
pub trait DatabaseIntrospector: Send + Sync {
    /// Returns schemas present in the database.
    async fn get_schemas(&self) -> Result<Vec<String>, DriverError>;

    /// Returns all tables and views with their column definitions.
    async fn get_tables(&self) -> Result<Vec<TableMetadata>, DriverError>;

    /// Returns full database metadata.
    async fn get_metadata(&self) -> Result<DatabaseMetadata, DriverError> {
        let tables = self.get_tables().await?;
        Ok(DatabaseMetadata { tables })
    }
}
