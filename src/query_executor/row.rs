//! Database row representation and typed value extraction.

use compact_str::CompactString;
use thiserror::Error;

use crate::operation_node::Value;

/// Error occurring during column lookup or type casting on a row.
// @spec REQ-EXEC-005
#[derive(Debug, Error, PartialEq)]
pub enum ColumnError {
    /// Column not found in row
    #[error("Column '{0}' not found in query result row")]
    NotFound(CompactString),

    /// Column index out of bounds
    #[error("Column index {0} out of bounds (row has {1} columns)")]
    IndexOutOfBounds(usize, usize),

    /// Value type mismatch
    #[error("Column '{0}' has value '{1}' which cannot be converted to target type")]
    TypeMismatch(CompactString, String),
}

/// Map-like container representing a single database result row.
// @spec REQ-EXEC-005
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Row {
    pub(crate) columns: Vec<CompactString>,
    pub(crate) values: Vec<Value>,
}

impl Row {
    /// Creates a new row from columns and values.
    pub fn new(
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
        values: impl IntoIterator<Item = impl Into<Value>>,
    ) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the number of columns in this row.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if this row has no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns the column names in order.
    pub fn columns(&self) -> &[CompactString] {
        &self.columns
    }

    /// Returns the raw scalar values in order.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Finds the index of a column by name.
    pub fn index_of(&self, col: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.as_str() == col)
    }

    /// Returns the raw `Value` at the given column index.
    pub fn get_value_by_index(&self, idx: usize) -> Result<&Value, ColumnError> {
        self.values
            .get(idx)
            .ok_or(ColumnError::IndexOutOfBounds(idx, self.values.len()))
    }

    /// Returns the raw `Value` for a given column name.
    pub fn get_value(&self, col: &str) -> Result<&Value, ColumnError> {
        let idx = self.index_of(col).ok_or_else(|| ColumnError::NotFound(CompactString::new(col)))?;
        Ok(&self.values[idx])
    }
}
