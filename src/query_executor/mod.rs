//! Database query execution, row extraction, and result transformation.

pub mod result;
pub mod row;

pub use result::QueryResult;
pub use row::{ColumnError, Row};
