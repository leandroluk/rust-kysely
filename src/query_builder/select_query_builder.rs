//! Fluent, immutable SelectQueryBuilder implementation.

use std::sync::Arc;

use crate::expression::Expression;
use crate::operation_node::{FromNode, OperationNode, SelectQueryNode};
use crate::parser::{IntoSelections, IntoTableSource};
use crate::query_compiler::{CompiledQuery, QueryCompiler};

/// Fluent query builder for SQL SELECT statements.
// @spec REQ-SEL-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectQueryBuilder {
    pub(crate) node: Arc<SelectQueryNode>,
}

impl SelectQueryBuilder {
    /// Creates a new empty SELECT query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder wrapping an existing AST node.
    pub fn from_node(node: Arc<SelectQueryNode>) -> Self {
        Self { node }
    }

    /// Appends projections to the selection list using heterogeneous tuples, arrays, or closures.
    // @spec REQ-SEL-005
    pub fn select(mut self, selections: impl IntoSelections) -> Self {
        let sel_nodes = selections.into_selections();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.selections.extend(sel_nodes);
        self
    }

    /// Compiles the query using the specified query compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&OperationNode::SelectQuery((*self.node).clone()))
    }
}

impl Expression for SelectQueryBuilder {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::SelectQuery((*self.node).clone()))
    }
}

/// Entry point creating a `SelectQueryBuilder` targeting the specified table or source.
// @spec REQ-SEL-002
pub fn select_from(table: impl IntoTableSource) -> SelectQueryBuilder {
    let mut builder = SelectQueryBuilder::new();
    let q = SelectQueryNode::clone_mut(&mut builder.node);
    q.from = Some(Arc::new(OperationNode::From(FromNode::new(
        table.into_table_source(),
    ))));
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;

    #[test]
    fn test_select_from() {
        let compiler = PostgresQueryCompiler::new();
        let query = select_from("users as u")
            .select(["u.id", "u.name"])
            .compile(&compiler);

        assert_eq!(
            query.sql,
            "SELECT \"u\".\"id\", \"u\".\"name\" FROM \"users\" AS \"u\""
        );
        assert!(query.parameters.is_empty());
    }
}
