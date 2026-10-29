//! Fluent builders for table-level constraints (foreign keys, primary keys, checks).

use compact_str::CompactString;
use smallvec::SmallVec;

use crate::operation_node::{OnModifyForeignAction, TableConstraintNode};

/// Fluent builder for foreign key table constraints.
// @spec REQ-DDL-003
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKeyConstraintBuilder {
    pub(crate) name: Option<CompactString>,
    pub(crate) columns: SmallVec<[CompactString; 4]>,
    pub(crate) target_table: CompactString,
    pub(crate) target_columns: SmallVec<[CompactString; 4]>,
    pub(crate) on_delete: Option<OnModifyForeignAction>,
    pub(crate) on_update: Option<OnModifyForeignAction>,
}

impl ForeignKeyConstraintBuilder {
    /// Creates a new `ForeignKeyConstraintBuilder`.
    pub fn new(
        name: Option<impl Into<CompactString>>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
        target_table: impl Into<CompactString>,
        target_columns: impl IntoIterator<Item = impl Into<CompactString>>,
    ) -> Self {
        Self {
            name: name.map(Into::into),
            columns: columns.into_iter().map(Into::into).collect(),
            target_table: target_table.into(),
            target_columns: target_columns.into_iter().map(Into::into).collect(),
            on_delete: None,
            on_update: None,
        }
    }

    /// Sets the action to take when the referenced row is deleted.
    pub fn on_delete(mut self, action: OnModifyForeignAction) -> Self {
        self.on_delete = Some(action);
        self
    }

    /// Sets the action to take when the referenced row is updated.
    pub fn on_update(mut self, action: OnModifyForeignAction) -> Self {
        self.on_update = Some(action);
        self
    }

    /// Builds the `TableConstraintNode`.
    pub fn build(self) -> TableConstraintNode {
        TableConstraintNode::ForeignKey {
            name: self.name,
            columns: self.columns,
            target_table: self.target_table,
            target_columns: self.target_columns,
            on_delete: self.on_delete,
            on_update: self.on_update,
        }
    }
}
