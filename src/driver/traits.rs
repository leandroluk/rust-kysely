//! Async traits defining database communication and connection life-cycle.

use async_trait::async_trait;

use crate::driver::error::DriverError;
use crate::query_compiler::CompiledQuery;
use crate::query_executor::QueryResult;

/// Represents an active connection to the database.
// @spec REQ-EXEC-001
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Executes a compiled query against the connection and returns the result.
    async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError>;
}

/// Represents a database driver managing connection acquisition and life-cycle.
// @spec REQ-EXEC-001
#[async_trait]
pub trait Driver: Send + Sync {
    /// Acquires a new database connection from the driver or pool.
    async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError>;

    /// Releases an acquired connection back to the pool or closes it.
    async fn release_connection(
        &self,
        connection: Box<dyn DatabaseConnection>,
    ) -> Result<(), DriverError>;

    /// Destroys and cleans up the driver and its connection pool.
    async fn destroy(&self) -> Result<(), DriverError>;
}
