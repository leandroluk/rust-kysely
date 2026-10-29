//! MySQL SQL query compiler.

use crate::operation_node::OperationNode;
use crate::query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};

/// MySQL SQL query compiler using backtick identifiers (`` `name` ``), `?` parameter placeholders,
/// and `ON DUPLICATE KEY UPDATE` upsert syntax.
// @spec REQ-DIA-001
#[derive(Debug, Clone, Default)]
pub struct MysqlQueryCompiler {
    inner: DefaultQueryCompiler,
}

impl MysqlQueryCompiler {
    /// Creates a new `MysqlQueryCompiler`.
    pub fn new() -> Self {
        Self {
            inner: DefaultQueryCompiler::with_styles(
                ParameterStyle::QuestionMark,
                IdentifierQuote::Backtick,
            )
            .with_mysql_upsert(),
        }
    }
}

impl QueryCompiler for MysqlQueryCompiler {
    fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.inner.compile(node)
    }
}
