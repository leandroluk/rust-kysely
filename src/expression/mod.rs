//! Expression subsystem and traits for type-safe query construction.

pub mod expression_builder;
pub mod function_module;
pub mod wrapper;

use std::sync::Arc;
use compact_str::CompactString;

use crate::operation_node::{AliasNode, IdentifierNode, OperationNode};
pub use expression_builder::{parse_column_reference, ExpressionBuilder};
pub use function_module::FunctionModule;
pub use wrapper::ExpressionWrapper;

impl Expression<()> for crate::operation_node::SelectQueryNode {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::SelectQuery(self.clone()))
    }
}

/// Core trait representing any SQL expression producing values of type `T`.
// @spec REQ-EXPR-001
pub trait Expression<T = ()> {
    /// Returns the underlying AST `OperationNode`.
    fn to_operation_node(&self) -> Arc<OperationNode>;

    /// Wraps this expression with an SQL alias (`expr AS alias`).
    fn as_(self, alias: impl Into<CompactString>) -> ExpressionWrapper<T>
    where
        Self: Sized,
    {
        let node = self.to_operation_node();
        let alias_node = Arc::new(OperationNode::Identifier(IdentifierNode::new(alias)));
        ExpressionWrapper::new(Arc::new(OperationNode::Alias(AliasNode::new(node, alias_node))))
    }
}
