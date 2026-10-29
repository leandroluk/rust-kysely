//! Helper constructors for raw SQL interpolation: raw, id, table, lit, and join.

use compact_str::CompactString;
use std::sync::Arc;

use crate::expression::{Expression, ExpressionWrapper};
use crate::operation_node::{
    IdentifierNode, IntoValue, OperationNode, RawNode, TableNode, ValueNode,
};
use crate::raw_builder::RawBuilder;

/// Inserts raw verbatim SQL without escaping or parameterization.
// @spec REQ-EXPR-003
pub fn raw(sql_str: impl Into<CompactString>) -> ExpressionWrapper<()> {
    ExpressionWrapper::new(Arc::new(OperationNode::Raw(RawNode::new(sql_str))))
}

/// Constructs a sanitized identifier node that will be quoted according to the dialect.
// @spec REQ-EXPR-004
pub fn id(name: impl Into<CompactString>) -> ExpressionWrapper<()> {
    ExpressionWrapper::new(Arc::new(OperationNode::Identifier(IdentifierNode::new(
        name,
    ))))
}

/// Constructs a table reference node.
// @spec REQ-EXPR-004
pub fn table(name: impl Into<CompactString>) -> ExpressionWrapper<()> {
    ExpressionWrapper::new(Arc::new(OperationNode::Table(TableNode::new(name))))
}

/// Constructs an escaped literal parameter node.
// @spec REQ-EXPR-005
pub fn lit(val: impl IntoValue) -> ExpressionWrapper<()> {
    ExpressionWrapper::new(Arc::new(OperationNode::Value(ValueNode::new(
        val.into_value(),
    ))))
}

/// Joins a sequence of expressions using the specified separator.
// @spec REQ-EXPR-005
pub fn join<U>(
    items: impl IntoIterator<Item = impl Expression<U>>,
    separator: impl Into<CompactString>,
) -> RawBuilder<()> {
    let sep = separator.into();
    let mut builder = RawBuilder::new("");
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            builder = builder.append(sep.clone());
        }
        builder = builder.append_expr(item);
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;
    use crate::raw_builder::sql;

    #[test]
    fn test_sql_helpers() {
        let compiler = PostgresQueryCompiler::new();

        // Test sql::id and sql::table with PostgreSQL escaping
        let q = sql("SELECT ")
            .append_expr(id("first_name"))
            .append(" FROM ")
            .append_expr(table("users"))
            .append(" WHERE ")
            .append_expr(id("active"))
            .append(" = ")
            .append_expr(lit(true));

        let compiled = compiler.compile(&q.to_operation_node());
        assert_eq!(
            compiled.sql,
            "SELECT \"first_name\" FROM \"users\" WHERE \"active\" = $1"
        );
        assert_eq!(compiled.parameters, vec![Value::Bool(true)]);

        // Test sql::raw
        let r = sql("SELECT ").append_expr(raw("NOW()"));
        let compiled_r = compiler.compile(&r.to_operation_node());
        assert_eq!(compiled_r.sql, "SELECT NOW()");

        // Test sql::join
        let items = [id("col1"), id("col2"), id("col3")];
        let joined = join(items, ", ");
        let compiled_j = compiler.compile(&joined.to_operation_node());
        assert_eq!(compiled_j.sql, "\"col1\", \"col2\", \"col3\"");
    }
}
