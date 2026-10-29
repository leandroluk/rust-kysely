//! Concrete expression wrapper around an AST OperationNode.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::expression::Expression;
use crate::operation_node::OperationNode;

/// Generic wrapper around an `Arc<OperationNode>` with phantom output type `T`.
// @spec REQ-EXPR-001
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionWrapper<T = ()> {
    node: Arc<OperationNode>,
    _phantom: PhantomData<T>,
}

impl<T> ExpressionWrapper<T> {
    /// Creates a new expression wrapper from an AST node.
    pub fn new(node: Arc<OperationNode>) -> Self {
        Self {
            node,
            _phantom: PhantomData,
        }
    }

    /// Returns a reference to the inner AST node.
    pub fn node(&self) -> &Arc<OperationNode> {
        &self.node
    }
}

impl<T> Expression<T> for ExpressionWrapper<T> {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::clone(&self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::ColumnNode;

    #[test]
    fn test_expression_wrapper() {
        let col = Arc::new(OperationNode::Column(ColumnNode::new("email")));
        let expr: ExpressionWrapper<String> = ExpressionWrapper::new(col);

        assert!(matches!(**expr.node(), OperationNode::Column(_)));

        let aliased = expr.as_("user_email");
        if let OperationNode::Alias(ref a) = *aliased.to_operation_node() {
            if let OperationNode::Identifier(ref id) = *a.alias {
                assert_eq!(id.name.as_str(), "user_email");
            } else {
                panic!("Expected Identifier alias");
            }
        } else {
            panic!("Expected Alias node");
        }
    }
}
