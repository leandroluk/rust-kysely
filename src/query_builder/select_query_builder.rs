//! Fluent, immutable SelectQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;

use crate::expression::{Expression, ExpressionBuilder};
use crate::operation_node::{
    AndNode, ColumnNode, FromNode, IntoValue, JoinType, OperationNode, OrNode, ReferenceNode,
    SelectQueryNode, SelectionNode, WhereNode,
};
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

    /// Selects all columns (`*`).
    // @spec REQ-SEL-006
    pub fn select_all(mut self) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("*"))),
        ))));
        self
    }

    /// Selects all columns from a specific table (`table.*`).
    // @spec REQ-SEL-006
    pub fn select_all_from(mut self, table: impl Into<CompactString>) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        let reference = ReferenceNode::with_table(table.into(), "*");
        q.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Reference(reference)),
        ))));
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

    fn append_where(&mut self, next_cond: Arc<OperationNode>) {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(existing) = q.where_clause.take() {
            let existing_filter = match existing.as_ref() {
                OperationNode::Where(w) => w.where_expr.clone(),
                _ => existing,
            };
            let and_node = Arc::new(OperationNode::And(AndNode::new(existing_filter, next_cond)));
            q.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(and_node))));
        } else {
            q.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(next_cond))));
        }
    }

    fn append_or_where(&mut self, next_cond: Arc<OperationNode>) {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(existing) = q.where_clause.take() {
            let existing_filter = match existing.as_ref() {
                OperationNode::Where(w) => w.where_expr.clone(),
                _ => existing,
            };
            let or_node = Arc::new(OperationNode::Or(OrNode::new(existing_filter, next_cond)));
            q.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(or_node))));
        } else {
            q.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(next_cond))));
        }
    }

    /// Adds a `WHERE col OP val` filter condition to the query.
    // @spec REQ-SEL-004
    pub fn where_(
        mut self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp(col, operator, val);
        self.append_where(cond.to_operation_node());
        self
    }

    /// Adds a `WHERE col1 OP col2` reference comparison filter to the query.
    // @spec REQ-SEL-004
    pub fn where_ref(
        mut self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp_ref(col1, operator, col2);
        self.append_where(cond.to_operation_node());
        self
    }

    /// Adds an arbitrary boolean expression to the `WHERE` clause via a callback.
    // @spec REQ-SEL-004
    pub fn where_expr<F, E, T>(mut self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let cond = (cb)(&eb);
        self.append_where(cond.to_operation_node());
        self
    }

    /// Adds an `OR col OP val` filter condition to the query.
    // @spec REQ-SEL-004
    pub fn or_where_(
        mut self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp(col, operator, val);
        self.append_or_where(cond.to_operation_node());
        self
    }

    /// Adds an `OR col1 OP col2` reference comparison filter to the query.
    // @spec REQ-SEL-004
    pub fn or_where_ref(
        mut self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp_ref(col1, operator, col2);
        self.append_or_where(cond.to_operation_node());
        self
    }

    /// Adds an arbitrary boolean expression to the `WHERE` clause joined with `OR` via a callback.
    // @spec REQ-SEL-004
    pub fn or_where_expr<F, E, T>(mut self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let cond = (cb)(&eb);
        self.append_or_where(cond.to_operation_node());
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

    #[test]
    fn test_select_all() {
        let compiler = PostgresQueryCompiler::new();

        let q1 = select_from("users").select_all().compile(&compiler);
        assert_eq!(q1.sql, "SELECT * FROM \"users\"");

        let q2 = select_from("users as u")
            .select_all_from("u")
            .compile(&compiler);
        assert_eq!(q2.sql, "SELECT \"u\".* FROM \"users\" AS \"u\"");
    }

    #[test]
    fn test_select_where() {
        let compiler = PostgresQueryCompiler::new();

        // 1. where_ and where_ref
        let q1 = select_from("users as u")
            .select(["u.id", "u.name"])
            .where_("u.age", ">=", 18)
            .where_ref("u.manager_id", "!=", "u.id")
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "SELECT \"u\".\"id\", \"u\".\"name\" FROM \"users\" AS \"u\" WHERE \"u\".\"age\" >= $1 AND \"u\".\"manager_id\" != \"u\".\"id\""
        );
        assert_eq!(q1.parameters, vec![crate::operation_node::Value::from(18)]);

        // 2. or_where_ and where_expr with eb.or
        let q2 = select_from("users as u")
            .select(["u.id"])
            .where_("u.status", "=", "active")
            .or_where_("u.status", "=", "pending")
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "SELECT \"u\".\"id\" FROM \"users\" AS \"u\" WHERE \"u\".\"status\" = $1 OR \"u\".\"status\" = $2"
        );
        assert_eq!(
            q2.parameters,
            vec![
                crate::operation_node::Value::from("active"),
                crate::operation_node::Value::from("pending")
            ]
        );

        // 3. where_expr with nested eb.or
        let q3 = select_from("users as u")
            .select(["u.id"])
            .where_("u.active", "=", true)
            .where_expr(|eb| {
                eb.or([
                    eb.cmp("u.role", "=", "admin"),
                    eb.cmp("u.role", "=", "moderator"),
                ])
            })
            .compile(&compiler);

        assert_eq!(
            q3.sql,
            "SELECT \"u\".\"id\" FROM \"users\" AS \"u\" WHERE \"u\".\"active\" = $1 AND (\"u\".\"role\" = $2 OR \"u\".\"role\" = $3)"
        );
        assert_eq!(
            q3.parameters,
            vec![
                crate::operation_node::Value::from(true),
                crate::operation_node::Value::from("admin"),
                crate::operation_node::Value::from("moderator"),
            ]
        );
    }
}
