# Spec: 03 — Select Query Builder, Joins, Projections & Set Operations

## Summary
Implements the core `SelectQueryBuilder` in `rust-kysely`, ported from upstream `src/query-builder/select-query-builder.ts`. Provides a complete, fluent API for reading data with joins (`inner_join`, `left_join`, `right_join`, `full_join`), filtering (`where_`, `where_ref`, `where_expr`), grouping (`group_by`), having filters (`having`, `having_ref`, `having_expr`), sorting (`order_by`), pagination (`limit`, `offset`), deduplication (`distinct`, `distinct_on`), set operations (`union`, `union_all`, `intersect`, `except`), and Common Table Expressions (`with` / CTEs). Unifies selection lists via the `IntoSelections` trait, supporting both homogeneous string arrays (`["id", "name"]`) and heterogeneous tuples (`("id", "name", count(...))`) in a single, macro-free `.select(...)` call.

## Requirements
- REQ-SEL-001: Implement `SelectQueryBuilder` maintaining an immutable `Arc<SelectQueryNode>` and an executor reference.
- REQ-SEL-002: Implement `select_from(table)` supporting table names, aliases (`"users as u"`), and subquery expressions (`|eb| eb.select_from(...).as_("sub")`).
- REQ-SEL-003: Implement join methods: `inner_join`, `left_join`, `right_join`, `full_join` supporting both direct equality joins (`join("orders", "orders.user_id", "users.id")`) and closure-based join builders (`.inner_join("orders", |jb| jb.on(...).on_ref(...))`).
- REQ-SEL-004: Implement filter methods: `where_(col, op, val)`, `where_ref(col, op, col)`, and `where_expr(|eb| ...)` with logical disjunction equivalents (`or_where_`, `or_where_ref`, `or_where_expr`).
- REQ-SEL-005: Implement `IntoSelections` trait allowing `.select(...)` to accept a single column/expression, a string array (`["id", "name"]`), a heterogeneous tuple (`("id", "name", expr)`), or a callback closure (`|eb| (...)`).
- REQ-SEL-006: Implement `select_all()` producing `SelectAllNode` (`SELECT *`) or qualified table selection (`select_all("users")` -> `SELECT "users".*`).
- REQ-SEL-007: Implement grouping and having: `group_by(["u.id", "u.name"])`, `having(col, op, val)`, `having_ref`, and `having_expr(|eb| ...)`.
- REQ-SEL-008: Implement sorting: `order_by(expr, direction)` supporting `"asc"`, `"desc"`, and nulls ordering options (`nulls_first`, `nulls_last`).
- REQ-SEL-009: Implement pagination and deduplication: `limit(n)`, `offset(n)`, `distinct()`, and PostgreSQL `distinct_on(["col1", "col2"])`.
- REQ-SEL-010: Implement CTEs (`with("cte_name", |eb| eb.select_from(...))`, `with_recursive`) and set operations (`union`, `union_all`, `intersect`, `except`).

## Affected Components
- `src/query_builder/select_query_builder.rs` — `SelectQueryBuilder`
- `src/query_builder/join_builder.rs` — `JoinBuilder`
- `src/parser/select_parser.rs` — `IntoSelections`, tuple implementations
- `src/operation_node/select_query_node.rs` — AST select representation

## Out of Scope
- Direct in-place mutation of builders (builders are immutable / copy-on-write).
- MySQL-specific non-standard select modifiers (`SQL_CALC_FOUND_ROWS`).

## Open Questions
- None.
