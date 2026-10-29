//! Fluent OnConflictBuilder for constructing PostgreSQL upsert (ON CONFLICT) clauses.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::{Expression, ExpressionBuilder};
use crate::operation_node::{
    ColumnNode, ColumnUpdateNode, IntoValue, OnConflictNode, OperationNode, ReferenceNode,
    ValueNode, WhereNode,
};
use crate::query_builder::select_query_builder::IntoGroupBy;

/// Builder for constructing PostgreSQL ON CONFLICT clauses.
// @spec REQ-MUT-005
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OnConflictBuilder {
    pub(crate) columns: Option<SmallVec<[Arc<OperationNode>; 4]>>,
    pub(crate) do_nothing: bool,
    pub(crate) updates: Option<SmallVec<[Arc<OperationNode>; 4]>>,
    pub(crate) where_clause: Option<Arc<OperationNode>>,
}

impl OnConflictBuilder {
    /// Creates a new empty OnConflictBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the conflict target column(s).
    pub fn column(mut self, col: impl Into<CompactString>) -> Self {
        let cols = self.columns.get_or_insert_with(SmallVec::new);
        cols.push(Arc::new(OperationNode::Column(ColumnNode::new(col))));
        self
    }

    /// Sets multiple conflict target columns.
    pub fn columns(mut self, cols: impl IntoGroupBy) -> Self {
        self.columns = Some(cols.into_group_by_nodes());
        self
    }

    /// Sets conflict action to `DO NOTHING`.
    pub fn do_nothing(mut self) -> Self {
        self.do_nothing = true;
        self.updates = None;
        self
    }

    /// Sets a column update with a scalar literal value (`col = val`).
    pub fn set(mut self, col: impl Into<CompactString>, val: impl IntoValue) -> Self {
        self.do_nothing = false;
        let update_node = Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new(col))),
            Arc::new(OperationNode::Value(ValueNode::new(val.into_value()))),
        )));
        let updates = self.updates.get_or_insert_with(SmallVec::new);
        updates.push(update_node);
        self
    }

    /// Sets a column update referencing another column or `excluded` table (`col = excluded.col`).
    // @spec REQ-MUT-006
    pub fn set_ref(
        mut self,
        col: impl Into<CompactString>,
        ref_col: impl Into<CompactString>,
    ) -> Self {
        self.do_nothing = false;
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
        let updates = self.updates.get_or_insert_with(SmallVec::new);
        updates.push(update_node);
        self
    }

    /// Sets a column update with an arbitrary expression (`col = expr`).
    pub fn set_expr<T>(
        mut self,
        col: impl Into<CompactString>,
        expr: impl Expression<T>,
    ) -> Self {
        self.do_nothing = false;
        let update_node = Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new(col))),
            expr.to_operation_node(),
        )));
        let updates = self.updates.get_or_insert_with(SmallVec::new);
        updates.push(update_node);
        self
    }

    /// Configures the update set clause using a closure builder.
    pub fn do_update_set(self, cb: impl FnOnce(Self) -> Self) -> Self {
        cb(self)
    }

    /// Adds a `WHERE col OP val` filter to the conflict update clause.
    pub fn where_(
        mut self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp(col, operator, val);
        self.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(
            cond.to_operation_node(),
        ))));
        self
    }

    /// Adds an arbitrary boolean expression to the conflict `WHERE` clause via a callback.
    pub fn where_expr<F, E, T>(mut self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let cond = (cb)(&eb);
        self.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(
            cond.to_operation_node(),
        ))));
        self
    }

    /// Builds the `OnConflictNode`.
    pub fn build(self) -> OnConflictNode {
        OnConflictNode {
            columns: self.columns,
            do_nothing: self.do_nothing,
            updates: self.updates,
            where_clause: self.where_clause,
        }
    }
}
