//! Abstract Syntax Tree (AST) operation nodes for rust-kysely.

pub mod terminal_nodes;
pub mod value;

pub use terminal_nodes::{
    ColumnNode, IdentifierNode, PrimitiveValueListNode, RawNode, ReferenceNode, TableNode,
    ValueNode,
};
pub use value::Value;

/// Top-level AST node enum representing any node in the query tree.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub enum OperationNode {
    /// Identifier node
    Identifier(IdentifierNode),
    /// Table node
    Table(TableNode),
    /// Column node
    Column(ColumnNode),
    /// Reference node
    Reference(ReferenceNode),
    /// Parameter value node
    Value(ValueNode),
    /// Primitive value list node (e.g. IN expressions)
    PrimitiveValueList(PrimitiveValueListNode),
    /// Raw SQL node
    Raw(RawNode),
}
