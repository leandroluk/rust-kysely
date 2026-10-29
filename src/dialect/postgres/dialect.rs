//! PostgreSQL Dialect definition creating drivers and query compilers.

use std::sync::Arc;

use crate::dialect::adapter::DialectAdapter;
use crate::dialect::postgres::adapter::PostgresAdapter;
use crate::dialect::postgres::driver::PostgresDriver;
use crate::dialect::postgres::query_compiler::PostgresQueryCompiler;
use crate::dialect::Dialect;
use crate::driver::{Driver, DriverError};
use crate::query_compiler::QueryCompiler;

/// Dialect adapter for PostgreSQL databases.
// @spec REQ-EXEC-002, REQ-EXEC-003
pub struct PostgresDialect {
    driver: Arc<dyn Driver>,
}

impl PostgresDialect {
    /// Creates a `PostgresDialect` by connecting to the specified connection string.
    pub fn new(connection_string: &str) -> Result<Self, DriverError> {
        let driver = Arc::new(PostgresDriver::new(connection_string)?);
        Ok(Self { driver })
    }

    /// Creates a `PostgresDialect` wrapping an existing driver instance.
    pub fn with_driver(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }
}

impl Dialect for PostgresDialect {
    fn create_driver(&self) -> Arc<dyn Driver> {
        Arc::clone(&self.driver)
    }

    fn create_query_compiler(&self) -> Arc<dyn QueryCompiler> {
        Arc::new(PostgresQueryCompiler::new())
    }

    fn create_adapter(&self) -> Arc<dyn DialectAdapter> {
        Arc::new(PostgresAdapter)
    }
}
