//! Table source parser supporting table names, schema qualification, and aliases.

use compact_str::CompactString;
use std::sync::Arc;

use crate::expression::expression_builder::ExpressionBuilder;
use crate::expression::{Expression, ExpressionWrapper};
use crate::operation_node::{AliasNode, IdentifierNode, OperationNode, TableNode};
use crate::raw_builder::RawBuilder;

/// Parses strings such as `"users"`, `"users as u"`, or `"public.users AS u"` into AST nodes.
pub fn parse_table_source(source: &str) -> Arc<OperationNode> {
    let source = source.trim();
    let (table_part, alias_part) = if let Some(idx) = source.to_ascii_lowercase().find(" as ") {
        (source[..idx].trim(), Some(source[idx + 4..].trim()))
    } else {
        (source, None)
    };

    let table_node = if let Some((schema, table)) = table_part.split_once('.') {
        Arc::new(OperationNode::Table(TableNode::with_schema(schema, table)))
    } else {
        Arc::new(OperationNode::Table(TableNode::new(table_part)))
    };

    if let Some(alias) = alias_part {
        Arc::new(OperationNode::Alias(AliasNode::new(
            table_node,
            Arc::new(OperationNode::Identifier(IdentifierNode::new(alias))),
        )))
    } else {
        table_node
    }
}

/// Trait for converting inputs into an AST table source node.
// @spec REQ-SEL-002
pub trait IntoTableSource {
    /// Converts the target into an AST table source node.
    fn into_table_source(self) -> Arc<OperationNode>;
}

impl IntoTableSource for &str {
    fn into_table_source(self) -> Arc<OperationNode> {
        parse_table_source(self)
    }
}

impl IntoTableSource for String {
    fn into_table_source(self) -> Arc<OperationNode> {
        parse_table_source(&self)
    }
}

impl IntoTableSource for CompactString {
    fn into_table_source(self) -> Arc<OperationNode> {
        parse_table_source(self.as_str())
    }
}

impl<T> IntoTableSource for ExpressionWrapper<T> {
    fn into_table_source(self) -> Arc<OperationNode> {
        self.to_operation_node()
    }
}

impl<T> IntoTableSource for RawBuilder<T> {
    fn into_table_source(self) -> Arc<OperationNode> {
        self.to_operation_node()
    }
}

impl<F, E> IntoTableSource for F
where
    F: FnOnce(&ExpressionBuilder) -> E,
    E: Expression<()>,
{
    fn into_table_source(self) -> Arc<OperationNode> {
        let eb = ExpressionBuilder::new();
        (self)(&eb).to_operation_node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_parser() {
        let t1 = parse_table_source("users");
        assert!(matches!(*t1, OperationNode::Table(_)));

        let t2 = parse_table_source("users as u");
        if let OperationNode::Alias(ref a) = *t2 {
            if let OperationNode::Identifier(ref id) = *a.alias {
                assert_eq!(id.name.as_str(), "u");
            } else {
                panic!("Expected Identifier alias");
            }
        } else {
            panic!("Expected Alias node");
        }

        let t3 = parse_table_source("public.users AS u");
        if let OperationNode::Alias(ref a) = *t3 {
            if let OperationNode::Table(ref tbl) = *a.node {
                assert!(tbl.schema.is_some());
            } else {
                panic!("Expected Table node with schema");
            }
        } else {
            panic!("Expected Alias node");
        }
    }
}
