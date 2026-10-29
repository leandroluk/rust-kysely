//! Driver and connection error definitions.

use thiserror::Error;

/// Errors arising from database drivers and connections.
// @spec REQ-EXEC-001
#[derive(Debug, Error)]
pub enum DriverError {
    /// Connection acquisition failed
    #[error("Failed to acquire database connection: {0}")]
    ConnectionFailed(String),

    /// Query execution failed
    #[error("Query execution error: {0}")]
    ExecutionFailed(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Driver was destroyed or shut down
    #[error("Database driver has been shut down")]
    DriverDestroyed,

    /// Other generic or underlying driver errors
    #[error("Driver error: {0}")]
    Other(String),
}

impl From<crate::query_executor::ColumnError> for DriverError {
    fn from(err: crate::query_executor::ColumnError) -> Self {
        DriverError::ExecutionFailed(err.to_string())
    }
}
