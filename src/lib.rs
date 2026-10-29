//! # rust-kysely
//!
//! Pure Rust, type-safe SQL query builder and runner ported from Kysely (TypeScript) without macros.

#![warn(missing_docs)]

pub mod operation_node;
pub mod util;

pub use operation_node::{
    ColumnNode, IdentifierNode, OperationNode, PrimitiveValueListNode, RawNode, ReferenceNode,
    TableNode, Value, ValueNode,
};
pub use util::QueryId;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
