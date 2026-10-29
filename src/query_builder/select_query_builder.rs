//! Fluent, immutable SelectQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::{Expression, ExpressionBuilder, ExpressionWrapper};
use crate::operation_node::{
    AndNode, ColumnNode, CommonTableExpressionNode, FromNode, GroupByNode, HavingNode,
    IdentifierNode, IntoValue, JoinType, LimitNode, OffsetNode, OperationNode, OrderByItemNode,
    OrderByNode, OrderDirection, OrderNulls, OrNode, ReferenceNode, SelectQueryNode, SelectionNode,
    SetOperationNode, SetOperator, ValueNode, WhereNode, WithNode,
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

    fn append_having(&mut self, next_cond: Arc<OperationNode>) {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(existing) = q.having.take() {
            let existing_filter = match existing.as_ref() {
                OperationNode::Having(h) => h.having_expr.clone(),
                _ => existing,
            };
            let and_node = Arc::new(OperationNode::And(AndNode::new(existing_filter, next_cond)));
            q.having = Some(Arc::new(OperationNode::Having(HavingNode::new(and_node))));
        } else {
            q.having = Some(Arc::new(OperationNode::Having(HavingNode::new(next_cond))));
        }
    }

    /// Adds a `GROUP BY` clause with one or more grouping items.
    // @spec REQ-SEL-007
    pub fn group_by(mut self, items: impl IntoGroupBy) -> Self {
        let nodes = items.into_group_by_nodes();
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(ref existing) = q.group_by {
            if let OperationNode::GroupBy(gb) = existing.as_ref() {
                let mut combined = gb.items.clone();
                combined.extend(nodes);
                q.group_by = Some(Arc::new(OperationNode::GroupBy(GroupByNode::new(combined))));
                return self;
            }
        }
        q.group_by = Some(Arc::new(OperationNode::GroupBy(GroupByNode::new(nodes))));
        self
    }

    /// Adds a `HAVING col OP val` filter to the query.
    // @spec REQ-SEL-007
    pub fn having(
        mut self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp(col, operator, val);
        self.append_having(cond.to_operation_node());
        self
    }

    /// Adds a `HAVING col1 OP col2` reference comparison filter to the query.
    // @spec REQ-SEL-007
    pub fn having_ref(
        mut self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp_ref(col1, operator, col2);
        self.append_having(cond.to_operation_node());
        self
    }

    /// Adds an arbitrary boolean expression to the `HAVING` clause via a callback.
    // @spec REQ-SEL-007
    pub fn having_expr<F, E, T>(mut self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let cond = (cb)(&eb);
        self.append_having(cond.to_operation_node());
        self
    }

    /// Adds an `ORDER BY` item with direction (`"asc"` or `"desc"` or `OrderDirection`).
    // @spec REQ-SEL-008
    pub fn order_by(
        mut self,
        expr: impl IntoOrderByExpr,
        direction: impl IntoOrderDirection,
    ) -> Self {
        let item = OrderByItemNode::new(
            expr.into_order_by_expr(),
            direction.into_order_direction(),
        );
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(ref existing) = q.order_by {
            if let OperationNode::OrderBy(ob) = existing.as_ref() {
                let mut combined = ob.items.clone();
                combined.push(Arc::new(OperationNode::OrderByItem(item)));
                q.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new(combined))));
                return self;
            }
        }
        q.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new([
            Arc::new(OperationNode::OrderByItem(item)),
        ]))));
        self
    }

    /// Adds an `ORDER BY` item with direction and nulls positioning.
    // @spec REQ-SEL-008
    pub fn order_by_nulls(
        mut self,
        expr: impl IntoOrderByExpr,
        direction: impl IntoOrderDirection,
        nulls: OrderNulls,
    ) -> Self {
        let item = OrderByItemNode::with_nulls(
            expr.into_order_by_expr(),
            direction.into_order_direction(),
            nulls,
        );
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(ref existing) = q.order_by {
            if let OperationNode::OrderBy(ob) = existing.as_ref() {
                let mut combined = ob.items.clone();
                combined.push(Arc::new(OperationNode::OrderByItem(item)));
                q.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new(combined))));
                return self;
            }
        }
        q.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new([
            Arc::new(OperationNode::OrderByItem(item)),
        ]))));
        self
    }

    /// Sets the `LIMIT` count for pagination.
    // @spec REQ-SEL-009
    pub fn limit(mut self, limit: i64) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.limit = Some(Arc::new(OperationNode::Limit(LimitNode::new(Arc::new(
            OperationNode::Value(ValueNode::new(limit)),
        )))));
        self
    }

    /// Sets the `OFFSET` count for pagination.
    // @spec REQ-SEL-009
    pub fn offset(mut self, offset: i64) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.offset = Some(Arc::new(OperationNode::Offset(OffsetNode::new(Arc::new(
            OperationNode::Value(ValueNode::new(offset)),
        )))));
        self
    }

    /// Marks the query with `DISTINCT`.
    // @spec REQ-SEL-009
    pub fn distinct(mut self) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.distinct_on = Some(SmallVec::new());
        self
    }

    /// Marks the query with `DISTINCT ON` for specific columns (PostgreSQL).
    // @spec REQ-SEL-009
    pub fn distinct_on(mut self, cols: impl IntoGroupBy) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.distinct_on = Some(cols.into_group_by_nodes().into_iter().collect());
        self
    }

    fn append_cte(
        &mut self,
        recursive: bool,
        name: impl Into<CompactString>,
        expression: Arc<OperationNode>,
    ) {
        let cte_item = Arc::new(OperationNode::CommonTableExpression(
            CommonTableExpressionNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new(name))),
                expression,
            ),
        ));
        let q = SelectQueryNode::clone_mut(&mut self.node);
        if let Some(ref existing) = q.with {
            if let OperationNode::With(w) = existing.as_ref() {
                let mut expressions = w.expressions.clone();
                expressions.push(cte_item);
                let is_rec = w.recursive || recursive;
                q.with = Some(Arc::new(OperationNode::With(WithNode::new(
                    is_rec,
                    expressions,
                ))));
                return;
            }
        }
        q.with = Some(Arc::new(OperationNode::With(WithNode::new(
            recursive,
            [cte_item],
        ))));
    }

    /// Adds a Common Table Expression (`WITH cte AS (...)`).
    // @spec REQ-SEL-010
    pub fn with(mut self, name: impl Into<CompactString>, expr: impl Expression) -> Self {
        self.append_cte(false, name, expr.to_operation_node());
        self
    }

    /// Adds a Common Table Expression via a builder callback (`WITH cte AS (...)`).
    // @spec REQ-SEL-010
    pub fn with_expr<F, E, T>(mut self, name: impl Into<CompactString>, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let expr = (cb)(&eb);
        self.append_cte(false, name, expr.to_operation_node());
        self
    }

    /// Adds a recursive Common Table Expression (`WITH RECURSIVE cte AS (...)`).
    // @spec REQ-SEL-010
    pub fn with_recursive(
        mut self,
        name: impl Into<CompactString>,
        expr: impl Expression,
    ) -> Self {
        self.append_cte(true, name, expr.to_operation_node());
        self
    }

    /// Adds a recursive Common Table Expression via a builder callback.
    // @spec REQ-SEL-010
    pub fn with_recursive_expr<F, E, T>(mut self, name: impl Into<CompactString>, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let expr = (cb)(&eb);
        self.append_cte(true, name, expr.to_operation_node());
        self
    }

    /// Combines this query with another using `UNION`.
    // @spec REQ-SEL-010
    pub fn union(mut self, other: impl Expression) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.set_operations.push(Arc::new(OperationNode::SetOperation(
            SetOperationNode::new(SetOperator::Union, other.to_operation_node()),
        )));
        self
    }

    /// Combines this query with another using `UNION ALL`.
    // @spec REQ-SEL-010
    pub fn union_all(mut self, other: impl Expression) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.set_operations.push(Arc::new(OperationNode::SetOperation(
            SetOperationNode::new(SetOperator::UnionAll, other.to_operation_node()),
        )));
        self
    }

    /// Combines this query with another using `INTERSECT`.
    // @spec REQ-SEL-010
    pub fn intersect(mut self, other: impl Expression) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.set_operations.push(Arc::new(OperationNode::SetOperation(
            SetOperationNode::new(SetOperator::Intersect, other.to_operation_node()),
        )));
        self
    }

    /// Combines this query with another using `EXCEPT`.
    // @spec REQ-SEL-010
    pub fn except(mut self, other: impl Expression) -> Self {
        let q = SelectQueryNode::clone_mut(&mut self.node);
        q.set_operations.push(Arc::new(OperationNode::SetOperation(
            SetOperationNode::new(SetOperator::Except, other.to_operation_node()),
        )));
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

/// Trait for converting items into order by expressions.
// @spec REQ-SEL-008
pub trait IntoOrderByExpr {
    /// Converts this item into an AST node.
    fn into_order_by_expr(self) -> Arc<OperationNode>;
}

impl IntoOrderByExpr for &str {
    fn into_order_by_expr(self) -> Arc<OperationNode> {
        if let Some((tbl, col)) = self.split_once('.') {
            Arc::new(OperationNode::Reference(ReferenceNode::with_table(tbl, col)))
        } else {
            Arc::new(OperationNode::Column(ColumnNode::new(self)))
        }
    }
}

impl IntoOrderByExpr for CompactString {
    fn into_order_by_expr(self) -> Arc<OperationNode> {
        self.as_str().into_order_by_expr()
    }
}

impl IntoOrderByExpr for String {
    fn into_order_by_expr(self) -> Arc<OperationNode> {
        self.as_str().into_order_by_expr()
    }
}

impl<T> IntoOrderByExpr for ExpressionWrapper<T> {
    fn into_order_by_expr(self) -> Arc<OperationNode> {
        self.to_operation_node()
    }
}

impl IntoOrderByExpr for Arc<OperationNode> {
    fn into_order_by_expr(self) -> Arc<OperationNode> {
        self
    }
}

/// Trait for converting values into an `OrderDirection`.
// @spec REQ-SEL-008
pub trait IntoOrderDirection {
    /// Converts this item into an `OrderDirection`.
    fn into_order_direction(self) -> OrderDirection;
}

impl IntoOrderDirection for OrderDirection {
    fn into_order_direction(self) -> OrderDirection {
        self
    }
}

impl IntoOrderDirection for &str {
    fn into_order_direction(self) -> OrderDirection {
        if self.eq_ignore_ascii_case("desc") {
            OrderDirection::Desc
        } else {
            OrderDirection::Asc
        }
    }
}

/// Trait for converting types into a list of grouping AST nodes.
// @spec REQ-SEL-007
pub trait IntoGroupBy {
    /// Converts this into a collection of AST nodes.
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]>;
}

impl IntoGroupBy for &str {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let mut v = SmallVec::new();
        v.push(self.into_order_by_expr());
        v
    }
}

impl IntoGroupBy for CompactString {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.as_str().into_group_by_nodes()
    }
}

impl<const N: usize> IntoGroupBy for [&str; N] {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter().map(|s| s.into_order_by_expr()).collect()
    }
}

impl<const N: usize> IntoGroupBy for [CompactString; N] {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter().map(|s| s.into_order_by_expr()).collect()
    }
}

impl IntoGroupBy for Vec<&str> {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter().map(|s| s.into_order_by_expr()).collect()
    }
}

impl IntoGroupBy for Vec<CompactString> {
    fn into_group_by_nodes(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter().map(|s| s.into_order_by_expr()).collect()
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

    #[test]
    fn test_select_clauses() {
        let compiler = PostgresQueryCompiler::new();

        // 1. group_by, having, order_by, limit, offset
        let q1 = select_from("orders as o")
            .select(["o.user_id", "o.status"])
            .group_by(["o.user_id", "o.status"])
            .having("o.status", "=", "completed")
            .order_by("o.user_id", "desc")
            .limit(10)
            .offset(20)
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "SELECT \"o\".\"user_id\", \"o\".\"status\" FROM \"orders\" AS \"o\" GROUP BY \"o\".\"user_id\", \"o\".\"status\" HAVING \"o\".\"status\" = $1 ORDER BY \"o\".\"user_id\" DESC LIMIT $2 OFFSET $3"
        );
        assert_eq!(
            q1.parameters,
            vec![
                crate::operation_node::Value::from("completed"),
                crate::operation_node::Value::from(10),
                crate::operation_node::Value::from(20),
            ]
        );

        // 2. distinct and distinct_on
        let q2 = select_from("users as u")
            .select(["u.id", "u.name"])
            .distinct()
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "SELECT DISTINCT \"u\".\"id\", \"u\".\"name\" FROM \"users\" AS \"u\""
        );

        let q3 = select_from("users as u")
            .select(["u.id", "u.email"])
            .distinct_on(["u.email"])
            .compile(&compiler);

        assert_eq!(
            q3.sql,
            "SELECT DISTINCT ON (\"u\".\"email\") \"u\".\"id\", \"u\".\"email\" FROM \"users\" AS \"u\""
        );

        // 3. order_by_nulls
        let q4 = select_from("users as u")
            .select(["u.id"])
            .order_by_nulls("u.age", "asc", OrderNulls::First)
            .compile(&compiler);

        assert_eq!(
            q4.sql,
            "SELECT \"u\".\"id\" FROM \"users\" AS \"u\" ORDER BY \"u\".\"age\" ASC NULLS FIRST"
        );
    }

    #[test]
    fn test_select_with_and_set_ops() {
        let compiler = PostgresQueryCompiler::new();

        // 1. CTE: WITH cte AS (SELECT ...)
        let cte_query = select_from("users").select(["id"]).where_("active", "=", true);
        let q1 = select_from("active_users")
            .with("active_users", cte_query)
            .select_all()
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "WITH \"active_users\" AS (SELECT \"id\" FROM \"users\" WHERE \"active\" = $1) SELECT * FROM \"active_users\""
        );
        assert_eq!(q1.parameters, vec![crate::operation_node::Value::from(true)]);

        // 2. UNION & INTERSECT & EXCEPT
        let q_a = select_from("users").select(["id"]).where_("role", "=", "admin");
        let q_b = select_from("users").select(["id"]).where_("role", "=", "moderator");
        let q_union = q_a.union(q_b).compile(&compiler);

        assert_eq!(
            q_union.sql,
            "SELECT \"id\" FROM \"users\" WHERE \"role\" = $1 UNION SELECT \"id\" FROM \"users\" WHERE \"role\" = $2"
        );
        assert_eq!(
            q_union.parameters,
            vec![
                crate::operation_node::Value::from("admin"),
                crate::operation_node::Value::from("moderator"),
            ]
        );
    }
}
