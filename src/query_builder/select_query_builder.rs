//! Fluent, immutable SelectQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;

use crate::expression::Expression;
use crate::operation_node::{FromNode, JoinType, OperationNode, SelectQueryNode};
use crate::parser::{IntoSelections, IntoTableSource};
use crate::query_builder::join_builder::JoinBuilder;
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

    /// Performs an `INNER JOIN` using a closure builder for multi-condition `ON` clauses.
    // @spec REQ-SEL-003
    pub fn inner_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Inner, table.into_table_source());
        let join_node = on(jb).build();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs an `INNER JOIN` matching two column references (`ON col1 = col2`).
    // @spec REQ-SEL-003
    pub fn inner_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.inner_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    /// Performs a `LEFT JOIN` using a closure builder for `ON` clauses.
    // @spec REQ-SEL-003
    pub fn left_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Left, table.into_table_source());
        let join_node = on(jb).build();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs a `LEFT JOIN` matching two column references (`ON col1 = col2`).
    // @spec REQ-SEL-003
    pub fn left_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.left_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    /// Performs a `RIGHT JOIN` using a closure builder.
    // @spec REQ-SEL-003
    pub fn right_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Right, table.into_table_source());
        let join_node = on(jb).build();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs a `RIGHT JOIN` matching two column references (`ON col1 = col2`).
    // @spec REQ-SEL-003
    pub fn right_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.right_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    /// Performs a `FULL JOIN` using a closure builder.
    // @spec REQ-SEL-003
    pub fn full_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Full, table.into_table_source());
        let join_node = on(jb).build();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs a `FULL JOIN` matching two column references (`ON col1 = col2`).
    // @spec REQ-SEL-003
    pub fn full_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.full_join(table, |jb| jb.on_ref(col1, "=", col2))
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

    #[test]
    fn test_select_joins() {
        let compiler = PostgresQueryCompiler::new();

        // 1. Equi-join via inner_join_ref
        let q1 = select_from("users as u")
            .select(["u.id", "o.total"])
            .inner_join_ref("orders as o", "o.user_id", "u.id")
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "SELECT \"u\".\"id\", \"o\".\"total\" FROM \"users\" AS \"u\" INNER JOIN \"orders\" AS \"o\" ON \"o\".\"user_id\" = \"u\".\"id\""
        );

        // 2. Complex join via closure builder
        let q2 = select_from("users as u")
            .select(["u.id"])
            .left_join("orders as o", |jb| {
                jb.on_ref("o.user_id", "=", "u.id")
                  .on("o.status", "=", "completed")
            })
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "SELECT \"u\".\"id\" FROM \"users\" AS \"u\" LEFT JOIN \"orders\" AS \"o\" ON \"o\".\"user_id\" = \"u\".\"id\" AND \"o\".\"status\" = $1"
        );
        assert_eq!(q2.parameters, vec![crate::operation_node::Value::from("completed")]);
    }
}
