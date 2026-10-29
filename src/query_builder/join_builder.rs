//! Fluent JoinBuilder context for constructing multi-condition JOIN clauses.

use std::sync::Arc;
use compact_str::CompactString;

use crate::expression::{Expression, ExpressionBuilder};
use crate::operation_node::*;

/// Builder context for constructing complex `ON` conditions in SQL joins.
// @spec REQ-SEL-003
#[derive(Debug, Clone, PartialEq)]
pub struct JoinBuilder {
    pub(crate) join_type: JoinType,
    pub(crate) table: Arc<OperationNode>,
    pub(crate) on: Option<Arc<OperationNode>>,
}

impl JoinBuilder {
    /// Creates a new JoinBuilder.
    pub fn new(join_type: JoinType, table: Arc<OperationNode>) -> Self {
        Self {
            join_type,
            table,
            on: None,
        }
    }

    /// Adds an `ON col OP val` condition.
    pub fn on(
        mut self,
        col: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        val: impl IntoValue,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp(col, operator, val);
        self.append_on(cond.to_operation_node());
        self
    }

    /// Adds an `ON col1 OP col2` reference comparison condition.
    pub fn on_ref(
        mut self,
        col1: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        col2: impl Into<CompactString>,
    ) -> Self {
        let eb = ExpressionBuilder::new();
        let cond = eb.cmp_ref(col1, operator, col2);
        self.append_on(cond.to_operation_node());
        self
    }

    /// Adds an arbitrary boolean expression to the `ON` clause via a callback.
    pub fn on_expr<F, E, T>(mut self, cb: F) -> Self
    where
        F: FnOnce(&ExpressionBuilder) -> E,
        E: Expression<T>,
    {
        let eb = ExpressionBuilder::new();
        let cond = (cb)(&eb);
        self.append_on(cond.to_operation_node());
        self
    }

    fn append_on(&mut self, next: Arc<OperationNode>) {
        if let Some(existing) = self.on.take() {
            self.on = Some(Arc::new(OperationNode::And(AndNode::new(existing, next))));
        } else {
            self.on = Some(next);
        }
    }

    /// Builds the resulting `JoinNode`.
    pub fn build(self) -> JoinNode {
        JoinNode {
            join_type: self.join_type,
            table: self.table,
            on: self.on,
        }
    }
}
