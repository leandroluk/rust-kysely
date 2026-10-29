# Roadmap

## Milestones & Feature Specifications

### [Milestone 1: Core AST & PostgreSQL Compiler](../features/01-core-ast-compiler/spec.md)
- `OperationNode` hierarchy (enums and structs covering query constructs).
- `OperationNodeVisitor` and `OperationNodeTransformer`.
- `DefaultQueryCompiler` and `PostgresQueryCompiler`.
- Compilation of SQL strings, parameter accumulation (`$1, $2, ...`), and identifier wrapping (`"table"."col"`).
- Specs: `REQ-AST-001` through `REQ-AST-010`.

### [Milestone 2: Expression Builder & Raw SQL (`sql`)](../features/02-expressions-raw-sql/spec.md)
- Macro-free `sql` builder (`sql("SELECT ...")`, `.bind()`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`).
- `Expression` trait and `ExpressionWrapper`.
- `ExpressionBuilder` (`eb`) with boolean logic (`eb.or`, `eb.and`, `eb.not`), comparison helpers, and SQL functions (`eb.fn.count`, `eb.fn.max`, etc.).
- Specs: `REQ-EXPR-001` through `REQ-EXPR-010`.

### [Milestone 3: Select Query Builder](../features/03-select-query-builder/spec.md)
- `SelectQueryBuilder` with method chaining:
  - `select_from`, `inner_join`, `left_join`, `right_join`, `full_join`.
  - `where_`, `where_ref`, `where_expr` (closure `|eb| ...`).
  - `IntoSelections` tuple support: single item, array `["id", "name"]`, tuple `("id", "name", count(...))`, callback `|eb| (...)`.
  - `order_by`, `group_by`, `having`, `limit`, `offset`.
  - `distinct`, `distinct_on`.
  - `with` (CTEs / Common Table Expressions), `union`, `union_all`.
- Specs: `REQ-SEL-001` through `REQ-SEL-010`.

### [Milestone 4: Mutation Query Builders & RETURNING](../features/04-mutation-query-builders/spec.md)
- `InsertQueryBuilder`:
  - `values` (closure builder, batch inserts, and serde structs without `.into()`).
  - `on_conflict`: `do_nothing()`, `do_update_set(...)`.
  - `returning` and `returning_all`.
- `UpdateQueryBuilder`:
  - `set` (columns, expressions, raw SQL).
  - `where_`, `where_ref`, `returning`.
- `DeleteQueryBuilder`:
  - `where_`, `returning`.
- Specs: `REQ-MUT-001` through `REQ-MUT-010`.

### [Milestone 5: PostgreSQL Driver & Execution Engine](../features/05-postgres-driver-execution/spec.md)
- `DatabaseConnection` and `Driver` traits.
- `PostgresDialect` wrapping `tokio-postgres`.
- `Kysely` main client struct.
- Result representation: `Row`, `QueryResult`, typed row extraction (`.get::<T>("col")?`, `.execute_into::<T>()`).
- Transaction support (`db.transaction(|trx| ...)`), savepoints.
- Specs: `REQ-EXEC-001` through `REQ-EXEC-010`.

### [Milestone 6: Schema DDL & Introspection](../features/06-schema-ddl-introspection/spec.md)
- `SchemaModule`:
  - `create_table`, `alter_table`, `drop_table`.
  - `create_index`, `drop_index`.
  - `create_schema`, `drop_schema`.
  - `create_type` (Postgres enums), `create_view`.
- `PostgresIntrospector` querying `information_schema` and `pg_catalog`.
- Specs: `REQ-DDL-001` through `REQ-DDL-010`.

### [Milestone 7: Migrator & Advisory Locks](../features/07-migrator-advisory-locks/spec.md)
- `Migrator` and `MigrationProvider` (file-based and in-memory).
- PostgreSQL advisory lock synchronization (`pg_advisory_lock`, `pg_advisory_unlock`).
- Migration state table creation and tracking (`migrate_to_latest`, `migrate_up`, `migrate_down`).
- Specs: `REQ-MIG-001` through `REQ-MIG-008`.

### [Milestone 8: Plugin Architecture](../features/08-plugin-system/spec.md)
- `KyselyPlugin` trait with `transform_query` and `transform_result`.
- Built-in plugins: `CamelCasePlugin`, `DeduplicateJoinsPlugin`, `WithSchemaPlugin`.
- Specs: `REQ-PLUG-001` through `REQ-PLUG-006`.
