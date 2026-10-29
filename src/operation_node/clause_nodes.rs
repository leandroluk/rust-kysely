//! Clause AST nodes for FROM, JOIN, WHERE, GROUP BY, ORDER BY, LIMIT, OFFSET, and WITH.

use std::sync::Arc;
use smallvec::SmallVec;

use crate::operation_node::OperationNode;

/// Join type specifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT JOIN
    Left,
    /// RIGHT JOIN
    Right,
    /// FULL OUTER JOIN
    Full,
    /// LATERAL INNER JOIN
    LateralInner,
    /// LATERAL LEFT JOIN
    LateralLeft,
}

impl JoinType {
    /// Returns the SQL string representation of this join type.
    pub fn as_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
            JoinType::LateralInner => "INNER JOIN LATERAL",
            JoinType::LateralLeft => "LEFT JOIN LATERAL",
        }
    }
}

/// FROM clause node holding one or more tables/sources.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct FromNode {
    /// Target tables or subqueries.
    pub froms: SmallVec<[Arc<OperationNode>; 2]>,
}

impl FromNode {
    /// Creates a new FromNode with a single source.
    pub fn new(source: Arc<OperationNode>) -> Self {
        let mut froms = SmallVec::new();
        froms.push(source);
        Self { froms }
    }

    /// Creates a new FromNode from multiple sources.
    // @spec REQ-AST-004
    pub fn from_many(sources: impl IntoIterator<Item = Arc<OperationNode>>) -> Self {
        Self {
            froms: sources.into_iter().collect(),
        }
    }
}

/// JOIN clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct JoinNode {
    /// Type of join.
    pub join_type: JoinType,
    /// Target table or subquery to join.
    pub table: Arc<OperationNode>,
    /// Optional ON condition expression.
    pub on: Option<Arc<OperationNode>>,
}

impl JoinNode {
    /// Creates a new join node without ON condition.
    pub fn new(join_type: JoinType, table: Arc<OperationNode>) -> Self {
        Self {
            join_type,
            table,
            on: None,
        }
    }

    /// Creates a new join node with ON condition.
    pub fn with_on(join_type: JoinType, table: Arc<OperationNode>, on: Arc<OperationNode>) -> Self {
        Self {
            join_type,
            table,
            on: Some(on),
        }
    }
}

/// Selection item node (expression + optional alias).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionNode {
    /// Selected column or expression.
    pub expr: Arc<OperationNode>,
    /// Optional alias.
    pub alias: Option<Arc<OperationNode>>,
}

impl SelectionNode {
    /// Creates a selection without an alias.
    pub fn new(expr: Arc<OperationNode>) -> Self {
        Self { expr, alias: None }
    }

    /// Creates a selection with an alias.
    pub fn with_alias(expr: Arc<OperationNode>, alias: Arc<OperationNode>) -> Self {
        Self {
            expr,
            alias: Some(alias),
        }
    }
}

/// WHERE clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct WhereNode {
    /// Boolean condition expression.
    pub where_expr: Arc<OperationNode>,
}

impl WhereNode {
    /// Creates a new where clause node.
    pub fn new(where_expr: Arc<OperationNode>) -> Self {
        Self { where_expr }
    }
}

/// HAVING clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct HavingNode {
    /// Boolean condition expression.
    pub having_expr: Arc<OperationNode>,
}

impl HavingNode {
    /// Creates a new having clause node.
    pub fn new(having_expr: Arc<OperationNode>) -> Self {
        Self { having_expr }
    }
}

/// GROUP BY clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByNode {
    /// Grouping expressions.
    // @spec REQ-AST-004
    pub items: SmallVec<[Arc<OperationNode>; 4]>,
}

impl GroupByNode {
    /// Creates a new group by node with multiple expressions.
    pub fn new(items: impl IntoIterator<Item = Arc<OperationNode>>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }
}

/// Sort order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderDirection {
    /// ASC
    Asc,
    /// DESC
    Desc,
}

/// NULL ordering placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderNulls {
    /// NULLS FIRST
    First,
    /// NULLS LAST
    Last,
}

/// Individual item in an ORDER BY clause.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItemNode {
    /// Target expression to sort by.
    pub expr: Arc<OperationNode>,
    /// Sort direction.
    pub direction: Option<OrderDirection>,
    /// NULLs positioning.
    pub nulls: Option<OrderNulls>,
}

impl OrderByItemNode {
    /// Creates a simple order by item with direction.
    pub fn new(expr: Arc<OperationNode>, direction: OrderDirection) -> Self {
        Self {
            expr,
            direction: Some(direction),
            nulls: None,
        }
    }

    /// Creates an order by item with direction and nulls positioning.
    pub fn with_nulls(
        expr: Arc<OperationNode>,
        direction: OrderDirection,
        nulls: OrderNulls,
    ) -> Self {
        Self {
            expr,
            direction: Some(direction),
            nulls: Some(nulls),
        }
    }
}

/// ORDER BY clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByNode {
    /// Order by items.
    // @spec REQ-AST-004
    pub items: SmallVec<[Arc<OperationNode>; 4]>,
}

impl OrderByNode {
    /// Creates a new order by node.
    pub fn new(items: impl IntoIterator<Item = Arc<OperationNode>>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }
}

/// LIMIT clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct LimitNode {
    /// Limit count or parameter expression.
    pub limit: Arc<OperationNode>,
}

impl LimitNode {
    /// Creates a new limit node.
    pub fn new(limit: Arc<OperationNode>) -> Self {
        Self { limit }
    }
}

/// OFFSET clause node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct OffsetNode {
    /// Offset count or parameter expression.
    pub offset: Arc<OperationNode>,
}

impl OffsetNode {
    /// Creates a new offset node.
    pub fn new(offset: Arc<OperationNode>) -> Self {
        Self { offset }
    }
}

/// Common Table Expression (CTE) item node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct CommonTableExpressionNode {
    /// Name of CTE.
    pub name: Arc<OperationNode>,
    /// CTE expression / query.
    pub expression: Arc<OperationNode>,
}

impl CommonTableExpressionNode {
    /// Creates a new CTE item node.
    pub fn new(name: Arc<OperationNode>, expression: Arc<OperationNode>) -> Self {
        Self { name, expression }
    }
}

/// WITH clause node (CTEs).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct WithNode {
    /// Whether this is `WITH RECURSIVE`.
    pub recursive: bool,
    /// List of CTE items.
    pub expressions: SmallVec<[Arc<OperationNode>; 2]>,
}

impl WithNode {
    /// Creates a new WITH node.
    pub fn new(
        recursive: bool,
        expressions: impl IntoIterator<Item = Arc<OperationNode>>,
    ) -> Self {
        Self {
            recursive,
            expressions: expressions.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::{IdentifierNode, ValueNode};

    #[test]
    fn test_clause_nodes() {
        let table = Arc::new(OperationNode::Identifier(IdentifierNode::new("users")));
        let from = FromNode::new(Arc::clone(&table));
        assert_eq!(from.froms.len(), 1);

        let join = JoinNode::new(JoinType::Inner, Arc::clone(&table));
        assert_eq!(join.join_type, JoinType::Inner);
        assert_eq!(join.join_type.as_sql(), "INNER JOIN");

        let sel = SelectionNode::new(Arc::clone(&table));
        assert!(sel.alias.is_none());

        let cond = Arc::new(OperationNode::Value(ValueNode::new(true)));
        let whr = WhereNode::new(Arc::clone(&cond));
        assert_eq!(whr.where_expr, cond);

        let lim = LimitNode::new(Arc::new(OperationNode::Value(ValueNode::new(10))));
        assert!(matches!(*lim.limit, OperationNode::Value(_)));

        let off = OffsetNode::new(Arc::new(OperationNode::Value(ValueNode::new(20))));
        assert!(matches!(*off.offset, OperationNode::Value(_)));

        let ob_item = OrderByItemNode::new(Arc::clone(&table), OrderDirection::Desc);
        assert_eq!(ob_item.direction, Some(OrderDirection::Desc));

        let ob = OrderByNode::new([Arc::new(OperationNode::OrderByItem(ob_item))]);
        assert_eq!(ob.items.len(), 1);
    }
}
