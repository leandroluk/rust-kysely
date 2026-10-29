//! # rust-kysely
//!
//! Pure Rust, type-safe SQL query builder and runner ported from Kysely (TypeScript) without macros.

#![warn(missing_docs)]

pub mod dialect;
pub mod expression;
pub mod operation_node;
pub mod query_compiler;
pub mod util;

#[cfg(feature = "postgres")]
pub use dialect::PostgresQueryCompiler;
pub use expression::{Expression, ExpressionWrapper};
pub use operation_node::*;
pub use query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};
pub use util::QueryId;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
