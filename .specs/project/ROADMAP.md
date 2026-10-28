# Roadmap

## Milestones

### Milestone 1: Core AST & PostgreSQL Compiler
- `OperationNode` hierarchy (enums and structs covering query constructs).
- `OperationNodeVisitor` and `OperationNodeTransformer`.
- `DefaultQueryCompiler` and `PostgresQueryCompiler`.
- Compilation of SQL strings, parameter accumulation (`$1, $2, ...`), and identifier wrapping (`"table"."col"`).

### Milestone 2: Expression Builder & Raw SQL (`sql`)
- Macro-free `sql` builder (`sql("SELECT ...")`, `.bind()`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`).
- `Expression` trait and `ExpressionWrapper`.
- `ExpressionBuilder` (`eb`) with boolean logic (`eb.or`, `eb.and`, `eb.not`), comparison helpers, and SQL functions (`eb.fn.count`, `eb.fn.max`, etc.).

### Milestone 3: Select Query Builder
- `SelectQueryBuilder` with method chaining:
  - `select_from`, `inner_join`, `left_join`, `right_join`, `full_join`.
  - `where_`, `where_ref`, `where_expr` (closure `|eb| ...`).
  - `order_by`, `group_by`, `having`, `limit`, `offset`.
  - `distinct`, `distinct_on`.
  - `with` (CTEs / Common Table Expressions), `union`, `union_all`.
  - `compile()` producing verified PostgreSQL SQL.

### Milestone 4: Mutation Query Builders & RETURNING
- `InsertQueryBuilder`:
  - `values` (single row and batch inserts).
  - `on_conflict`: `do_nothing()`, `do_update_set(...)`.
  - `returning` and `returning_all`.
- `UpdateQueryBuilder`:
  - `set` (columns, expressions, raw SQL).
  - `where_`, `where_ref`, `returning`.
- `DeleteQueryBuilder`:
  - `where_`, `returning`.

### Milestone 5: PostgreSQL Driver & Execution Engine
- `DatabaseConnection` and `Driver` traits.
- `PostgresDialect` wrapping `tokio-postgres`.
- `Kysely` main client struct.
- Result representation: `Row`, `QueryResult`, typed row extraction (`.get::<T>("col")?`, `.execute_into::<T>()`).
- Transaction support (`db.transaction(|trx| ...)`), savepoints.

### Milestone 6: Schema DDL & Introspection
- `SchemaModule`:
  - `create_table`, `alter_table`, `drop_table`.
  - `create_index`, `drop_index`.
  - `create_schema`, `drop_schema`.
  - `create_type` (Postgres enums), `create_view`.
- `PostgresIntrospector` querying `information_schema` and `pg_catalog`.

### Milestone 7: Migrator & Advisory Locks
- `Migrator` and `MigrationProvider` (file-based and in-memory).
- PostgreSQL advisory lock synchronization (`pg_advisory_lock`, `pg_advisory_unlock`).
- Migration state table creation and tracking (`migrate_to_latest`, `migrate_up`, `migrate_down`).

### Milestone 8: Plugin Architecture
- `KyselyPlugin` trait with `transform_query` and `transform_result`.
- Built-in plugins: `CamelCasePlugin`, `DeduplicateJoinsPlugin`, `WithSchemaPlugin`.
