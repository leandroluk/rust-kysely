//! Representation of a compiled SQL query ready for database execution.

use crate::operation_node::Value;
use crate::util::QueryId;

/// A fully compiled SQL query ready for execution on a database connection.
// @spec REQ-AST-008
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledQuery {
    /// Parameterized SQL string with dialect-specific placeholders.
    pub sql: String,
    /// Parameter values collected in order of appearance in the SQL string.
    pub parameters: Vec<Value>,
    /// Unique identifier for tracing this query execution.
    pub query_id: QueryId,
}

impl CompiledQuery {
    /// Creates a new compiled query instance.
    pub fn new(sql: impl Into<String>, parameters: Vec<Value>, query_id: QueryId) -> Self {
        Self {
            sql: sql.into(),
            parameters,
            query_id,
        }
    }
}
