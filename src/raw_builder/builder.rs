//! Fluent RawBuilder implementation for constructing parameterized SQL without macros.

use std::marker::PhantomData;
use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::Expression;
use crate::operation_node::{IntoValue, OperationNode, RawNode, ValueNode};

/// Fluent builder for parameter-safe raw SQL queries and expression fragments.
// @spec REQ-EXPR-002
#[derive(Debug, Clone, PartialEq)]
pub struct RawBuilder<T = ()> {
    sql_fragments: SmallVec<[CompactString; 2]>,
    parameters: SmallVec<[Arc<OperationNode>; 2]>,
    _phantom: PhantomData<T>,
}

impl<T> RawBuilder<T> {
    /// Creates a new RawBuilder initialized with the given SQL text.
    pub fn new(sql: impl Into<CompactString>) -> Self {
        let mut sql_fragments = SmallVec::new();
        sql_fragments.push(sql.into());
        Self {
            sql_fragments,
            parameters: SmallVec::new(),
            _phantom: PhantomData,
        }
    }

    /// Binds a scalar value as a parameter, automatically converting via `IntoValue`.
    // @spec REQ-EXPR-002
    pub fn bind(mut self, value: impl IntoValue) -> Self {
        self.parameters
            .push(Arc::new(OperationNode::Value(ValueNode::new(
                value.into_value(),
            ))));
        // Ensure there is a trailing fragment following this parameter
        self.sql_fragments.push(CompactString::new(""));
        self
    }

    /// Appends a raw SQL string to the query.
    pub fn append(mut self, text: impl Into<CompactString>) -> Self {
        let text = text.into();
        if let Some(last) = self.sql_fragments.last_mut() {
            last.push_str(&text);
        } else {
            self.sql_fragments.push(text);
        }
        self
    }

    /// Appends an AST expression as an interpolated node parameter.
    pub fn append_expr<U>(mut self, expr: impl Expression<U>) -> Self {
        self.parameters.push(expr.to_operation_node());
        self.sql_fragments.push(CompactString::new(""));
        self
    }

    /// Casts the phantom result type `T` to another type `U`.
    pub fn cast<U>(self) -> RawBuilder<U> {
        RawBuilder {
            sql_fragments: self.sql_fragments,
            parameters: self.parameters,
            _phantom: PhantomData,
        }
    }
}

impl<T> Expression<T> for RawBuilder<T> {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::Raw(RawNode {
            sql_fragments: self.sql_fragments.clone(),
            parameters: self.parameters.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::Value;
    use crate::query_compiler::DefaultQueryCompiler;

    #[test]
    fn test_raw_builder() {
        let builder = RawBuilder::<()>::new("SELECT * FROM users WHERE id = ")
            .bind(42)
            .append(" AND active = ")
            .bind(true);

        let node = builder.to_operation_node();
        let compiler = DefaultQueryCompiler::new();
        let query = compiler.compile(&node);

        assert_eq!(
            query.sql,
            "SELECT * FROM users WHERE id = ? AND active = ?"
        );
        assert_eq!(query.parameters, vec![Value::I64(42), Value::Bool(true)]);
    }
}
