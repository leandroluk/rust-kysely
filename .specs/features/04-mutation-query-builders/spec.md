# Spec: 04 — Mutation Query Builders: Insert, Update, Delete & RETURNING

## Summary
Implements data manipulation query builders (`InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`) in `rust-kysely`, ported from upstream `src/query-builder/insert-query-builder.ts`, `update-query-builder.ts`, and `delete-query-builder.ts`. Eliminates redundant `.into()` calls by leveraging the `IntoValue` trait, chained `.set(col, val)` methods, closure builders (`.values(|v| ...)`), batch inserts (`.values_batch(...)`), and serde struct mapping. Implements full PostgreSQL `ON CONFLICT` upsert capabilities (`do_nothing()`, `do_update_set(|eb| ...)`), and `RETURNING` clauses (`returning(...)`, `returning_all()`) across all mutation operations.

## Requirements
- REQ-MUT-001: Implement `InsertQueryBuilder` managing `InsertQueryNode` with immutable Copy-on-Write semantics.
- REQ-MUT-002: Implement chained `.set(col, val)` on `InsertQueryBuilder` where `val: impl IntoValue`, allowing zero-boilerplate value assignment without manual `.into()` calls.
- REQ-MUT-003: Implement `.values(|v| ...)` closure builder and `.values_batch(...)` for inserting multiple rows in a single batch insert query.
- REQ-MUT-004: Implement `.values_record(&struct)` allowing standard serde structs to populate insert columns and values without custom derive macros.
- REQ-MUT-005: Implement PostgreSQL `on_conflict(|oc| ...)` supporting conflict targets (columns, constraints, where conditions) and conflict actions (`do_nothing()`, `do_update_set(|eb| ...)`).
- REQ-MUT-006: Support `excluded` pseudo-table references in upsert update sets (`eb.ref_("excluded.name")`).
- REQ-MUT-007: Implement `UpdateQueryBuilder` managing `UpdateQueryNode` with `.set(col, val)`, `.set_expr(col, expr)`, and `.set_values(|v| ...)`.
- REQ-MUT-008: Support filtering and table joins in updates (`where_`, `where_ref`, `where_expr`, `from`, `inner_join`).
- REQ-MUT-009: Implement `DeleteQueryBuilder` managing `DeleteQueryNode` with `.where_`, `.where_ref`, `.where_expr`, and `using` joins.
- REQ-MUT-010: Implement `returning(selections)` and `returning_all()` on `InsertQueryBuilder`, `UpdateQueryBuilder`, and `DeleteQueryBuilder`, compiling to standard PostgreSQL `RETURNING` clauses.

## Affected Components
- `src/query_builder/insert_query_builder.rs` — `InsertQueryBuilder`, `RowValues`
- `src/query_builder/on_conflict_builder.rs` — `OnConflictBuilder`
- `src/query_builder/update_query_builder.rs` — `UpdateQueryBuilder`
- `src/query_builder/delete_query_builder.rs` — `DeleteQueryBuilder`

## Out of Scope
- Procedural derive macros for row insertion (uses standard serde or `.set()` builders).
- MySQL `ON DUPLICATE KEY UPDATE` (Postgres focus first; `ON CONFLICT` used).

## Open Questions
- None.
