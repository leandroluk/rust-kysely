//! Query-level AST nodes: SelectQueryNode, InsertQueryNode, UpdateQueryNode, DeleteQueryNode.

use smallvec::SmallVec;
use std::sync::Arc;

use crate::operation_node::OperationNode;

/// Column update assignment node (`col = value`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnUpdateNode {
    /// Target column.
    pub column: Arc<OperationNode>,
    /// Assigned value or expression.
    pub value: Arc<OperationNode>,
}

impl ColumnUpdateNode {
    /// Creates a new column update assignment node.
    pub fn new(column: Arc<OperationNode>, value: Arc<OperationNode>) -> Self {
        Self { column, value }
    }
}

/// VALUES row list node for INSERT statements.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ValuesNode {
    /// List of row expressions (each usually a PrimitiveValueListNode or ParensNode).
    // @spec REQ-AST-004
    pub rows: SmallVec<[Arc<OperationNode>; 4]>,
}

impl ValuesNode {
    /// Creates a new values node with one or more rows.
    pub fn new(rows: impl IntoIterator<Item = Arc<OperationNode>>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
        }
    }
}

/// ON CONFLICT clause node (PostgreSQL / SQLite).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct OnConflictNode {
    /// Target conflict columns or constraint name.
    pub columns: Option<SmallVec<[Arc<OperationNode>; 4]>>,
    /// DO NOTHING modifier.
    pub do_nothing: bool,
    /// DO UPDATE SET assignments.
    pub updates: Option<SmallVec<[Arc<OperationNode>; 4]>>,
    /// Optional WHERE condition on conflict.
    pub where_clause: Option<Arc<OperationNode>>,
}

impl OnConflictNode {
    /// Creates an ON CONFLICT DO NOTHING clause.
    pub fn do_nothing(columns: Option<SmallVec<[Arc<OperationNode>; 4]>>) -> Self {
        Self {
            columns,
            do_nothing: true,
            updates: None,
            where_clause: None,
        }
    }

    /// Creates an ON CONFLICT DO UPDATE SET clause.
    pub fn do_update(
        columns: Option<SmallVec<[Arc<OperationNode>; 4]>>,
        updates: SmallVec<[Arc<OperationNode>; 4]>,
        where_clause: Option<Arc<OperationNode>>,
    ) -> Self {
        Self {
            columns,
            do_nothing: false,
            updates: Some(updates),
            where_clause,
        }
    }
}

/// Top-level SELECT query AST node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectQueryNode {
    /// Optional WITH (CTE) clause.
    pub with: Option<Arc<OperationNode>>,
    /// Selected columns and expressions.
    // @spec REQ-AST-004
    pub selections: SmallVec<[Arc<OperationNode>; 4]>,
    /// DISTINCT ON expressions.
    pub distinct_on: Option<SmallVec<[Arc<OperationNode>; 2]>>,
    /// FROM clause.
    pub from: Option<Arc<OperationNode>>,
    /// JOIN clauses.
    // @spec REQ-AST-004
    pub joins: SmallVec<[Arc<OperationNode>; 2]>,
    /// WHERE condition.
    pub where_clause: Option<Arc<OperationNode>>,
    /// GROUP BY clause.
    pub group_by: Option<Arc<OperationNode>>,
    /// HAVING clause.
    pub having: Option<Arc<OperationNode>>,
    /// ORDER BY clause.
    pub order_by: Option<Arc<OperationNode>>,
    /// LIMIT clause.
    pub limit: Option<Arc<OperationNode>>,
    /// OFFSET clause.
    pub offset: Option<Arc<OperationNode>>,
    /// Set operations (UNION, INTERSECT, EXCEPT).
    // @spec REQ-AST-004
    pub set_operations: SmallVec<[Arc<OperationNode>; 1]>,
}

impl SelectQueryNode {
    /// Creates a new empty SELECT query node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Obtains a mutable reference via Copy-on-Write (`Arc::make_mut`).
    // @spec REQ-AST-002
    pub fn clone_mut(arc: &mut Arc<Self>) -> &mut Self {
        Arc::make_mut(arc)
    }
}

/// Top-level INSERT query AST node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InsertQueryNode {
    /// Optional WITH (CTE) clause.
    pub with: Option<Arc<OperationNode>>,
    /// Target table (TableNode).
    pub into: Option<Arc<OperationNode>>,
    /// Optional explicit column list.
    // @spec REQ-AST-004
    pub columns: Option<SmallVec<[Arc<OperationNode>; 4]>>,
    /// Insert values (ValuesNode or SelectQueryNode).
    pub values: Option<Arc<OperationNode>>,
    /// Optional ON CONFLICT clause.
    pub on_conflict: Option<Arc<OperationNode>>,
    /// Optional RETURNING clause items.
    // @spec REQ-AST-004
    pub returning: Option<SmallVec<[Arc<OperationNode>; 4]>>,
}

impl InsertQueryNode {
    /// Creates a new empty INSERT query node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Obtains a mutable reference via Copy-on-Write (`Arc::make_mut`).
    // @spec REQ-AST-002
    pub fn clone_mut(arc: &mut Arc<Self>) -> &mut Self {
        Arc::make_mut(arc)
    }
}

/// Top-level UPDATE query AST node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UpdateQueryNode {
    /// Optional WITH (CTE) clause.
    pub with: Option<Arc<OperationNode>>,
    /// Target table (TableNode).
    pub table: Option<Arc<OperationNode>>,
    /// Optional FROM clause (PostgreSQL UPDATE ... FROM).
    pub from: Option<Arc<OperationNode>>,
    /// Optional JOIN clauses.
    // @spec REQ-AST-004
    pub joins: SmallVec<[Arc<OperationNode>; 2]>,
    /// Update assignments (`col = value`).
    // @spec REQ-AST-004
    pub updates: SmallVec<[Arc<OperationNode>; 4]>,
    /// WHERE condition.
    pub where_clause: Option<Arc<OperationNode>>,
    /// Optional RETURNING clause items.
    // @spec REQ-AST-004
    pub returning: Option<SmallVec<[Arc<OperationNode>; 4]>>,
}

impl UpdateQueryNode {
    /// Creates a new empty UPDATE query node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Obtains a mutable reference via Copy-on-Write (`Arc::make_mut`).
    // @spec REQ-AST-002
    pub fn clone_mut(arc: &mut Arc<Self>) -> &mut Self {
        Arc::make_mut(arc)
    }
}

/// Top-level DELETE query AST node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeleteQueryNode {
    /// Optional WITH (CTE) clause.
    pub with: Option<Arc<OperationNode>>,
    /// Target table (FromNode).
    pub from: Option<Arc<OperationNode>>,
    /// Optional JOIN clauses (PostgreSQL / MySQL DELETE USING / JOIN).
    // @spec REQ-AST-004
    pub joins: SmallVec<[Arc<OperationNode>; 2]>,
    /// WHERE condition.
    pub where_clause: Option<Arc<OperationNode>>,
    /// Optional RETURNING clause items.
    // @spec REQ-AST-004
    pub returning: Option<SmallVec<[Arc<OperationNode>; 4]>>,
}

impl DeleteQueryNode {
    /// Creates a new empty DELETE query node.
    pub fn new() -> Self {
        Self::default()
    }

    /// Obtains a mutable reference via Copy-on-Write (`Arc::make_mut`).
    // @spec REQ-AST-002
    pub fn clone_mut(arc: &mut Arc<Self>) -> &mut Self {
        Arc::make_mut(arc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::{FromNode, IdentifierNode, SelectionNode};

    #[test]
    fn test_query_nodes_cow() {
        // REQ-AST-002: Test Copy-on-Write semantics with Arc::make_mut
        let mut query = Arc::new(SelectQueryNode::new());

        // Initial unique pointer: mutates in-place (pointer unchanged)
        let ptr_before = Arc::as_ptr(&query);
        let q_mut = SelectQueryNode::clone_mut(&mut query);
        q_mut
            .selections
            .push(Arc::new(OperationNode::Selection(SelectionNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new("id"))),
            ))));
        let ptr_after = Arc::as_ptr(&query);
        assert_eq!(ptr_before, ptr_after, "Unique Arc must mutate in-place");
        assert_eq!(query.selections.len(), 1);

        // Cloned pointer: strong count = 2
        let query_snapshot = Arc::clone(&query);
        assert_eq!(Arc::strong_count(&query), 2);

        // Mutation on cloned Arc forces copy-on-write
        let q_mut_cloned = SelectQueryNode::clone_mut(&mut query);
        q_mut_cloned.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
            OperationNode::Identifier(IdentifierNode::new("users")),
        )))));

        let ptr_cow = Arc::as_ptr(&query);
        assert_ne!(
            ptr_before, ptr_cow,
            "Shared Arc must allocate a new copy on write"
        );
        assert!(query.from.is_some());
        assert!(
            query_snapshot.from.is_none(),
            "Snapshot must remain unaffected"
        );
    }
}
