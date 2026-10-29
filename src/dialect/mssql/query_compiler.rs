//! Microsoft SQL Server (MSSQL) SQL query compiler.

use crate::operation_node::OperationNode;
use crate::query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};

/// MSSQL query compiler using bracket identifiers (`[table].[col]`) and `@p1, @p2, ...` parameter placeholders.
// @spec REQ-DIA-002
#[derive(Debug, Clone, Default)]
pub struct MssqlQueryCompiler {
    inner: DefaultQueryCompiler,
}

impl MssqlQueryCompiler {
    /// Creates a new `MssqlQueryCompiler`.
    pub fn new() -> Self {
        Self {
            inner: DefaultQueryCompiler::with_styles(ParameterStyle::AtP, IdentifierQuote::Bracket),
        }
    }
}

impl QueryCompiler for MssqlQueryCompiler {
    fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.inner.compile(node)
    }
}
