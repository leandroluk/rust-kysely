# Architecture

## Overview
`rust-kysely` is a pure Rust port of the TypeScript SQL query builder Kysely (https://github.com/kysely-org/kysely). It replicates Kysely's modular architecture, clean separation of concerns, and introspectable AST without using procedural macros.

## Core Pillars
1. **OperationNode AST**:
   An immutable, typed enum/struct hierarchy representing SQL operations (`SelectQueryNode`, `InsertQueryNode`, `UpdateQueryNode`, `DeleteQueryNode`, `JoinNode`, `WhereNode`, `BinaryOperationNode`, `ValueNode`, etc.).
2. **OperationNodeVisitor & QueryCompiler**:
   Visitors that traverse the AST and produce a `CompiledQuery` (`sql: String`, `parameters: Vec<Value>`, `query_id: QueryId`).
   - `DefaultQueryCompiler`: Base ANSI/Postgres SQL compiler.
   - `PostgresQueryCompiler`: Dialect-specific compiler handling Postgres identifier quoting (`""`), parameter placeholders (`$1, $2`), and specialized operators.
3. **Fluent Query Builders**:
   - `SelectQueryBuilder`: SELECT queries, joins, filters, ordering, grouping, pagination, CTEs, unions.
   - `InsertQueryBuilder`: INSERT queries with `values`, `returning`, `on_conflict` (`do_nothing`, `do_update_set`).
   - `UpdateQueryBuilder`: UPDATE queries with `set`, `where`, `returning`.
   - `DeleteQueryBuilder`: DELETE queries with `where`, `returning`.
4. **Expression Builders (`eb`) & Raw SQL (`sql`)**:
   - `ExpressionBuilder`: Closure-based expression factory (`|eb| eb.or([...])`, subqueries, functions).
   - `sql`: Macro-free raw SQL builder with parameter binding, identifier wrapping (`sql::id`), table references (`sql::table`), and fragment composition (`sql::join`).
5. **Dialect, Driver & Execution**:
   - `Dialect`: Factory for driver, compiler, adapter, and introspector.
   - `PostgresDialect`: Production-ready PostgreSQL driver on top of `tokio-postgres`.
   - `QueryExecutor`: Orchestrates plugins, query compilation, execution, and row mapping.
6. **Schema DDL & Migrations**:
   - `SchemaModule`: Type-safe DDL builder (`create_table`, `alter_table`, `drop_table`, `create_index`, `create_view`, `create_type`).
   - `Migrator`: Schema migration runner with Postgres advisory lock synchronization (`pg_advisory_lock`).
7. **Plugins System**:
   - `KyselyPlugin`: Hooks for `transform_query` and `transform_result`.
   - Built-ins: `CamelCasePlugin`, `DeduplicateJoinsPlugin`, `WithSchemaPlugin`.
