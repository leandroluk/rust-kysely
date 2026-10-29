# Spec: 06 — Schema DDL Builders & PostgreSQL Introspection

## Summary
Implements Data Definition Language (DDL) builders and database introspection for `rust-kysely`, ported from upstream `src/schema/` and `src/dialect/postgres/postgres-introspector.ts`. Exposes `db.schema()` providing type-safe, fluent builders for creating, altering, and dropping tables, indexes, views, schemas, and PostgreSQL-specific types (enums and composite types). Column definitions support types, nullability, defaults, primary keys, foreign keys with cascading actions (`on_delete`, `on_update`), and check constraints. The `PostgresIntrospector` queries `information_schema` and `pg_catalog` to extract table schemas, column data types, nullability, and primary/foreign keys.

## Requirements
- REQ-DDL-001: Implement `SchemaModule` accessible via `db.schema()` coordinating DDL operations.
- REQ-DDL-002: Implement `create_table(name)` with `.add_column(name, data_type, |col| ...)` supporting column builders (`primary_key`, `not_null`, `unique`, `default_to`, `check`).
- REQ-DDL-003: Implement table-level constraints: `.add_primary_key_constraint(...)`, `.add_foreign_key_constraint(...)`, `.add_unique_constraint(...)`, and `.add_check_constraint(...)`.
- REQ-DDL-004: Implement foreign key referencing: `.references("target_table.col")` with `.on_delete(action)` and `.on_update(action)` (`cascade`, `restrict`, `set_null`, `no_action`).
- REQ-DDL-005: Implement `alter_table(name)` with `.add_column(...)`, `.drop_column(...)`, `.alter_column(...)`, `.rename_column(...)`, `.add_constraint(...)`, and `.drop_constraint(...)`.
- REQ-DDL-006: Implement `drop_table(name)` with `.if_exists()` and `.cascade()`.
- REQ-DDL-007: Implement index builders: `create_index(name).on(table).column(col)` with `.unique()`, `.using(method)` (e.g. `btree`, `gin`, `gist`), and partial index filters (`.where(...)`).
- REQ-DDL-008: Implement PostgreSQL custom type DDL: `create_type(name).as_enum(["val1", "val2"])`, `drop_type(name)`.
- REQ-DDL-009: Implement view DDL: `create_view(name).as(select_query)`, `drop_view(name)`.
- REQ-DDL-010: Implement `PostgresIntrospector` querying `information_schema.tables`, `columns`, and `pg_catalog` to return structured metadata (`TableMetadata`, `ColumnMetadata`).

## Affected Components
- `src/schema/` — `SchemaModule`, `CreateTableBuilder`, `AlterTableBuilder`, `ColumnDefinitionBuilder`, `CreateIndexBuilder`
- `src/dialect/postgres/postgres_introspector.rs` — `PostgresIntrospector`

## Out of Scope
- Direct live migrations without version tracking (handled by Migrator in Milestone 7).
- Non-PostgreSQL dialect-specific DDL syntax.

## Open Questions
- None.
