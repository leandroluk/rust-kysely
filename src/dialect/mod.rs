//! SQL dialect-specific query compilers and driver adapters.

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use postgres::PostgresQueryCompiler;
