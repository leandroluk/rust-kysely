//! Fluent builder for column definitions in CREATE TABLE and ALTER TABLE.

use std::sync::Arc;
use compact_str::CompactString;

use crate::expression::Expression;
use crate::operation_node::{
    ColumnDefinitionNode, IntoValue, OnModifyForeignAction, OperationNode, ReferencesNode, ValueNode,
};

/// Fluent builder for a column definition.
// @spec REQ-DDL-002, REQ-DDL-004
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinitionBuilder {
    pub(crate) node: ColumnDefinitionNode,
}

impl ColumnDefinitionBuilder {
    /// Creates a new `ColumnDefinitionBuilder` with name and SQL data type.
    pub fn new(name: impl Into<CompactString>, data_type: impl Into<CompactString>) -> Self {
        Self {
            node: ColumnDefinitionNode::new(name, data_type),
        }
    }

    /// Marks the column as a PRIMARY KEY.
    // @spec REQ-DDL-002
    pub fn primary_key(mut self) -> Self {
        self.node.is_primary_key = true;
        self
    }

    /// Marks the column as NOT NULL.
    // @spec REQ-DDL-002
    pub fn not_null(mut self) -> Self {
        self.node.is_nullable = false;
        self
    }

    /// Marks the column as UNIQUE.
    // @spec REQ-DDL-002
    pub fn unique(mut self) -> Self {
        self.node.is_unique = true;
        self
    }

    /// Sets a literal default value for the column.
    // @spec REQ-DDL-002
    pub fn default_to(mut self, val: impl IntoValue) -> Self {
        self.node.default_to = Some(Arc::new(OperationNode::Value(ValueNode::new(val.into_value()))));
        self
    }

    /// Sets an expression default value for the column (e.g. `now()`).
    // @spec REQ-DDL-002
    pub fn default_to_expr(mut self, expr: impl Expression) -> Self {
        self.node.default_to = Some(expr.to_operation_node());
        self
    }

    /// Sets a CHECK constraint expression on the column.
    // @spec REQ-DDL-002
    pub fn check(mut self, expr: impl Expression) -> Self {
        self.node.check = Some(expr.to_operation_node());
        self
    }

    /// Adds a foreign key reference (`table.column`).
    // @spec REQ-DDL-004
    pub fn references(mut self, target: &str) -> Self {
        let (tbl, col) = target.split_once('.').unwrap_or((target, "id"));
        self.node.references = Some(ReferencesNode::new(tbl, col));
        self
    }

    /// Sets the ON DELETE action for the foreign key reference.
    // @spec REQ-DDL-004
    pub fn on_delete(mut self, action: OnModifyForeignAction) -> Self {
        if let Some(ref mut refs) = self.node.references {
            refs.on_delete = Some(action);
        }
        self
    }

    /// Sets the ON UPDATE action for the foreign key reference.
    // @spec REQ-DDL-004
    pub fn on_update(mut self, action: OnModifyForeignAction) -> Self {
        if let Some(ref mut refs) = self.node.references {
            refs.on_update = Some(action);
        }
        self
    }

    /// Builds the inner `ColumnDefinitionNode`.
    pub fn build(self) -> ColumnDefinitionNode {
        self.node
    }
}
