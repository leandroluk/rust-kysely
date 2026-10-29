//! Execution result container holding returned rows and execution metadata.

use crate::query_executor::row::Row;

/// Query result container holding returned rows and execution metadata.
// @spec REQ-EXEC-005
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryResult {
    /// Rows returned by the query.
    pub rows: Vec<Row>,
    /// Number of rows affected (for INSERT, UPDATE, DELETE).
    pub num_affected_rows: Option<u64>,
    /// Generated primary key ID (if supported by dialect).
    pub insert_id: Option<i64>,
}

impl QueryResult {
    /// Creates a new query result with rows.
    pub fn with_rows(rows: Vec<Row>) -> Self {
        Self {
            rows,
            num_affected_rows: None,
            insert_id: None,
        }
    }

    /// Creates a new query result with affected rows count.
    pub fn with_affected_rows(num_affected_rows: u64) -> Self {
        Self {
            rows: Vec::new(),
            num_affected_rows: Some(num_affected_rows),
            insert_id: None,
        }
    }

    /// Returns the number of returned rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns true if the result contains no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

impl IntoIterator for QueryResult {
    type Item = Row;
    type IntoIter = std::vec::IntoIter<Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryResult {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}
