//! # rust-kysely
//!
//! Pure Rust, type-safe SQL query builder and runner ported from Kysely (TypeScript) without macros.

#![warn(missing_docs)]

pub mod dialect;
pub mod driver;
pub mod expression;
pub mod introspection;
pub mod kysely;
pub mod migration;
pub mod operation_node;
pub mod parser;
pub mod plugin;
pub mod query_builder;
pub mod query_compiler;
pub mod query_executor;
pub mod raw_builder;
pub mod schema;
pub mod transaction;
pub mod util;

#[cfg(feature = "mssql")]
pub use dialect::mssql::{
    MssqlAdapter, MssqlDialect, MssqlIntrospector, MssqlQueryCompiler, MSSQL_MIGRATION_LOCK_NAME,
};
#[cfg(feature = "mysql")]
pub use dialect::mysql::{
    MysqlAdapter, MysqlDialect, MysqlIntrospector, MysqlQueryCompiler, MYSQL_MIGRATION_LOCK_NAME,
};
#[cfg(feature = "pglite")]
pub use dialect::pglite::PGliteDialect;
#[cfg(feature = "postgres")]
pub use dialect::postgres::{
    PostgresAdapter, PostgresConnection, PostgresDialect, PostgresDriver, PostgresIntrospector,
    PostgresQueryCompiler, KYSELY_MIGRATION_LOCK_ID,
};
#[cfg(feature = "sqlite")]
pub use dialect::sqlite::{SqliteAdapter, SqliteDialect, SqliteIntrospector, SqliteQueryCompiler};
pub use dialect::{CustomDialect, DefaultAdapter, Dialect, DialectAdapter};
pub use driver::{DatabaseConnection, Driver, DriverError};
pub use expression::{Expression, ExpressionBuilder, ExpressionWrapper, FunctionModule};
pub use introspection::{ColumnMetadata, DatabaseIntrospector, DatabaseMetadata, TableMetadata};
pub use kysely::Kysely;
pub use migration::{
    format_iso8601, FileMigrationProvider, Migration, MigrationInfo, MigrationProvider,
    MigrationResult, MigrationResultSet, MigrationStatus, Migrator, MigratorConfig,
    SimpleMigrationProvider, SqlMigration, NO_MIGRATIONS,
};
pub use operation_node::*;
pub use parser::{IntoSelectionItem, IntoSelections, IntoTableSource};
pub use plugin::{
    to_camel_case, to_snake_case, CamelCaseOptions, CamelCasePlugin, DeduplicateJoinsPlugin,
    KyselyPlugin, WithSchemaPlugin,
};
pub use query_builder::{
    delete_from, insert_into, select_from, update_table, DeleteQueryBuilder, InsertQueryBuilder,
    IntoGroupBy, IntoOrderByExpr, IntoOrderDirection, IntoRowValues, JoinBuilder,
    OnConflictBuilder, RowValues, SelectQueryBuilder, UpdateQueryBuilder,
};
pub use query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};
pub use query_executor::{
    ColumnError, DefaultQueryExecutor, FromRowValue, PluginExecutor, QueryExecutor,
    QueryExecutorExt, QueryResult, Row, SingleConnectionExecutor,
};
pub use raw_builder::{sql, RawBuilder};
pub use schema::{
    alter_table, create_index, create_table, create_type, create_view, drop_index, drop_table,
    drop_type, drop_view, AlterTableBuilder, ColumnDefinitionBuilder, CreateIndexBuilder,
    CreateTableBuilder, CreateTypeBuilder, CreateViewBuilder, DropIndexBuilder, DropTableBuilder,
    DropTypeBuilder, DropViewBuilder, ForeignKeyConstraintBuilder, SchemaModule,
};
pub use transaction::{IsolationLevel, Transaction, TransactionBuilder};
pub use util::QueryId;

/// Namespace containing raw SQL helper functions (`sql::raw`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`).
pub mod sql {
    pub use crate::raw_builder::helpers::{id, join, lit, raw, table};
    pub use crate::raw_builder::sql;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
