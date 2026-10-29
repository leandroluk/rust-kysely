//! Fluent, immutable UpdateQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::{Expression, ExpressionBuilder};
use crate::operation_node::{
    AndNode, ColumnNode, ColumnUpdateNode, FromNode, IntoValue, JoinType, OperationNode, OrNode,
    ReferenceNode, SelectionNode, UpdateQueryNode, ValueNode, WhereNode,
};
use crate::parser::{IntoSelections, IntoTableSource};
use crate::query_builder::join_builder::JoinBuilder;
use crate::query_compiler::{CompiledQuery, QueryCompiler};

/// Fluent query builder for SQL UPDATE statements.
// @spec REQ-MUT-007
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UpdateQueryBuilder {
    pub(crate) node: Arc<UpdateQueryNode>,
}

impl UpdateQueryBuilder {
    /// Creates a new empty UPDATE query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder wrapping an existing AST node.
    pub fn from_node(node: Arc<UpdateQueryNode>) -> Self {
        Self { node }
    }

    /// Sets a column update with a literal value (`.set("name", "Alice")`).
    // @spec REQ-MUT-007
    pub fn set(mut self, col: impl Into<CompactString>, val: impl IntoValue) -> Self {
        let update_node = Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new(col))),
            Arc::new(OperationNode::Value(ValueNode::new(val.into_value()))),
        )));
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.updates.push(update_node);
        self
    }

    /// Sets a column update referencing another column (`.set_ref("target", "source")`).
    // @spec REQ-MUT-007
    pub fn set_ref(
        mut self,
        col: impl Into<CompactString>,
        ref_col: impl Into<CompactString>,
    ) -> Self {
        let ref_str: CompactString = ref_col.into();
        let ref_node = if let Some((tbl, c)) = ref_str.split_once('.') {
            ReferenceNode::with_table(tbl, c)
        } else {
            ReferenceNode::new(ref_str)
        };
        let update_node = Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new(col))),
            Arc::new(OperationNode::Reference(ref_node)),
        )));
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.updates.push(update_node);
        self
    }

    /// Sets a column update with an arbitrary expression (`.set_expr("total", eb.fn().coalesce(...))`).
    // @spec REQ-MUT-007
    pub fn set_expr<T>(
        mut self,
        col: impl Into<CompactString>,
        expr: impl Expression<T>,
    ) -> Self {
        let update_node = Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new(col))),
            expr.to_operation_node(),
        )));
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.updates.push(update_node);
        self
    }

    /// Adds a `FROM` clause for cross-table updates (PostgreSQL `UPDATE ... FROM`).
    // @spec REQ-MUT-008
    pub fn from(mut self, source: impl IntoTableSource) -> Self {
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.from = Some(Arc::new(OperationNode::From(FromNode::new(
            source.into_table_source(),
        ))));
        self
    }

    /// Performs an `INNER JOIN` on the update query.
    // @spec REQ-MUT-008
    pub fn inner_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Inner, table.into_table_source());
        let join_node = on(jb).build();
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs an `INNER JOIN` matching two column references.
    // @spec REQ-MUT-008
    pub fn inner_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.inner_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    /// Performs a `LEFT JOIN` on the update query.
    // @spec REQ-MUT-008
    pub fn left_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Left, table.into_table_source());
        let join_node = on(jb).build();
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs a `LEFT JOIN` matching two column references.
    // @spec REQ-MUT-008
    pub fn left_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.left_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    fn append_where(&mut self, next_cond: Arc<OperationNode>) {
        let q = UpdateQueryNode::clone_mut(&mut self.node);
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
        let q = UpdateQueryNode::clone_mut(&mut self.node);
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

    /// Adds a `WHERE col OP val` filter to the UPDATE query.
    // @spec REQ-MUT-008
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
    // @spec REQ-MUT-008
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
    // @spec REQ-MUT-008
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
    // @spec REQ-MUT-008
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
    // @spec REQ-MUT-008
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

    /// Appends selections to the `RETURNING` clause.
    // @spec REQ-MUT-010
    pub fn returning(mut self, selections: impl IntoSelections) -> Self {
        let sel_nodes = selections.into_selections();
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.extend(sel_nodes);
        self
    }

    /// Selects all columns in the `RETURNING` clause (`RETURNING *`).
    // @spec REQ-MUT-010
    pub fn returning_all(mut self) -> Self {
        let q = UpdateQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("*"))),
        ))));
        self
    }

    /// Compiles the query using the specified query compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&OperationNode::UpdateQuery((*self.node).clone()))
    }
}

impl Expression for UpdateQueryBuilder {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::UpdateQuery((*self.node).clone()))
    }
}

/// Entry point creating an `UpdateQueryBuilder` targeting the specified table.
// @spec REQ-MUT-007
pub fn update_table(table: impl IntoTableSource) -> UpdateQueryBuilder {
    let mut builder = UpdateQueryBuilder::new();
    let q = UpdateQueryNode::clone_mut(&mut builder.node);
    q.table = Some(table.into_table_source());
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;

    #[test]
    fn test_update_query() {
        let compiler = PostgresQueryCompiler::new();

        // 1. Basic update with set and where
        let q1 = update_table("users")
            .set("first_name", "Bob")
            .set("age", 35)
            .where_("id", "=", 42)
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "UPDATE \"users\" SET \"first_name\" = $1, \"age\" = $2 WHERE \"id\" = $3"
        );
        assert_eq!(
            q1.parameters,
            vec![
                Value::from("Bob"),
                Value::from(35),
                Value::from(42),
            ]
        );

        // 2. Complex update with set_ref, from, join, and where_expr
        let q2 = update_table("users as u")
            .set_ref("manager_name", "m.name")
            .from("managers as m")
            .inner_join_ref("departments as d", "d.id", "m.dept_id")
            .where_ref("u.manager_id", "=", "m.id")
            .where_("d.active", "=", true)
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "UPDATE \"users\" AS \"u\" SET \"manager_name\" = \"m\".\"name\" FROM \"managers\" AS \"m\" INNER JOIN \"departments\" AS \"d\" ON \"d\".\"id\" = \"m\".\"dept_id\" WHERE \"u\".\"manager_id\" = \"m\".\"id\" AND \"d\".\"active\" = $1"
        );
        assert_eq!(q2.parameters, vec![Value::from(true)]);
    }
}
