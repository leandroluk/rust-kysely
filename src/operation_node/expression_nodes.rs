//! Expression AST nodes: Binary, Unary, Case, When, Cast, Alias, Parens, And, Or.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::operation_node::OperationNode;

/// Binary operation node (`left OP right`, e.g. `age > 18`, `first_name = 'Bob'`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperationNode {
    /// Left operand.
    pub left: Arc<OperationNode>,
    /// Binary operator string (e.g. `=`, `!=`, `>`, `<`, `LIKE`, `IN`).
    pub operator: CompactString,
    /// Right operand.
    pub right: Arc<OperationNode>,
}

impl BinaryOperationNode {
    /// Creates a new binary operation node.
    pub fn new(
        left: Arc<OperationNode>,
        operator: impl Into<CompactString>,
        right: Arc<OperationNode>,
    ) -> Self {
        Self {
            left,
            operator: operator.into(),
            right,
        }
    }
}

/// Unary operation node (`OP operand`, e.g. `NOT is_active`, `-amount`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperationNode {
    /// Unary operator string (e.g. `NOT`, `EXISTS`, `-`).
    pub operator: CompactString,
    /// Target operand.
    pub operand: Arc<OperationNode>,
}

impl UnaryOperationNode {
    /// Creates a new unary operation node.
    pub fn new(operator: impl Into<CompactString>, operand: Arc<OperationNode>) -> Self {
        Self {
            operator: operator.into(),
            operand,
        }
    }
}

/// WHEN branch inside a CASE expression (`WHEN condition THEN result`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct WhenNode {
    /// WHEN condition.
    pub condition: Arc<OperationNode>,
    /// THEN result expression.
    pub result: Arc<OperationNode>,
}

impl WhenNode {
    /// Creates a new when node.
    pub fn new(condition: Arc<OperationNode>, result: Arc<OperationNode>) -> Self {
        Self { condition, result }
    }
}

/// CASE expression node (`CASE [value] WHEN ... THEN ... ELSE ... END`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct CaseNode {
    /// Optional initial expression being matched (simple CASE).
    pub value: Option<Arc<OperationNode>>,
    /// List of WHEN branches.
    // @spec REQ-AST-004
    pub when_clauses: SmallVec<[Arc<OperationNode>; 4]>,
    /// Optional ELSE result.
    pub else_clause: Option<Arc<OperationNode>>,
}

impl CaseNode {
    /// Creates a new searched CASE node (`CASE WHEN ... THEN ... END`).
    pub fn new(
        when_clauses: impl IntoIterator<Item = Arc<OperationNode>>,
        else_clause: Option<Arc<OperationNode>>,
    ) -> Self {
        Self {
            value: None,
            when_clauses: when_clauses.into_iter().collect(),
            else_clause,
        }
    }

    /// Creates a new simple CASE node (`CASE value WHEN ... THEN ... END`).
    pub fn with_value(
        value: Arc<OperationNode>,
        when_clauses: impl IntoIterator<Item = Arc<OperationNode>>,
        else_clause: Option<Arc<OperationNode>>,
    ) -> Self {
        Self {
            value: Some(value),
            when_clauses: when_clauses.into_iter().collect(),
            else_clause,
        }
    }
}

/// CAST expression node (`CAST(expr AS data_type)`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct CastNode {
    /// Expression being cast.
    pub expr: Arc<OperationNode>,
    /// Target SQL data type name.
    pub data_type: CompactString,
}

impl CastNode {
    /// Creates a new cast expression node.
    pub fn new(expr: Arc<OperationNode>, data_type: impl Into<CompactString>) -> Self {
        Self {
            expr,
            data_type: data_type.into(),
        }
    }
}

/// Alias node (`node AS alias`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct AliasNode {
    /// Target expression or table.
    pub node: Arc<OperationNode>,
    /// Alias identifier.
    pub alias: Arc<OperationNode>,
}

impl AliasNode {
    /// Creates a new alias node.
    pub fn new(node: Arc<OperationNode>, alias: Arc<OperationNode>) -> Self {
        Self { node, alias }
    }
}

/// Parenthesized expression node (`(node)`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ParensNode {
    /// Enclosed inner node.
    pub node: Arc<OperationNode>,
}

impl ParensNode {
    /// Creates a parenthesized node.
    pub fn new(node: Arc<OperationNode>) -> Self {
        Self { node }
    }
}

/// SQL function call node (`func_name(arg1, arg2, ...)`).
// @spec REQ-EXPR-010
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNode {
    /// Function name (e.g. `COUNT`, `MAX`, `COALESCE`).
    pub name: CompactString,
    /// Arguments passed to the function.
    pub arguments: SmallVec<[Arc<OperationNode>; 2]>,
}

impl FunctionNode {
    /// Creates a new function call node.
    pub fn new(
        name: impl Into<CompactString>,
        arguments: impl IntoIterator<Item = Arc<OperationNode>>,
    ) -> Self {
        Self {
            name: name.into(),
            arguments: arguments.into_iter().collect(),
        }
    }
}

/// AND boolean operation node (`left AND right`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct AndNode {
    /// Left boolean operand.
    pub left: Arc<OperationNode>,
    /// Right boolean operand.
    pub right: Arc<OperationNode>,
}

impl AndNode {
    /// Creates a new AND node.
    pub fn new(left: Arc<OperationNode>, right: Arc<OperationNode>) -> Self {
        Self { left, right }
    }
}

/// OR boolean operation node (`left OR right`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct OrNode {
    /// Left boolean operand.
    pub left: Arc<OperationNode>,
    /// Right boolean operand.
    pub right: Arc<OperationNode>,
}

impl OrNode {
    /// Creates a new OR node.
    pub fn new(left: Arc<OperationNode>, right: Arc<OperationNode>) -> Self {
        Self { left, right }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::{IdentifierNode, ValueNode};

    #[test]
    fn test_expression_nodes() {
        let col = Arc::new(OperationNode::Identifier(IdentifierNode::new("age")));
        let val = Arc::new(OperationNode::Value(ValueNode::new(18)));

        let bin = BinaryOperationNode::new(Arc::clone(&col), ">", Arc::clone(&val));
        assert_eq!(bin.operator.as_str(), ">");

        let un = UnaryOperationNode::new("NOT", Arc::clone(&col));
        assert_eq!(un.operator.as_str(), "NOT");

        let when = WhenNode::new(Arc::clone(&col), Arc::clone(&val));
        let case = CaseNode::new([Arc::new(OperationNode::When(when))], None);
        assert_eq!(case.when_clauses.len(), 1);

        let cast = CastNode::new(Arc::clone(&col), "integer");
        assert_eq!(cast.data_type.as_str(), "integer");

        let alias_id = Arc::new(OperationNode::Identifier(IdentifierNode::new("user_age")));
        let alias = AliasNode::new(Arc::clone(&col), alias_id);
        assert!(matches!(*alias.alias, OperationNode::Identifier(_)));

        let parens = ParensNode::new(Arc::clone(&col));
        assert!(matches!(*parens.node, OperationNode::Identifier(_)));

        let and_node = AndNode::new(Arc::clone(&col), Arc::clone(&val));
        assert_eq!(and_node.left, col);

        let or_node = OrNode::new(Arc::clone(&col), Arc::clone(&val));
        assert_eq!(or_node.right, val);
    }
}
