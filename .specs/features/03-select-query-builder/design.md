# Design: 03 — Select Query Builder, Joins, Projections & Set Operations

## Architecture Overview

`SelectQueryBuilder` implements an immutable, fluent query builder for SQL SELECT statements. It wraps an `Arc<SelectQueryNode>` with Copy-on-Write mutations (`Arc::make_mut`), ensuring thread safety and query reusability with zero redundant heap allocations.

```
┌────────────────────────────────────────────────────────┐
│                   SelectQueryBuilder                   │
│   - .select_from("users as u")                         │
│   - .select(("u.id", "u.name", eb.fn().count(...)))    │
│   - .inner_join("orders", "orders.user_id", "u.id")    │
│   - .where_("u.age", ">=", 18)                         │
│   - .compile(&PostgresQueryCompiler::new())            │
└───────────────────────────┬────────────────────────────┘
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
┌──────────────┐   ┌─────────────────┐   ┌────────────────┐
│IntoSelections│   │   JoinBuilder   │   │   Set Ops &    │
│Tuples/Arrays │   │(jb.on / on_ref) │   │ CTEs (with)    │
└──────┬───────┘   └────────┬────────┘   └───────┬────────┘
       │                    │                    │
       └────────────────────┼────────────────────┘
                            ▼
┌────────────────────────────────────────────────────────┐
│                   SelectQueryNode                      │
│                  (OperationNode AST)                   │
└────────────────────────────────────────────────────────┘
```

## Dependency Paths (from nexspec)
- REQ-SEL-001 → `src/query_builder/select_query_builder.rs` (`SelectQueryBuilder`)
- REQ-SEL-002 → `select_from` parsing `"table as alias"` and subqueries
- REQ-SEL-003 → `src/query_builder/join_builder.rs` (`JoinBuilder`, `inner_join`, etc.)
- REQ-SEL-004 → `where_`, `where_ref`, `where_expr`, `or_where_*`
- REQ-SEL-005 → `src/parser/select_parser.rs` (`IntoSelections` and tuples)
- REQ-SEL-006 → `select_all()` (`*` and `"table".*`)
- REQ-SEL-007 → `group_by`, `having`, `having_ref`, `having_expr`
- REQ-SEL-008 → `order_by` (`asc`, `desc`, nulls placement)
- REQ-SEL-009 → `limit`, `offset`, `distinct`, `distinct_on`
- REQ-SEL-010 → CTEs (`with`, `with_recursive`) and set operations

## New Components

| Component | Responsibility | Location |
|---|---|---|
| `SelectQueryBuilder` | Immutable fluent query builder for SELECT queries | `src/query_builder/select_query_builder.rs` |
| `JoinBuilder` | Context passed to closure for multi-condition JOIN clauses | `src/query_builder/join_builder.rs` |
| `IntoSelections` | Trait enabling heterogeneous tuples, arrays, and closures in `.select(...)` | `src/parser/select_parser.rs` |
| `IntoTableSource` | Trait for table names, aliases, and subqueries passed to `select_from` | `src/parser/table_parser.rs` |

## Modified Components

| Component | Change | Risk |
|---|---|---|
| `src/lib.rs` | Expose `SelectQueryBuilder`, `JoinBuilder`, `IntoSelections` | Low |

## Risks & Mitigation
- **Tuple Size Limit**: Standard Rust macros/traits must support common tuple sizes.
  *Mitigation*: Implement `IntoSelections` for single items, arrays, and tuples up to 12 items.
- **Copy-on-Write In-Place Optimization**: Chained calls should not clone unless shared.
  *Mitigation*: `SelectQueryNode::clone_mut(&mut self.node)` uses `Arc::make_mut` guaranteeing zero clone overhead when builder is unshared.

## Decision Log
- **Heterogeneous Tuples vs Dynamic Boxing**: Used the `IntoSelections` trait pattern (as in Axum/Actix) to support heterogeneous tuples `("id", "name", expr)` statically at compile time with zero heap boxing.
