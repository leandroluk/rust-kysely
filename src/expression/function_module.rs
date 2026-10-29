//! SQL aggregate and scalar function factory (`count`, `max`, `min`, `avg`, `sum`, `coalesce`).

use compact_str::CompactString;
use std::sync::Arc;

use crate::expression::expression_builder::parse_column_reference;
use crate::expression::{Expression, ExpressionWrapper};
use crate::operation_node::{FunctionNode, OperationNode, RawNode};

/// Factory module for building standard SQL aggregation and scalar functions.
// @spec REQ-EXPR-010
#[derive(Debug, Clone, Copy, Default)]
pub struct FunctionModule;

impl FunctionModule {
    /// Creates a new FunctionModule instance.
    pub fn new() -> Self {
        Self
    }

    /// Constructs a `COUNT(column)` function call returning `i64`.
    pub fn count(&self, column: impl Into<CompactString>) -> ExpressionWrapper<i64> {
        let col = parse_column_reference(column.into().as_str());
        let func = FunctionNode::new("COUNT", [col]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs a `COUNT(*)` function call returning `i64`.
    pub fn count_all(&self) -> ExpressionWrapper<i64> {
        let star = Arc::new(OperationNode::Raw(RawNode::new("*")));
        let func = FunctionNode::new("COUNT", [star]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs a `MAX(column)` function call.
    pub fn max(&self, column: impl Into<CompactString>) -> ExpressionWrapper<()> {
        let col = parse_column_reference(column.into().as_str());
        let func = FunctionNode::new("MAX", [col]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs a `MIN(column)` function call.
    pub fn min(&self, column: impl Into<CompactString>) -> ExpressionWrapper<()> {
        let col = parse_column_reference(column.into().as_str());
        let func = FunctionNode::new("MIN", [col]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs an `AVG(column)` function call returning `f64`.
    pub fn avg(&self, column: impl Into<CompactString>) -> ExpressionWrapper<f64> {
        let col = parse_column_reference(column.into().as_str());
        let func = FunctionNode::new("AVG", [col]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs a `SUM(column)` function call.
    pub fn sum(&self, column: impl Into<CompactString>) -> ExpressionWrapper<()> {
        let col = parse_column_reference(column.into().as_str());
        let func = FunctionNode::new("SUM", [col]);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }

    /// Constructs a `COALESCE(arg1, arg2, ...)` function call.
    pub fn coalesce<U>(
        &self,
        arguments: impl IntoIterator<Item = impl Expression<U>>,
    ) -> ExpressionWrapper<()> {
        let args = arguments.into_iter().map(|a| a.to_operation_node());
        let func = FunctionNode::new("COALESCE", args);
        ExpressionWrapper::new(Arc::new(OperationNode::Function(func)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::expression::ExpressionBuilder;

    #[test]
    fn test_function_module() {
        let eb = ExpressionBuilder::new();
        let compiler = PostgresQueryCompiler::new();

        // COUNT("id") AS "total"
        let count_expr = eb.fn_().count("id").as_("total");
        let compiled = compiler.compile(&count_expr.to_operation_node());
        assert_eq!(compiled.sql, "COUNT(\"id\") AS \"total\"");

        // COUNT(*)
        let count_all = eb.fn_().count_all();
        let compiled_all = compiler.compile(&count_all.to_operation_node());
        assert_eq!(compiled_all.sql, "COUNT(*)");

        // MAX("u"."score")
        let max_expr = eb.r#fn().max("u.score");
        let compiled_max = compiler.compile(&max_expr.to_operation_node());
        assert_eq!(compiled_max.sql, "MAX(\"u\".\"score\")");

        // COALESCE("a" = $1)
        let coalesce_expr = eb.fn_().coalesce([eb.cmp("a", "=", 1)]);
        let compiled_coalesce = compiler.compile(&coalesce_expr.to_operation_node());
        assert_eq!(compiled_coalesce.sql, "COALESCE(\"a\" = $1)");
    }
}
