//! SQL dialect-specific query compilers and driver adapters.

use std::sync::Arc;
use crate::driver::Driver;
use crate::query_compiler::QueryCompiler;

pub mod adapter;
pub use adapter::{DefaultAdapter, DialectAdapter};

/// Database dialect abstraction creating drivers and query compilers.
// @spec REQ-EXEC-003, REQ-MIG-005
pub trait Dialect: Send + Sync {
    /// Creates a new driver instance for this dialect.
    fn create_driver(&self) -> Arc<dyn Driver>;

    /// Creates a new query compiler instance for this dialect.
    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler>;

    /// Creates a dialect adapter for migration locking and dialect features.
    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(DefaultAdapter)
    }
}

/// Dialect adapter using arbitrary driver and query compiler implementations.
pub struct CustomDialect {
    driver: Arc<dyn Driver>,
    compiler: Arc<dyn QueryCompiler>,
    adapter: Option<Arc<dyn DialectAdapter>>,
}

impl CustomDialect {
    /// Creates a new `CustomDialect`.
    pub fn new(driver: Arc<dyn Driver>, compiler: Arc<dyn QueryCompiler>) -> Self {
        Self {
            driver,
            compiler,
            adapter: None,
        }
    }

    /// Sets the dialect adapter for this `CustomDialect`.
    pub fn with_adapter(mut self, adapter: Arc<dyn DialectAdapter>) -> Self {
        self.adapter = Some(adapter);
        self
    }
}

impl Dialect for CustomDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::clone(&self.compiler)
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        self.adapter
            .clone()
            .unwrap_or_else(|| Arc::new(DefaultAdapter))
    }
}

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub use postgres::PostgresQueryCompiler;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteAdapter, SqliteDialect, SqliteIntrospector, SqliteQueryCompiler};

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "mysql")]
pub use mysql::{MysqlAdapter, MysqlDialect, MysqlIntrospector, MysqlQueryCompiler};
