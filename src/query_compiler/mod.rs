//! Query compilation infrastructure translating AST operation nodes into SQL.

pub mod compiled_query;
pub mod default_compiler;

pub use compiled_query::CompiledQuery;
pub use default_compiler::{DefaultQueryCompiler, IdentifierQuote, ParameterStyle};

use crate::operation_node::OperationNode;

/// General trait for compiling an `OperationNode` AST into a `CompiledQuery`.
// @spec REQ-AST-008
pub trait QueryCompiler {
    /// Compiles an AST node into a database-ready query with parameters.
    fn compile(&self, node: &OperationNode) -> CompiledQuery;
}

impl QueryCompiler for DefaultQueryCompiler {
    fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.compile(node)
    }
}
