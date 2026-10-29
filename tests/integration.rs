use rust_kysely::dialect::PostgresQueryCompiler;
use rust_kysely::operation_node::*;
use rust_kysely::query_compiler::DefaultQueryCompiler;
use rust_kysely::VERSION;
use std::sync::Arc;

#[test]
fn test_version_present() {
    assert!(!VERSION.is_empty());
}

#[test]
fn test_integration_postgres_select_complex() {
    let compiler = PostgresQueryCompiler::new();

    // Query:
    // WITH "active_users" AS (SELECT "id", "org_id" FROM "users" WHERE "active" = $1)
    // SELECT "u"."id", "o"."name" AS "org_name"
    // FROM "active_users" AS "u"
    // INNER JOIN "organizations" AS "o" ON "o"."id" = "u"."org_id"
    // WHERE "u"."org_id" > $2
    // ORDER BY "u"."id" ASC
    // LIMIT $3 OFFSET $4

    let mut cte_query = SelectQueryNode::new();
    cte_query.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
        Arc::new(OperationNode::Column(ColumnNode::new("id"))),
    ))));
    cte_query.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
        Arc::new(OperationNode::Column(ColumnNode::new("org_id"))),
    ))));
    cte_query.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
        OperationNode::Table(TableNode::new("users")),
    )))));
    cte_query.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
        OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("active"))),
            "=",
            Arc::new(OperationNode::Value(ValueNode::new(true))),
        )),
    )))));

    let cte = Arc::new(OperationNode::CommonTableExpression(
        CommonTableExpressionNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("active_users"))),
            Arc::new(OperationNode::SelectQuery(cte_query)),
        ),
    ));

    let mut select = SelectQueryNode::new();
    select.with = Some(Arc::new(OperationNode::With(WithNode::new(false, [cte]))));

    select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
        Arc::new(OperationNode::Reference(ReferenceNode::with_table("u", "id"))),
    ))));
    select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::with_alias(
        Arc::new(OperationNode::Reference(ReferenceNode::with_table("o", "name"))),
        Arc::new(OperationNode::Identifier(IdentifierNode::new("org_name"))),
    ))));

    select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
        OperationNode::Alias(AliasNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("active_users"))),
            Arc::new(OperationNode::Identifier(IdentifierNode::new("u"))),
        )),
    )))));

    select.joins.push(Arc::new(OperationNode::Join(JoinNode::with_on(
        JoinType::Inner,
        Arc::new(OperationNode::Alias(AliasNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("organizations"))),
            Arc::new(OperationNode::Identifier(IdentifierNode::new("o"))),
        ))),
        Arc::new(OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Reference(ReferenceNode::with_table("o", "id"))),
            "=",
            Arc::new(OperationNode::Reference(ReferenceNode::with_table("u", "org_id"))),
        ))),
    ))));

    select.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
        OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Reference(ReferenceNode::with_table("u", "org_id"))),
            ">",
            Arc::new(OperationNode::Value(ValueNode::new(10))),
        )),
    )))));

    select.order_by = Some(Arc::new(OperationNode::OrderBy(OrderByNode::new([Arc::new(
        OperationNode::OrderByItem(OrderByItemNode::new(
            Arc::new(OperationNode::Reference(ReferenceNode::with_table("u", "id"))),
            OrderDirection::Asc,
        )),
    )]))));

    select.limit = Some(Arc::new(OperationNode::Limit(LimitNode::new(Arc::new(
        OperationNode::Value(ValueNode::new(25)),
    )))));
    select.offset = Some(Arc::new(OperationNode::Offset(OffsetNode::new(Arc::new(
        OperationNode::Value(ValueNode::new(50)),
    )))));

    let query = compiler.compile(&OperationNode::SelectQuery(select));

    assert_eq!(
        query.sql,
        "WITH \"active_users\" AS (SELECT \"id\", \"org_id\" FROM \"users\" WHERE \"active\" = $1) SELECT \"u\".\"id\", \"o\".\"name\" AS \"org_name\" FROM \"active_users\" AS \"u\" INNER JOIN \"organizations\" AS \"o\" ON \"o\".\"id\" = \"u\".\"org_id\" WHERE \"u\".\"org_id\" > $2 ORDER BY \"u\".\"id\" ASC LIMIT $3 OFFSET $4"
    );
    assert_eq!(
        query.parameters,
        vec![Value::Bool(true), Value::I64(10), Value::I64(25), Value::I64(50)]
    );
    assert!(query.query_id.raw() > 0);
}

#[test]
fn test_integration_postgres_insert_on_conflict_returning() {
    let compiler = PostgresQueryCompiler::new();

    // INSERT INTO "users" ("email", "name") VALUES ($1, $2) ON CONFLICT ("email") DO UPDATE SET "name" = $3 WHERE "users"."archived" = $4 RETURNING "id", "created_at"
    let mut insert = InsertQueryNode::new();
    insert.into = Some(Arc::new(OperationNode::Table(TableNode::new("users"))));
    insert.columns = Some(smallvec::smallvec![
        Arc::new(OperationNode::Column(ColumnNode::new("email"))),
        Arc::new(OperationNode::Column(ColumnNode::new("name"))),
    ]);
    insert.values = Some(Arc::new(OperationNode::Values(ValuesNode::new([
        Arc::new(OperationNode::PrimitiveValueList(PrimitiveValueListNode::new([
            Value::from("bob@example.com"),
            Value::from("Bob"),
        ]))),
    ]))));

    let conflict_cols = smallvec::smallvec![Arc::new(OperationNode::Column(ColumnNode::new(
        "email"
    )))];
    let conflict_updates = smallvec::smallvec![Arc::new(OperationNode::ColumnUpdate(
        ColumnUpdateNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("name"))),
            Arc::new(OperationNode::Value(ValueNode::new("Bob (Updated)"))),
        )
    ))];
    let conflict_where = Some(Arc::new(OperationNode::Binary(BinaryOperationNode::new(
        Arc::new(OperationNode::Reference(ReferenceNode::with_table("users", "archived"))),
        "=",
        Arc::new(OperationNode::Value(ValueNode::new(false))),
    ))));

    insert.on_conflict = Some(Arc::new(OperationNode::OnConflict(OnConflictNode::do_update(
        Some(conflict_cols),
        conflict_updates,
        conflict_where,
    ))));

    insert.returning = Some(smallvec::smallvec![
        Arc::new(OperationNode::Selection(SelectionNode::new(Arc::new(
            OperationNode::Column(ColumnNode::new("id"))
        )))),
        Arc::new(OperationNode::Selection(SelectionNode::new(Arc::new(
            OperationNode::Column(ColumnNode::new("created_at"))
        )))),
    ]);

    let query = compiler.compile(&OperationNode::InsertQuery(insert));
    assert_eq!(
        query.sql,
        "INSERT INTO \"users\" (\"email\", \"name\") VALUES ($1, $2) ON CONFLICT (\"email\") DO UPDATE SET \"name\" = $3 WHERE \"users\".\"archived\" = $4 RETURNING \"id\", \"created_at\""
    );
    assert_eq!(
        query.parameters,
        vec![
            Value::from("bob@example.com"),
            Value::from("Bob"),
            Value::from("Bob (Updated)"),
            Value::Bool(false),
        ]
    );
}

#[test]
fn test_integration_postgres_update_and_delete() {
    let compiler = PostgresQueryCompiler::new();

    // UPDATE "accounts" SET "balance" = $1, "updated_at" = NOW() WHERE "id" = $2 RETURNING "balance"
    let mut update = UpdateQueryNode::new();
    update.table = Some(Arc::new(OperationNode::Table(TableNode::new("accounts"))));
    update.updates.push(Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
        Arc::new(OperationNode::Column(ColumnNode::new("balance"))),
        Arc::new(OperationNode::Value(ValueNode::new(1500))),
    ))));
    update.updates.push(Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
        Arc::new(OperationNode::Column(ColumnNode::new("updated_at"))),
        Arc::new(OperationNode::Raw(RawNode::new("NOW()"))),
    ))));
    update.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
        OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("id"))),
            "=",
            Arc::new(OperationNode::Value(ValueNode::new(42))),
        )),
    )))));
    update.returning = Some(smallvec::smallvec![Arc::new(OperationNode::Selection(
        SelectionNode::new(Arc::new(OperationNode::Column(ColumnNode::new("balance")))),
    ))]);

    let update_q = compiler.compile(&OperationNode::UpdateQuery(update));
    assert_eq!(
        update_q.sql,
        "UPDATE \"accounts\" SET \"balance\" = $1, \"updated_at\" = NOW() WHERE \"id\" = $2 RETURNING \"balance\""
    );
    assert_eq!(update_q.parameters, vec![Value::I64(1500), Value::I64(42)]);

    // DELETE FROM "sessions" WHERE "expires_at" < NOW() RETURNING "id"
    let mut delete = DeleteQueryNode::new();
    delete.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
        OperationNode::Table(TableNode::new("sessions")),
    )))));
    delete.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
        OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("expires_at"))),
            "<",
            Arc::new(OperationNode::Raw(RawNode::new("NOW()"))),
        )),
    )))));
    delete.returning = Some(smallvec::smallvec![Arc::new(OperationNode::Selection(
        SelectionNode::new(Arc::new(OperationNode::Column(ColumnNode::new("id")))),
    ))]);

    let delete_q = compiler.compile(&OperationNode::DeleteQuery(delete));
    assert_eq!(
        delete_q.sql,
        "DELETE FROM \"sessions\" WHERE \"expires_at\" < NOW() RETURNING \"id\""
    );
    assert!(delete_q.parameters.is_empty());
}

#[test]
fn test_integration_default_ansi_compiler() {
    let compiler = DefaultQueryCompiler::new();

    let mut select = SelectQueryNode::new();
    select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
        Arc::new(OperationNode::Column(ColumnNode::new("total"))),
    ))));
    select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
        OperationNode::Table(TableNode::new("orders")),
    )))));
    select.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
        OperationNode::Binary(BinaryOperationNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("status"))),
            "IN",
            Arc::new(OperationNode::PrimitiveValueList(PrimitiveValueListNode::new([
                Value::from("completed"),
                Value::from("shipped"),
            ]))),
        )),
    )))));

    let q = compiler.compile(&OperationNode::SelectQuery(select));
    assert_eq!(
        q.sql,
        "SELECT \"total\" FROM \"orders\" WHERE \"status\" IN (?, ?)"
    );
    assert_eq!(
        q.parameters,
        vec![Value::from("completed"), Value::from("shipped")]
    );
}
