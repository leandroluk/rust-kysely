//! Microsoft SQL Server (MSSQL) dialect components.

pub mod adapter;
pub mod dialect;
pub mod introspector;
pub mod query_compiler;

pub use adapter::{MssqlAdapter, MSSQL_MIGRATION_LOCK_NAME};
pub use dialect::MssqlDialect;
pub use introspector::MssqlIntrospector;
pub use query_compiler::MssqlQueryCompiler;
