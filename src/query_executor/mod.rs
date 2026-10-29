//! Database query execution, row extraction, and result transformation.

pub mod executor;
pub mod result;
pub mod row;

pub use executor::{
    DefaultQueryExecutor, PluginExecutor, QueryExecutor, QueryExecutorExt, SingleConnectionExecutor,
};
pub use result::QueryResult;
pub use row::{ColumnError, FromRowValue, Row};
