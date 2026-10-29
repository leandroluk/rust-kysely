use rust_kysely::dialect::PostgresQueryCompiler;
use rust_kysely::expression::Expression;
use rust_kysely::operation_node::*;
use rust_kysely::query_compiler::DefaultQueryCompiler;
use rust_kysely::{
    delete_from, insert_into, select_from, update_table, sql, ExpressionBuilder, OrderNulls,
    VERSION,
};
use std::sync::Arc;

#[test]
fn test_version_present() {
    assert!(!VERSION.is_empty());
}

#[test]
fn test_integration_postgres_select_complex() {
    let compiler = PostgresQueryCompiler::new();

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

#[test]
fn test_integration_eb_and_raw_builder_combined() {
    let eb = ExpressionBuilder::new();
    let compiler = PostgresQueryCompiler::new();

    // 1. Combining eb.or, eb.and, eb.cmp, eb.between, and eb.fn_
    let complex_filter = eb.and([
        eb.or([
            eb.cmp("u.status", "=", "active"),
            eb.cmp("u.role", "=", "admin"),
        ]),
        eb.between("u.age", 21, 55),
        eb.not(eb.cmp("u.deleted", "=", true)),
    ]);

    let mut select = SelectQueryNode::new();
    select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
        Arc::new(OperationNode::Reference(ReferenceNode::with_table("u", "id"))),
    ))));
    select.selections.push(
        eb.fn_().count("o.id").as_("order_count").to_operation_node(),
    );
    select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
        OperationNode::Alias(AliasNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))),
            Arc::new(OperationNode::Identifier(IdentifierNode::new("u"))),
        )),
    )))));
    select.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(
        complex_filter.to_operation_node(),
    ))));

    let compiled = compiler.compile(&OperationNode::SelectQuery(select));
    assert_eq!(
        compiled.sql,
        "SELECT \"u\".\"id\", COUNT(\"o\".\"id\") AS \"order_count\" FROM \"users\" AS \"u\" WHERE ((\"u\".\"status\" = $1 OR \"u\".\"role\" = $2) AND \"u\".\"age\" BETWEEN $3 AND $4 AND NOT (\"u\".\"deleted\" = $5))"
    );
    assert_eq!(
        compiled.parameters,
        vec![
            Value::from("active"),
            Value::from("admin"),
            Value::I64(21),
            Value::I64(55),
            Value::Bool(true),
        ]
    );

    // 2. Raw SQL builder with sql::* helpers
    let raw_query = sql("SELECT ")
        .append_expr(sql::join(
            [sql::id("first_name"), sql::id("last_name"), sql::id("email")],
            ", ",
        ))
        .append(" FROM ")
        .append_expr(sql::table("employees"))
        .append(" WHERE ")
        .append_expr(sql::id("salary"))
        .append(" >= ")
        .append_expr(sql::lit(75000))
        .append(" AND ")
        .append_expr(sql::id("created_at"))
        .append(" < ")
        .append_expr(sql::raw("NOW()"));

    let compiled_raw = compiler.compile(&raw_query.to_operation_node());
    assert_eq!(
        compiled_raw.sql,
        "SELECT \"first_name\", \"last_name\", \"email\" FROM \"employees\" WHERE \"salary\" >= $1 AND \"created_at\" < NOW()"
    );
    assert_eq!(compiled_raw.parameters, vec![Value::I64(75000)]);
}

#[test]
fn test_integration_select_builder_full_fluent_pipeline() {
    let compiler = PostgresQueryCompiler::new();
    let eb = ExpressionBuilder::new();

    let cte = select_from("users")
        .select(["id", "org_id"])
        .where_("active", "=", true);

    let query = select_from("active_users as u")
        .with("active_users", cte)
        .select((
            "u.id",
            "u.name",
            eb.fn_().count("u.id").as_("total_users"),
        ))
        .inner_join_ref("organizations as o", "o.id", "u.org_id")
        .left_join("orders as ord", |jb| {
            jb.on_ref("ord.user_id", "=", "u.id")
              .on("ord.status", "=", "completed")
        })
        .where_("u.org_id", ">", 10)
        .where_expr(|eb| {
            eb.or([
                eb.cmp("u.role", "=", "admin"),
                eb.cmp("u.role", "=", "manager"),
            ])
        })
        .group_by(["u.id", "u.name", "o.name"])
        .having_expr(|eb| eb.cmp_expr(eb.fn_().count("u.id"), ">", sql::lit(1)))
        .order_by_nulls("u.id", "asc", OrderNulls::First)
        .limit(25)
        .offset(50)
        .compile(&compiler);

    assert_eq!(
        query.sql,
        "WITH \"active_users\" AS (SELECT \"id\", \"org_id\" FROM \"users\" WHERE \"active\" = $1) SELECT \"u\".\"id\", \"u\".\"name\", COUNT(\"u\".\"id\") AS \"total_users\" FROM \"active_users\" AS \"u\" INNER JOIN \"organizations\" AS \"o\" ON \"o\".\"id\" = \"u\".\"org_id\" LEFT JOIN \"orders\" AS \"ord\" ON \"ord\".\"user_id\" = \"u\".\"id\" AND \"ord\".\"status\" = $2 WHERE \"u\".\"org_id\" > $3 AND (\"u\".\"role\" = $4 OR \"u\".\"role\" = $5) GROUP BY \"u\".\"id\", \"u\".\"name\", \"o\".\"name\" HAVING COUNT(\"u\".\"id\") > $6 ORDER BY \"u\".\"id\" ASC NULLS FIRST LIMIT $7 OFFSET $8"
    );
    assert_eq!(
        query.parameters,
        vec![
            Value::from(true),
            Value::from("completed"),
            Value::from(10),
            Value::from("admin"),
            Value::from("manager"),
            Value::from(1),
            Value::from(25),
            Value::from(50),
        ]
    );
}

#[test]
fn test_integration_select_builder_set_operations() {
    let compiler = PostgresQueryCompiler::new();

    let q1 = select_from("users").select(["id", "email"]).where_("role", "=", "admin");
    let q2 = select_from("users").select(["id", "email"]).where_("role", "=", "moderator");
    let q3 = select_from("users").select(["id", "email"]).where_("active", "=", false);

    let query = q1.union_all(q2).except(q3).compile(&compiler);
    assert_eq!(
        query.sql,
        "SELECT \"id\", \"email\" FROM \"users\" WHERE \"role\" = $1 UNION ALL SELECT \"id\", \"email\" FROM \"users\" WHERE \"role\" = $2 EXCEPT SELECT \"id\", \"email\" FROM \"users\" WHERE \"active\" = $3"
    );
    assert_eq!(
        query.parameters,
        vec![
            Value::from("admin"),
            Value::from("moderator"),
            Value::from(false),
        ]
    );
}

#[test]
fn test_integration_insert_builder_with_upsert_and_returning() {
    let compiler = PostgresQueryCompiler::new();

    let query = insert_into("users")
        .set("id", 100)
        .set("name", "Alice")
        .set("email", "alice@corp.com")
        .set("active", true)
        .on_conflict(|oc| {
            oc.column("id")
              .do_update_set(|ub| {
                  ub.set_ref("name", "excluded.name")
                    .set_ref("email", "excluded.email")
                    .set("updated_at", "NOW()")
                    .where_("users.active", "=", true)
              })
        })
        .returning(["id", "name", "email"])
        .compile(&compiler);

    assert_eq!(
        query.sql,
        "INSERT INTO \"users\" (\"id\", \"name\", \"email\", \"active\") VALUES ($1, $2, $3, $4) ON CONFLICT (\"id\") DO UPDATE SET \"name\" = \"excluded\".\"name\", \"email\" = \"excluded\".\"email\", \"updated_at\" = $5 WHERE \"users\".\"active\" = $6 RETURNING \"id\", \"name\", \"email\""
    );
    assert_eq!(
        query.parameters,
        vec![
            Value::from(100),
            Value::from("Alice"),
            Value::from("alice@corp.com"),
            Value::from(true),
            Value::from("NOW()"),
            Value::from(true),
        ]
    );
}

#[test]
fn test_integration_update_builder_with_from_joins_and_returning() {
    let compiler = PostgresQueryCompiler::new();

    let query = update_table("employees as e")
        .set("salary", 95000)
        .set_ref("department_name", "d.name")
        .from("departments as d")
        .inner_join_ref("companies as c", "c.id", "d.company_id")
        .where_ref("e.department_id", "=", "d.id")
        .where_("c.name", "=", "Acme Corp")
        .returning(["e.id", "e.salary"])
        .compile(&compiler);

    assert_eq!(
        query.sql,
        "UPDATE \"employees\" AS \"e\" SET \"salary\" = $1, \"department_name\" = \"d\".\"name\" FROM \"departments\" AS \"d\" INNER JOIN \"companies\" AS \"c\" ON \"c\".\"id\" = \"d\".\"company_id\" WHERE \"e\".\"department_id\" = \"d\".\"id\" AND \"c\".\"name\" = $2 RETURNING \"e\".\"id\", \"e\".\"salary\""
    );
    assert_eq!(
        query.parameters,
        vec![
            Value::from(95000),
            Value::from("Acme Corp"),
        ]
    );
}

#[test]
fn test_integration_delete_builder_with_using_and_returning() {
    let compiler = PostgresQueryCompiler::new();

    let query = delete_from("sessions as s")
        .using("users as u")
        .where_ref("s.user_id", "=", "u.id")
        .where_("u.active", "=", false)
        .returning_all()
        .compile(&compiler);

    assert_eq!(
        query.sql,
        "DELETE FROM \"sessions\" AS \"s\" USING \"users\" AS \"u\" WHERE \"s\".\"user_id\" = \"u\".\"id\" AND \"u\".\"active\" = $1 RETURNING *"
    );
    assert_eq!(query.parameters, vec![Value::from(false)]);
}

#[tokio::test]
async fn test_integration_kysely_query_execution_full_cycle() {
    use async_trait::async_trait;
    use rust_kysely::{
        CustomDialect, DatabaseConnection, Driver, DriverError, Kysely, QueryResult, Row,
    };
    use std::sync::Mutex;

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
    struct OrderRecord {
        id: i64,
        customer_name: String,
        total: f64,
        notes: Option<String>,
    }

    struct MockConnection {
        executed: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for MockConnection {
        async fn execute_query(&mut self, query: &rust_kysely::CompiledQuery) -> Result<QueryResult, DriverError> {
            self.executed.lock().unwrap().push(query.sql.clone());

            let rows = vec![
                Row::new(
                    vec!["id", "customer_name", "total", "notes"],
                    vec![
                        Value::from(101),
                        Value::from("Acme Corp"),
                        Value::from(1540.50),
                        Value::Null,
                    ],
                ),
                Row::new(
                    vec!["id", "customer_name", "total", "notes"],
                    vec![
                        Value::from(102),
                        Value::from("Globex"),
                        Value::from(320.00),
                        Value::from("Urgent delivery"),
                    ],
                ),
            ];
            Ok(QueryResult::with_rows(rows))
        }
    }

    struct MockDriver {
        executed: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for MockDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockConnection {
                executed: Arc::clone(&self.executed),
            }))
        }

        async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    let executed = Arc::new(Mutex::new(Vec::new()));
    let driver = Arc::new(MockDriver {
        executed: Arc::clone(&executed),
    });
    let compiler = Arc::new(DefaultQueryCompiler::new());
    let dialect = CustomDialect::new(driver, compiler);
    let db = Kysely::new(dialect);

    // 1. SELECT query execution with execute_into
    let orders: Vec<OrderRecord> = db
        .select_from("orders as o")
        .select(["o.id", "o.customer_name", "o.total", "o.notes"])
        .where_("o.total", ">", 100.0)
        .order_by("o.id", "asc")
        .execute_into()
        .await
        .expect("Query failed");

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].id, 101);
    assert_eq!(orders[0].customer_name, "Acme Corp");
    assert_eq!(orders[0].notes, None);
    assert_eq!(orders[1].id, 102);
    assert_eq!(orders[1].notes, Some("Urgent delivery".into()));

    // 2. SELECT execute_take_first
    let first_row = db
        .select_from("orders")
        .select(["id", "total"])
        .execute_take_first()
        .await
        .expect("take_first failed")
        .expect("row should exist");

    assert_eq!(first_row.get::<i64>("id").unwrap(), 101);
    assert_eq!(first_row.get::<f64>("total").unwrap(), 1540.50);

    // 3. Transactions with commit and rollback
    executed.lock().unwrap().clear();
    let tx_res = db
        .transaction()
        .isolation_level(rust_kysely::IsolationLevel::RepeatableRead)
        .execute(|trx| async move {
            trx.insert_into("orders")
                .set("customer_name", "Initech")
                .set("total", 999.0)
                .execute()
                .await?;

            trx.savepoint_block("sp1", |sub| async move {
                sub.update_table("orders")
                    .set("total", 1000.0)
                    .where_("id", "=", 101)
                    .execute()
                    .await?;
                Ok(())
            })
            .await?;

            Ok("success")
        })
        .await;

    assert_eq!(tx_res.unwrap(), "success");

    let history = executed.lock().unwrap().clone();
    assert_eq!(history[0], "BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ");
    assert!(history[1].starts_with("INSERT INTO"));
    assert_eq!(history[2], "SAVEPOINT \"sp1\"");
    assert!(history[3].starts_with("UPDATE"));
    assert_eq!(history[4], "RELEASE SAVEPOINT \"sp1\"");
    assert_eq!(history[5], "COMMIT");
}

#[tokio::test]
async fn test_integration_schema_ddl_full_lifecycle() {
    use async_trait::async_trait;
    use rust_kysely::{
        CustomDialect, DatabaseConnection, DefaultQueryCompiler, Driver, DriverError,
        Kysely, OnModifyForeignAction, QueryResult,
    };
    use std::sync::Mutex;

    struct DdlRecordingConnection {
        executed: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for DdlRecordingConnection {
        async fn execute_query(&mut self, query: &rust_kysely::CompiledQuery) -> Result<QueryResult, DriverError> {
            self.executed.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct DdlRecordingDriver {
        executed: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for DdlRecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(DdlRecordingConnection {
                executed: Arc::clone(&self.executed),
            }))
        }

        async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    let executed = Arc::new(Mutex::new(Vec::new()));
    let driver = Arc::new(DdlRecordingDriver {
        executed: Arc::clone(&executed),
    });
    let compiler = Arc::new(DefaultQueryCompiler::new());
    let dialect = CustomDialect::new(driver, compiler);
    let db = Kysely::new(dialect);

    // 1. CREATE TYPE
    db.schema()
        .create_type("user_role")
        .as_enum(["admin", "user", "guest"])
        .execute()
        .await
        .unwrap();

    // 2. CREATE TABLE tenants
    db.schema()
        .create_table("tenants")
        .if_not_exists()
        .add_column("id", "serial", |c| c.primary_key())
        .add_column("name", "varchar(100)", |c| c.not_null())
        .execute()
        .await
        .unwrap();

    // 3. CREATE TABLE users with foreign key
    db.schema()
        .create_table("users")
        .if_not_exists()
        .add_column("id", "serial", |c| c.primary_key())
        .add_column("tenant_id", "int4", |c| {
            c.references("tenants.id").on_delete(OnModifyForeignAction::Cascade)
        })
        .add_column("email", "varchar(255)", |c| c.not_null().unique())
        .add_column("role", "user_role", |c| c.default_to("user"))
        .execute()
        .await
        .unwrap();

    // 4. CREATE INDEX
    db.schema()
        .create_index("idx_users_tenant_email")
        .on("users")
        .columns(["tenant_id", "email"])
        .unique()
        .using("btree")
        .execute()
        .await
        .unwrap();

    // 5. ALTER TABLE
    db.schema()
        .alter_table("users")
        .add_column("last_login", "timestamp", |c| c.not_null())
        .execute()
        .await
        .unwrap();

    // 6. CREATE VIEW
    db.schema()
        .create_view("active_tenants")
        .as_query(select_from("tenants").select(["id", "name"]))
        .execute()
        .await
        .unwrap();

    // 7. DROP VIEW
    db.schema().drop_view("active_tenants").if_exists().execute().await.unwrap();

    // 8. DROP TABLE
    db.schema().drop_table("users").if_exists().cascade().execute().await.unwrap();

    // 9. DROP TYPE
    db.schema().drop_type("user_role").if_exists().cascade().execute().await.unwrap();

    let history = executed.lock().unwrap().clone();
    assert_eq!(history.len(), 9);
    assert_eq!(history[0], "CREATE TYPE \"user_role\" AS ENUM ('admin', 'user', 'guest')");
    assert_eq!(history[1], "CREATE TABLE IF NOT EXISTS \"tenants\" (\"id\" serial PRIMARY KEY, \"name\" varchar(100) NOT NULL)");
    assert_eq!(history[2], "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" serial PRIMARY KEY, \"tenant_id\" int4 REFERENCES \"tenants\" (\"id\") ON DELETE CASCADE, \"email\" varchar(255) NOT NULL UNIQUE, \"role\" user_role DEFAULT ?)");
    assert_eq!(history[3], "CREATE UNIQUE INDEX \"idx_users_tenant_email\" ON \"users\" USING btree (\"tenant_id\", \"email\")");
    assert_eq!(history[4], "ALTER TABLE \"users\" ADD COLUMN \"last_login\" timestamp NOT NULL");
    assert_eq!(history[5], "CREATE VIEW \"active_tenants\" AS SELECT \"id\", \"name\" FROM \"tenants\"");
    assert_eq!(history[6], "DROP VIEW IF EXISTS \"active_tenants\"");
    assert_eq!(history[7], "DROP TABLE IF EXISTS \"users\" CASCADE");
    assert_eq!(history[8], "DROP TYPE IF EXISTS \"user_role\" CASCADE");
}
