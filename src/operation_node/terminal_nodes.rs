//! Terminal AST nodes representing identifiers, tables, columns, references, and raw SQL.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::operation_node::{OperationNode, Value};

/// An identifier node representing a table name, column name, alias, etc.
// @spec REQ-AST-003
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentifierNode {
    /// Identifier name (stored inline if <= 24 bytes).
    pub name: CompactString,
}

impl IdentifierNode {
    /// Creates a new identifier node.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self { name: name.into() }
    }
}

/// A database table reference.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct TableNode {
    /// Optional schema qualification.
    pub schema: Option<Arc<OperationNode>>,
    /// Table name or expression.
    pub table: Arc<OperationNode>,
}

impl TableNode {
    /// Creates a new table node without schema.
    pub fn new(table: impl Into<CompactString>) -> Self {
        Self {
            schema: None,
            table: Arc::new(OperationNode::Identifier(IdentifierNode::new(table))),
        }
    }

    /// Creates a new table node with explicit schema qualification.
    pub fn with_schema(schema: impl Into<CompactString>, table: impl Into<CompactString>) -> Self {
        Self {
            schema: Some(Arc::new(OperationNode::Identifier(IdentifierNode::new(schema)))),
            table: Arc::new(OperationNode::Identifier(IdentifierNode::new(table))),
        }
    }
}

/// A database column reference.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnNode {
    /// Column identifier.
    pub column: Arc<OperationNode>,
}

impl ColumnNode {
    /// Creates a new column node from a string identifier.
    pub fn new(column: impl Into<CompactString>) -> Self {
        Self {
            column: Arc::new(OperationNode::Identifier(IdentifierNode::new(column))),
        }
    }
}

/// A qualified reference node (e.g. `table.column`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceNode {
    /// Optional table qualification.
    pub table: Option<Arc<OperationNode>>,
    /// Column reference.
    pub column: Arc<OperationNode>,
}

impl ReferenceNode {
    /// Creates an unqualified reference node (`column`).
    pub fn new(column: impl Into<CompactString>) -> Self {
        Self {
            table: None,
            column: Arc::new(OperationNode::Column(ColumnNode::new(column))),
        }
    }

    /// Creates a table-qualified reference node (`table.column`).
    pub fn with_table(table: impl Into<CompactString>, column: impl Into<CompactString>) -> Self {
        Self {
            table: Some(Arc::new(OperationNode::Table(TableNode::new(table)))),
            column: Arc::new(OperationNode::Column(ColumnNode::new(column))),
        }
    }
}

/// A scalar value parameter node.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct ValueNode {
    /// Parameter value.
    pub value: Value,
}

impl ValueNode {
    /// Creates a new scalar value node.
    pub fn new(value: impl Into<Value>) -> Self {
        Self { value: value.into() }
    }
}

/// A list of primitive value nodes (e.g. for `IN (...)`).
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveValueListNode {
    /// Parameter values.
    pub values: SmallVec<[Value; 4]>,
}

impl PrimitiveValueListNode {
    /// Creates a new primitive value list node.
    pub fn new(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

/// Raw SQL snippet with embedded parameters or expression nodes.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub struct RawNode {
    /// SQL text fragments between parameters.
    pub sql_fragments: SmallVec<[CompactString; 2]>,
    /// Parameter or node expressions interpolated between fragments.
    pub parameters: SmallVec<[Arc<OperationNode>; 2]>,
}

impl RawNode {
    /// Creates a raw SQL node without parameters.
    pub fn new(sql: impl Into<CompactString>) -> Self {
        let mut sql_fragments = SmallVec::new();
        sql_fragments.push(sql.into());
        Self {
            sql_fragments,
            parameters: SmallVec::new(),
        }
    }

    /// Creates a raw SQL node with parameters.
    pub fn with_parameters(
        sql_fragments: SmallVec<[CompactString; 2]>,
        parameters: SmallVec<[Arc<OperationNode>; 2]>,
    ) -> Self {
        Self {
            sql_fragments,
            parameters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_nodes() {
        // REQ-AST-003: CompactString fits inline up to 24 bytes on stack
        assert_eq!(std::mem::size_of::<IdentifierNode>(), 24);

        let id = IdentifierNode::new("users");
        assert_eq!(id.name.as_str(), "users");
        assert!(!id.name.is_heap_allocated());

        let table = TableNode::with_schema("public", "users");
        assert!(table.schema.is_some());

        let col = ColumnNode::new("id");
        assert_eq!(
            col.column,
            Arc::new(OperationNode::Identifier(IdentifierNode::new("id")))
        );

        let qref = ReferenceNode::with_table("users", "email");
        assert!(qref.table.is_some());

        let val = ValueNode::new(12345);
        assert_eq!(val.value, Value::I64(12345));

        let list = PrimitiveValueListNode::new([1, 2, 3]);
        assert_eq!(list.values.len(), 3);

        let raw = RawNode::new("NOW()");
        assert_eq!(raw.sql_fragments[0].as_str(), "NOW()");
        assert_eq!(raw.parameters.len(), 0);
    }
}
