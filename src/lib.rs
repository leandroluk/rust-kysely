//! # rust-kysely
//!
//! Pure Rust, type-safe SQL query builder and runner ported from Kysely (TypeScript) without macros.

#![warn(missing_docs)]

pub mod dialect;
pub mod driver;
pub mod expression;
pub mod operation_node;
pub mod parser;
pub mod query_builder;
pub mod query_compiler;
pub mod query_executor;
pub mod raw_builder;
pub mod util;

pub use driver::{DatabaseConnection, Driver, DriverError};
#[cfg(feature = "postgres")]
pub use dialect::PostgresQueryCompiler;
pub use expression::{Expression, ExpressionBuilder, ExpressionWrapper, FunctionModule};
pub use operation_node::*;
pub use parser::{IntoSelectionItem, IntoSelections, IntoTableSource};
pub use query_builder::{
    delete_from, insert_into, select_from, update_table, DeleteQueryBuilder, InsertQueryBuilder,
    IntoGroupBy, IntoOrderByExpr, IntoOrderDirection, IntoRowValues, JoinBuilder, OnConflictBuilder,
    RowValues, SelectQueryBuilder, UpdateQueryBuilder,
};
pub use query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};
pub use query_executor::{ColumnError, QueryResult, Row};
pub use raw_builder::{sql, RawBuilder};
pub use util::QueryId;

/// Namespace containing raw SQL helper functions (`sql::raw`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`).
pub mod sql {
    pub use crate::raw_builder::helpers::{id, join, lit, raw, table};
    pub use crate::raw_builder::sql;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
