# Design: 04 — Mutation Query Builders: Insert, Update, Delete & RETURNING

## Architecture Overview

Mutation query builders (`InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`) manage Data Manipulation Language (DML) statements in `rust-kysely`. Each builder wraps an `Arc` pointer to its respective AST node (`InsertQueryNode`, `UpdateQueryNode`, `DeleteQueryNode`) and implements immutable Copy-on-Write mutations (`Arc::make_mut`).

```
                    ┌────────────────────────┐
                    │    insert_into(table)  │
                    └───────────┬────────────┘
                                │
        ┌───────────────────────┼────────────────────────┐
        ▼                       ▼                        ▼
┌──────────────┐     ┌─────────────────────┐    ┌─────────────────┐
│ .set(col,val)│     │.values_batch(rows)  │    │.on_conflict(oc) │
└───────┬──────┘     └──────────┬──────────┘    └────────┬────────┘
        │                       │                        │
        └───────────────────────┼────────────────────────┘
                                ▼
                    ┌────────────────────────┐
                    │.returning(["id", ...]) │
                    └───────────┬────────────┘
                                ▼
                    ┌────────────────────────┐
                    │    compile(compiler)   │
                    └────────────────────────┘
```

## Component Architecture

### 1. `InsertQueryBuilder`
- Holds `Arc<InsertQueryNode>`.
- Supports single row via chained `.set(col, val)`.
- Supports multiple rows / closures via `.values(|v| ...)` and `.values_batch(...)`.
- Supports structs via `.values_record(&struct)` (when feature `"json"` is enabled).
- Supports `INSERT INTO ... SELECT ...` via `.columns([...]).expression(select_query)`.
- Supports `ON CONFLICT` via `.on_conflict(impl FnOnce(OnConflictBuilder) -> OnConflictBuilder)`.
- Supports `RETURNING` via `.returning(selections)` and `.returning_all()`.

### 2. `OnConflictBuilder`
- Configures conflict target: `.column(col)`, `.columns([cols])`, or `.constraint(name)`.
- Configures conflict resolution:
  - `.do_nothing()` -> `DO NOTHING`
  - `.do_update_set(|ub| ...)` -> `DO UPDATE SET col = val, ...`
- Supports `excluded` table references: `.set_ref("name", "excluded.name")` or `eb.ref_("excluded.name")`.
- Supports conflict WHERE filtering: `.where_`, `.where_ref`, `.where_expr`.

### 3. `UpdateQueryBuilder`
- Holds `Arc<UpdateQueryNode>`.
- Entry point: `update_table(table)`.
- Chained assignments: `.set(col, val)`, `.set_ref(col1, col2)`, `.set_expr(col, expr)`.
- Joins & sources: `.from(table)`, `.inner_join(...)`, `.left_join(...)`.
- Filtering: `.where_`, `.where_ref`, `.where_expr`, `.or_where_*`.
- `RETURNING` via `.returning(selections)` and `.returning_all()`.

### 4. `DeleteQueryBuilder`
- Holds `Arc<DeleteQueryNode>`.
- Entry point: `delete_from(table)`.
- Join clauses: `.using(table)`, `.inner_join(...)`, `.left_join(...)`.
- Filtering: `.where_`, `.where_ref`, `.where_expr`, `.or_where_*`.
- `RETURNING` via `.returning(selections)` and `.returning_all()`.

## Zero-Macro Serde Mapping
Rather than generating custom proc-macro derives for rows, `.values_record(&T)` serializes any `T: serde::Serialize` into a `serde_json::Value::Object`. It then transforms entries into `ColumnNode` and `ValueNode` pairs with deterministic key ordering.

## Task Breakdown Plan
1. `TASK-MUT-001`: Implement `InsertQueryBuilder` skeleton and chained `.set(col, val)` [REQ-MUT-001, REQ-MUT-002].
2. `TASK-MUT-002`: Implement `.values(...)`, `.values_batch(...)`, and `.values_record(...)` [REQ-MUT-003, REQ-MUT-004].
3. `TASK-MUT-003`: Implement `OnConflictBuilder` and `.on_conflict(...)` upsert handling with `excluded` [REQ-MUT-005, REQ-MUT-006].
4. `TASK-MUT-004`: Implement `UpdateQueryBuilder` with `.set()`, `.from()`, joins, and filtering [REQ-MUT-007, REQ-MUT-008].
5. `TASK-MUT-005`: Implement `DeleteQueryBuilder` with `using()`, joins, and filtering [REQ-MUT-009].
6. `TASK-MUT-006`: Implement `returning(selections)` and `returning_all()` on all mutation builders [REQ-MUT-010].
7. `TASK-MUT-007`: Comprehensive integration tests for DML builders in `tests/integration.rs`.
