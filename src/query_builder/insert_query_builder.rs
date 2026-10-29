//! Fluent, immutable InsertQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::Expression;
use crate::operation_node::{
    ColumnNode, InsertQueryNode, IntoValue, OperationNode, PrimitiveValueListNode, ValuesNode,
};
use crate::parser::IntoTableSource;
use crate::query_compiler::{CompiledQuery, QueryCompiler};

/// Fluent query builder for SQL INSERT statements.
// @spec REQ-MUT-001
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InsertQueryBuilder {
    pub(crate) node: Arc<InsertQueryNode>,
}

impl InsertQueryBuilder {
    /// Creates a new empty INSERT query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder wrapping an existing AST node.
    pub fn from_node(node: Arc<InsertQueryNode>) -> Self {
        Self { node }
    }

    /// Assigns a column value for single-row insert (`.set("name", "Alice")`).
    ///
    /// Accepts any type implementing `IntoValue` without requiring `.into()`.
    // @spec REQ-MUT-002
    pub fn set(mut self, col: impl Into<CompactString>, val: impl IntoValue) -> Self {
        let col_name: CompactString = col.into();
        let val = val.into_value();
        let col_node = Arc::new(OperationNode::Column(ColumnNode::new(col_name)));

        let q = InsertQueryNode::clone_mut(&mut self.node);

        // 1. Update or append columns
        let columns = q.columns.get_or_insert_with(SmallVec::new);
        columns.push(col_node);

        // 2. Update or create single row in values
        if let Some(ref mut values_op) = q.values {
            if let OperationNode::Values(ref mut values_node) = Arc::make_mut(values_op) {
                if let Some(first_row) = values_node.rows.first_mut() {
                    if let OperationNode::PrimitiveValueList(ref mut list) = Arc::make_mut(first_row) {
                        list.values.push(val);
                        return self;
                    }
                }
            }
        }

        let list_node = Arc::new(OperationNode::PrimitiveValueList(PrimitiveValueListNode::new([val])));
        q.values = Some(Arc::new(OperationNode::Values(ValuesNode::new([list_node]))));

        self
    }

    /// Compiles the query using the specified query compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&OperationNode::InsertQuery((*self.node).clone()))
    }
}

impl Expression for InsertQueryBuilder {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::InsertQuery((*self.node).clone()))
    }
}

/// Entry point creating an `InsertQueryBuilder` targeting the specified table.
// @spec REQ-MUT-001
pub fn insert_into(table: impl IntoTableSource) -> InsertQueryBuilder {
    let mut builder = InsertQueryBuilder::new();
    let q = InsertQueryNode::clone_mut(&mut builder.node);
    q.into = Some(table.into_table_source());
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;

    #[test]
    fn test_insert_single_row() {
        let compiler = PostgresQueryCompiler::new();

        let query = insert_into("users")
            .set("first_name", "Alice")
            .set("last_name", "Smith")
            .set("age", 30)
            .set("active", true)
            .compile(&compiler);

        assert_eq!(
            query.sql,
            "INSERT INTO \"users\" (\"first_name\", \"last_name\", \"age\", \"active\") VALUES ($1, $2, $3, $4)"
        );
        assert_eq!(
            query.parameters,
            vec![
                Value::from("Alice"),
                Value::from("Smith"),
                Value::from(30),
                Value::from(true),
            ]
        );
    }
}
