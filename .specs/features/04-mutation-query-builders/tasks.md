# Tasks: 04 — Mutation Query Builders: Insert, Update, Delete & RETURNING

### TASK-MUT-001: Implement InsertQueryBuilder and chained set method [x]
- **REQ**: REQ-MUT-001, REQ-MUT-002
- **Symbols**: `InsertQueryBuilder`, `insert_into`, `.set()`
- **What**: Implement `InsertQueryBuilder` managing `InsertQueryNode` with CoW, `insert_into` constructor, and `.set(col, val)` supporting `val: impl IntoValue`.
- **Where**: `src/query_builder/insert_query_builder.rs`, `src/query_builder/mod.rs`
- **Depends on**: none
- **Done when**: Single-row insert compiles to `INSERT INTO table (cols) VALUES (?)` with parameters.
- **Gate**: `cargo test test_insert_single_row`

### TASK-MUT-002: Implement values, values_batch, and values_record [x]
- **REQ**: REQ-MUT-003, REQ-MUT-004
- **Symbols**: `RowValues`, `.values()`, `.values_batch()`, `.values_record()`
- **What**: Implement multi-row insert helpers, closure row builders, and serde struct serialization into insert columns and values.
- **Where**: `src/query_builder/insert_query_builder.rs`
- **Depends on**: TASK-MUT-001
- **Done when**: Batch inserts and serde structs produce multiple parameter rows in `ValuesNode`.
- **Gate**: `cargo test test_insert_batch_and_record`

### TASK-MUT-003: Implement OnConflictBuilder and PostgreSQL upsert [x]
- **REQ**: REQ-MUT-005, REQ-MUT-006
- **Symbols**: `OnConflictBuilder`, `.on_conflict()`
- **What**: Implement `OnConflictBuilder` supporting `do_nothing()`, `do_update_set()`, column targets, where filtering, and `excluded` table references.
- **Where**: `src/query_builder/on_conflict_builder.rs`, `src/query_builder/insert_query_builder.rs`
- **Depends on**: TASK-MUT-002
- **Done when**: Upsert queries compile accurately with `ON CONFLICT (cols) DO UPDATE SET col = excluded.col`.
- **Gate**: `cargo test test_insert_on_conflict`

### TASK-MUT-004: Implement UpdateQueryBuilder with joins and filters [x]
- **REQ**: REQ-MUT-007, REQ-MUT-008
- **Symbols**: `UpdateQueryBuilder`, `update_table`, `.set()`, `.set_ref()`, `.set_expr()`
- **What**: Implement `UpdateQueryBuilder` supporting assignments, `from`, joins, and where filters (`where_`, `where_ref`, `where_expr`, `or_where_*`).
- **Where**: `src/query_builder/update_query_builder.rs`, `src/query_builder/mod.rs`
- **Depends on**: TASK-MUT-001
- **Done when**: UPDATE query with SET, FROM, JOIN, and WHERE compiles accurately to SQL.
- **Gate**: `cargo test test_update_query`

### TASK-MUT-005: Implement DeleteQueryBuilder with using joins and filters [x]
- **REQ**: REQ-MUT-009
- **Symbols**: `DeleteQueryBuilder`, `delete_from`, `.using()`, `.where_()`
- **What**: Implement `DeleteQueryBuilder` managing `DeleteQueryNode` with `using` joins and where filters.
- **Where**: `src/query_builder/delete_query_builder.rs`, `src/query_builder/mod.rs`
- **Depends on**: TASK-MUT-001
- **Done when**: DELETE query with USING and WHERE compiles accurately to SQL.
- **Gate**: `cargo test test_delete_query`

### TASK-MUT-006: Implement returning and returning_all on all mutation builders [x]
- **REQ**: REQ-MUT-010
- **Symbols**: `.returning()`, `.returning_all()`
- **What**: Add `.returning(selections: impl IntoSelections)` and `.returning_all()` to `InsertQueryBuilder`, `UpdateQueryBuilder`, and `DeleteQueryBuilder`.
- **Where**: `src/query_builder/insert_query_builder.rs`, `src/query_builder/update_query_builder.rs`, `src/query_builder/delete_query_builder.rs`
- **Depends on**: TASK-MUT-003, TASK-MUT-004, TASK-MUT-005
- **Done when**: All three mutation builders append valid RETURNING clauses to their compiled SQL.
- **Gate**: `cargo test test_mutation_returning`

### TASK-MUT-007: Comprehensive integration tests for DML builders
- **REQ**: REQ-MUT-001, REQ-MUT-002, REQ-MUT-003, REQ-MUT-004, REQ-MUT-005, REQ-MUT-006, REQ-MUT-007, REQ-MUT-008, REQ-MUT-009, REQ-MUT-010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests compiling realistic INSERT, UPDATE, DELETE queries with PostgresQueryCompiler and verifying zero warnings.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-MUT-006
- **Done when**: All tests pass, clippy has 0 warnings, target footprint stays under control.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings`
