//! Fluent, immutable DeleteQueryBuilder implementation.

use compact_str::CompactString;
use smallvec::SmallVec;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::expression::{Expression, ExpressionBuilder};
use crate::operation_node::{
    AndNode, ColumnNode, DeleteQueryNode, IntoValue, JoinNode, JoinType, OperationNode, OrNode,
    SelectionNode, WhereNode,
};
use crate::parser::{IntoSelections, IntoTableSource};
use crate::query_builder::join_builder::JoinBuilder;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult, Row};

/// Fluent query builder for SQL DELETE statements.
// @spec REQ-MUT-009
#[derive(Clone, Default)]
pub struct DeleteQueryBuilder {
    pub(crate) node: Arc<DeleteQueryNode>,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for DeleteQueryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeleteQueryBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for DeleteQueryBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl DeleteQueryBuilder {
    /// Creates a new empty DELETE query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder wrapping an existing AST node.
    pub fn from_node(node: Arc<DeleteQueryNode>) -> Self {
        Self {
            node,
            executor: None,
        }
    }

    /// Attaches an executor to the query builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Attaches an additional per-query plugin to this query builder.
    // @spec REQ-PLUG-006
    pub fn with_plugin<P: crate::plugin::KyselyPlugin + 'static>(mut self, plugin: P) -> Self {
        if let Some(exec) = self.executor.take() {
            self.executor = Some(Arc::new(crate::query_executor::PluginExecutor::new(
                exec,
                Arc::new(plugin),
            )));
        }
        self
    }

    /// Returns a reference to the executor if attached.
    pub fn executor(&self) -> Option<&Arc<dyn QueryExecutor>> {
        self.executor.as_ref()
    }

    /// Executes the query and returns the `QueryResult`.
    // @spec REQ-EXEC-004
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor
            .execute_node(&OperationNode::DeleteQuery((*self.node).clone()))
            .await
    }

    /// Executes the query and returns the first row, if any.
    // @spec REQ-EXEC-007
    pub async fn execute_take_first(&self) -> Result<Option<Row>, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor
            .execute_node_take_first(&OperationNode::DeleteQuery((*self.node).clone()))
            .await
    }

    /// Executes the query and returns the first row, or returns an error if empty.
    // @spec REQ-EXEC-007
    pub async fn execute_take_first_or_throw(&self) -> Result<Row, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor
            .execute_node_take_first_or_throw(&OperationNode::DeleteQuery((*self.node).clone()))
            .await
    }

    /// Executes the query and deserializes rows into a `Vec<T>`.
    // @spec REQ-EXEC-006
    #[cfg(feature = "json")]
    pub async fn execute_into<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<Vec<T>, DriverError> {
        let res = self.execute().await?;
        res.rows
            .into_iter()
            .map(|r| {
                r.into_record::<T>()
                    .map_err(|e| DriverError::ExecutionFailed(e.to_string()))
            })
            .collect()
    }

    /// Executes the query and deserializes the first row into `Option<T>`.
    // @spec REQ-EXEC-006
    #[cfg(feature = "json")]
    pub async fn execute_take_first_into<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<Option<T>, DriverError> {
        if let Some(row) = self.execute_take_first().await? {
            let record = row
                .into_record::<T>()
                .map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Adds a `USING` source for multi-table deletes (PostgreSQL `DELETE FROM ... USING`).
    // @spec REQ-MUT-009
    pub fn using(mut self, table: impl IntoTableSource) -> Self {
        let join_node = JoinNode::new(JoinType::Using, table.into_table_source());
        let q = DeleteQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs an `INNER JOIN` on the delete query.
    // @spec REQ-MUT-009
    pub fn inner_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Inner, table.into_table_source());
        let join_node = on(jb).build();
        let q = DeleteQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs an `INNER JOIN` matching two column references.
    // @spec REQ-MUT-009
    pub fn inner_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.inner_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    /// Performs a `LEFT JOIN` on the delete query.
    // @spec REQ-MUT-009
    pub fn left_join(
        mut self,
        table: impl IntoTableSource,
        on: impl FnOnce(JoinBuilder) -> JoinBuilder,
    ) -> Self {
        let jb = JoinBuilder::new(JoinType::Left, table.into_table_source());
        let join_node = on(jb).build();
        let q = DeleteQueryNode::clone_mut(&mut self.node);
        q.joins.push(Arc::new(OperationNode::Join(join_node)));
        self
    }

    /// Performs a `LEFT JOIN` matching two column references.
    // @spec REQ-MUT-009
    pub fn left_join_ref(
        self,
        table: impl IntoTableSource,
        col1: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.left_join(table, |jb| jb.on_ref(col1, "=", col2))
    }

    fn append_where(&mut self, next_cond: Arc<OperationNode>) {
        let q = DeleteQueryNode::clone_mut(&mut self.node);
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
        let q = DeleteQueryNode::clone_mut(&mut self.node);
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

    /// Adds a `WHERE col OP val` filter to the DELETE query.
    ///
    /// Alias for `r#where` and [`filter`](Self::filter).
    // @spec REQ-MUT-009
    pub fn where_(
        self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        self.r#where(col, operator, val)
    }

    /// Canonical Rust alias for SQL `WHERE col OP val` filter condition.
    ///
    /// In Rust, `where` is a reserved keyword for trait bounds.
    /// Use `filter` as the idiomatic, clean alternative to `.r#where(...)` or `.where_(...)`.
    // @spec REQ-MUT-009
    #[inline]
    pub fn filter(
        self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        self.r#where(col, operator, val)
    }

    /// Adds a `WHERE col OP val` filter condition using Rust raw identifier `r#where`.
    // @spec REQ-MUT-009
    pub fn r#where(
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

    /// Canonical Rust alias for SQL `WHERE col1 OP col2` reference comparison.
    // @spec REQ-MUT-009
    #[inline]
    pub fn filter_ref(
        self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.where_ref(col1, operator, col2)
    }

    /// Adds a `WHERE col1 OP col2` reference comparison filter to the query.
    // @spec REQ-MUT-009
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

    /// Canonical Rust alias for arbitrary boolean expressions in `WHERE`.
    // @spec REQ-MUT-009
    #[inline]
    pub fn filter_expr<F, E, T>(self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        self.where_expr(cb)
    }

    /// Adds an arbitrary boolean expression to the `WHERE` clause via a callback.
    // @spec REQ-MUT-009
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

    /// Canonical Rust alias for SQL `OR col OP val` filter condition.
    // @spec REQ-MUT-009
    #[inline]
    pub fn or_filter(
        self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        self.or_where_(col, operator, val)
    }

    /// Adds an `OR col OP val` filter condition to the query.
    // @spec REQ-MUT-009
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

    /// Canonical Rust alias for `OR col1 OP col2` reference comparison.
    // @spec REQ-MUT-009
    #[inline]
    pub fn or_filter_ref(
        self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        self.or_where_ref(col1, operator, col2)
    }

    /// Adds an `OR col1 OP col2` reference comparison filter to the query.
    // @spec REQ-MUT-009
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
        let q = DeleteQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.extend(sel_nodes);
        self
    }

    /// Selects all columns in the `RETURNING` clause (`RETURNING *`).
    // @spec REQ-MUT-010
    pub fn returning_all(mut self) -> Self {
        let q = DeleteQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("*"))),
        ))));
        self
    }

    /// Compiles the query using the specified query compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&OperationNode::DeleteQuery((*self.node).clone()))
    }
}

impl Expression for DeleteQueryBuilder {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::DeleteQuery((*self.node).clone()))
    }
}

/// Entry point creating a `DeleteQueryBuilder` targeting the specified table.
// @spec REQ-MUT-009
pub fn delete_from(table: impl IntoTableSource) -> DeleteQueryBuilder {
    let mut builder = DeleteQueryBuilder::new();
    let q = DeleteQueryNode::clone_mut(&mut builder.node);
    q.from = Some(table.into_table_source());
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;

    #[test]
    fn test_delete_query() {
        let compiler = PostgresQueryCompiler::new();

        // 1. Basic delete with where
        let q1 = delete_from("users")
            .where_("active", "=", false)
            .where_("age", "<", 18)
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "DELETE FROM \"users\" WHERE \"active\" = $1 AND \"age\" < $2"
        );
        assert_eq!(q1.parameters, vec![Value::from(false), Value::from(18),]);

        // 2. Delete with using and reference condition
        let q2 = delete_from("users as u")
            .using("inactive_accounts as ia")
            .where_ref("u.account_id", "=", "ia.id")
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "DELETE FROM \"users\" AS \"u\" USING \"inactive_accounts\" AS \"ia\" WHERE \"u\".\"account_id\" = \"ia\".\"id\""
        );
        assert!(q2.parameters.is_empty());
    }
}
