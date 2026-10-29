//! PostgreSQL-specific query compiler implementation.

use crate::operation_node::OperationNode;
use crate::query_compiler::{
    CompiledQuery, DefaultQueryCompiler, IdentifierQuote, ParameterStyle, QueryCompiler,
};

/// Query compiler specialized for the PostgreSQL dialect.
///
/// Features:
/// - Sequential parameter placeholders: `$1`, `$2`, `$3`, ...
/// - Identifier quoting with double quotes: `"column_name"`, `"user""table"`
/// - Full support for `RETURNING`, CTEs (`WITH`), and `ON CONFLICT`
// @spec REQ-AST-009
#[derive(Debug, Clone)]
pub struct PostgresQueryCompiler {
    inner: DefaultQueryCompiler,
}

impl Default for PostgresQueryCompiler {
    fn default() -> Self {
        Self {
            inner: DefaultQueryCompiler::with_styles(
                ParameterStyle::Dollar,
                IdentifierQuote::DoubleQuote,
            ),
        }
    }
}

impl PostgresQueryCompiler {
    /// Creates a new PostgreSQL query compiler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compiles an AST node into a PostgreSQL-compatible `CompiledQuery`.
    pub fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.inner.compile(node)
    }
}

impl QueryCompiler for PostgresQueryCompiler {
    fn compile(&self, node: &OperationNode) -> CompiledQuery {
        self.compile(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::*;
    use std::sync::Arc;

    #[test]
    fn test_postgres_compiler() {
        let compiler = PostgresQueryCompiler::new();

        // 1. SELECT "u"."id", "u"."name" FROM "users" AS "u" WHERE "u"."age" >= $1 AND "u"."status" = $2 ORDER BY "u"."created_at" DESC LIMIT $3 OFFSET $4
        let mut select = SelectQueryNode::new();
        select
            .selections
            .push(Arc::new(OperationNode::Selection(SelectionNode::new(
                Arc::new(OperationNode::Reference(ReferenceNode::with_table(
                    "u", "id",
                ))),
            ))));
        select
            .selections
            .push(Arc::new(OperationNode::Selection(SelectionNode::new(
                Arc::new(OperationNode::Reference(ReferenceNode::with_table(
                    "u", "name",
                ))),
            ))));
        select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
            OperationNode::Alias(AliasNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))),
                Arc::new(OperationNode::Identifier(IdentifierNode::new("u"))),
            )),
        )))));

        let age_cmp = Arc::new(OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Reference(ReferenceNode::with_table(
                "u", "age",
            ))),
            ">=",
            Arc::new(OperationNode::Value(ValueNode::new(18))),
        )));
        let status_cmp = Arc::new(OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Reference(ReferenceNode::with_table(
                "u", "status",
            ))),
            "=",
            Arc::new(OperationNode::Value(ValueNode::new("active"))),
        )));
        select.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
            OperationNode::And(AndNode::new(age_cmp, status_cmp)),
        )))));

        select.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new([
            Arc::new(OperationNode::OrderByItem(OrderByItemNode::new(
                Arc::new(OperationNode::Reference(ReferenceNode::with_table(
                    "u",
                    "created_at",
                ))),
                OrderDirection::Desc,
            ))),
        ]))));

        select.limit = Some(Arc::new(OperationNode::Limit(LimitNode::new(Arc::new(
            OperationNode::Value(ValueNode::new(50)),
        )))));
        select.offset = Some(Arc::new(OperationNode::Offset(OffsetNode::new(Arc::new(
            OperationNode::Value(ValueNode::new(100)),
        )))));

        let query = compiler.compile(&OperationNode::SelectQuery(select));
        assert_eq!(
            query.sql,
            "SELECT \"u\".\"id\", \"u\".\"name\" FROM \"users\" AS \"u\" WHERE \"u\".\"age\" >= $1 AND \"u\".\"status\" = $2 ORDER BY \"u\".\"created_at\" DESC LIMIT $3 OFFSET $4"
        );
        assert_eq!(
            query.parameters,
            vec![
                Value::I64(18),
                Value::from("active"),
                Value::I64(50),
                Value::I64(100),
            ]
        );

        // 2. INSERT INTO "users" ("name", "email") VALUES ($1, $2) RETURNING "id"
        let mut insert = InsertQueryNode::new();
        insert.into = Some(Arc::new(OperationNode::Table(TableNode::new("users"))));
        insert.columns = Some(smallvec::smallvec![
            Arc::new(OperationNode::Column(ColumnNode::new("name"))),
            Arc::new(OperationNode::Column(ColumnNode::new("email"))),
        ]);
        insert.values = Some(Arc::new(OperationNode::Values(ValuesNode::new([
            Arc::new(OperationNode::PrimitiveValueList(
                PrimitiveValueListNode::new([
                    Value::from("Alice"),
                    Value::from("alice@example.com"),
                ]),
            )),
        ]))));
        insert.returning = Some(smallvec::smallvec![Arc::new(OperationNode::Selection(
            SelectionNode::new(Arc::new(OperationNode::Column(ColumnNode::new("id")))),
        ))]);

        let insert_query = compiler.compile(&OperationNode::InsertQuery(insert));
        assert_eq!(
            insert_query.sql,
            "INSERT INTO \"users\" (\"name\", \"email\") VALUES ($1, $2) RETURNING \"id\""
        );
        assert_eq!(
            insert_query.parameters,
            vec![Value::from("Alice"), Value::from("alice@example.com")]
        );

        // 3. Identifier quotes escaping
        let quoted_id = IdentifierNode::new("table\"with\"quotes");
        let quoted_sql = compiler.inner.format_identifier(quoted_id.name.as_str());
        assert_eq!(quoted_sql, "\"table\"\"with\"\"quotes\"");
    }
}
