//! SQLite SQL query compiler.

use crate::operation_node::OperationNode;
use crate::query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};

/// SQLite SQL query compiler using double-quote identifiers and `?` parameter placeholders.
// @spec REQ-DIA-003
#[derive(Debug, Clone, Default)]
pub struct SqliteQueryCompiler {
    inner: DefaultQueryCompiler,
}

impl SqliteQueryCompiler {
    /// Creates a new `SqliteQueryCompiler`.
    pub fn new() -> Self {
        Self {
            inner: DefaultQueryCompiler::with_styles(
                ParameterStyle::QuestionMark,
                IdentifierQuote::DoubleQuote,
            ),
        }
    }
}

impl QueryCompiler for SqliteQueryCompiler {
    fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.inner.compile(node)
    }
}
