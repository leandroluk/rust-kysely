//! Raw SQL builder and helper constructors.

pub mod builder;
pub mod helpers;

pub use builder::RawBuilder;
pub use helpers::{id, join, lit, raw, table};

use compact_str::CompactString;

/// Creates a fluent `RawBuilder` initialized with the given raw SQL snippet.
// @spec REQ-EXPR-002
pub fn sql(sql_str: impl Into<CompactString>) -> RawBuilder<()> {
    RawBuilder::new(sql_str)
}
