//! ExpressionBuilder (`eb`) passed to closures for fluent boolean combinators and comparisons.

use std::sync::Arc;
use compact_str::CompactString;

use crate::expression::{Expression, ExpressionWrapper};
use crate::operation_node::*;

/// ExpressionBuilder context providing fluent boolean logic, comparisons, and subquery constructors.
// @spec REQ-EXPR-006
#[derive(Debug, Clone, Copy, Default)]
pub struct ExpressionBuilder;

impl ExpressionBuilder {
    /// Creates a new ExpressionBuilder instance.
    pub fn new() -> Self {
        Self
    }

    /// Compares a column reference to a scalar value parameter (`col OP val`).
    // @spec REQ-EXPR-007
    pub fn cmp(
        &self,
        column: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        value: impl IntoValue,
    ) -> ExpressionWrapper<bool> {
        let col_name = column.into();
        let col_node = parse_column_reference(col_name.as_str());
        let val_node = Arc::new(OperationNode::Value(ValueNode::new(value.into_value())));

        let bin = BinaryOperationNode::new(col_node, operator, val_node);
        ExpressionWrapper::new(Arc::new(OperationNode::Binary(bin)))
    }

    /// Compares two column references (`col1 OP col2`).
    // @spec REQ-EXPR-007
    pub fn cmp_ref(
        &self,
        left_column: impl Into<CompactString>,
        operator: impl Into<CompactString>,
        right_column: impl Into<CompactString>,
    ) -> ExpressionWrapper<bool> {
        let left_node = parse_column_reference(left_column.into().as_str());
        let right_node = parse_column_reference(right_column.into().as_str());

        let bin = BinaryOperationNode::new(left_node, operator, right_node);
        ExpressionWrapper::new(Arc::new(OperationNode::Binary(bin)))
    }

    /// Compares two arbitrary AST expressions (`expr1 OP expr2`).
    // @spec REQ-EXPR-007
    pub fn cmp_expr<T1, T2>(
        &self,
        left: impl Expression<T1>,
        operator: impl Into<CompactString>,
        right: impl Expression<T2>,
    ) -> ExpressionWrapper<bool> {
        let bin = BinaryOperationNode::new(left.to_operation_node(), operator, right.to_operation_node());
        ExpressionWrapper::new(Arc::new(OperationNode::Binary(bin)))
    }

    /// Combines multiple expressions with `OR` wrapped in parentheses (`(e1 OR e2 ...)`).
    // @spec REQ-EXPR-006
    pub fn or<T>(
        &self,
        expressions: impl IntoIterator<Item = impl Expression<T>>,
    ) -> ExpressionWrapper<bool> {
        let mut iter = expressions.into_iter();
        let first = match iter.next() {
            Some(e) => e.to_operation_node(),
            None => return ExpressionWrapper::new(Arc::new(OperationNode::Value(ValueNode::new(false)))),
        };

        let mut combined = first;
        for next in iter {
            combined = Arc::new(OperationNode::Or(OrNode::new(combined, next.to_operation_node())));
        }

        ExpressionWrapper::new(Arc::new(OperationNode::Parens(ParensNode::new(combined))))
    }

    /// Combines multiple expressions with `AND` wrapped in parentheses (`(e1 AND e2 ...)`).
    // @spec REQ-EXPR-006
    pub fn and<T>(
        &self,
        expressions: impl IntoIterator<Item = impl Expression<T>>,
    ) -> ExpressionWrapper<bool> {
        let mut iter = expressions.into_iter();
        let first = match iter.next() {
            Some(e) => e.to_operation_node(),
            None => return ExpressionWrapper::new(Arc::new(OperationNode::Value(ValueNode::new(true)))),
        };

        let mut combined = first;
        for next in iter {
            combined = Arc::new(OperationNode::And(AndNode::new(combined, next.to_operation_node())));
        }

        ExpressionWrapper::new(Arc::new(OperationNode::Parens(ParensNode::new(combined))))
    }

    /// Negates an expression with `NOT` (`NOT (expr)`).
    // @spec REQ-EXPR-006
    pub fn not<T>(&self, expression: impl Expression<T>) -> ExpressionWrapper<bool> {
        let inner = Arc::new(OperationNode::Parens(ParensNode::new(
            expression.to_operation_node(),
        )));
        let un = UnaryOperationNode::new("NOT", inner);
        ExpressionWrapper::new(Arc::new(OperationNode::Unary(un)))
    }
}

/// Helper parsing strings like `"users.id"` into table-qualified `ReferenceNode`s, or bare `"id"` into `ColumnNode`s.
pub fn parse_column_reference(name: &str) -> Arc<OperationNode> {
    if let Some((table, col)) = name.split_once('.') {
        Arc::new(OperationNode::Reference(ReferenceNode::with_table(table, col)))
    } else {
        Arc::new(OperationNode::Column(ColumnNode::new(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;

    #[test]
    fn test_eb_boolean() {
        let eb = ExpressionBuilder::new();
        let compiler = PostgresQueryCompiler::new();

        let expr = eb.or([
            eb.cmp("age", ">", 18),
            eb.cmp("first_name", "=", "Bob"),
        ]);

        let compiled = compiler.compile(&expr.to_operation_node());
        assert_eq!(
            compiled.sql,
            "(\"age\" > $1 OR \"first_name\" = $2)"
        );
        assert_eq!(compiled.parameters, vec![Value::I64(18), Value::from("Bob")]);

        let not_expr = eb.not(eb.cmp("active", "=", true));
        let compiled_not = compiler.compile(&not_expr.to_operation_node());
        assert_eq!(compiled_not.sql, "NOT (\"active\" = $1)");
    }

    #[test]
    fn test_eb_cmp() {
        let eb = ExpressionBuilder::new();
        let compiler = PostgresQueryCompiler::new();

        // Qualified reference comparison
        let cmp_ref = eb.cmp_ref("u.org_id", "=", "o.id");
        let compiled_ref = compiler.compile(&cmp_ref.to_operation_node());
        assert_eq!(compiled_ref.sql, "\"u\".\"org_id\" = \"o\".\"id\"");
        assert!(compiled_ref.parameters.is_empty());
    }
}
