# Tasks: 03 — Select Query Builder, Joins, Projections & Set Operations

### TASK-SEL-001: Implement IntoSelections trait and tuple projections
- **REQ**: REQ-SEL-005, REQ-SEL-006
- **Symbols**: `IntoSelections`, `IntoSelectionItem`
- **What**: Implement `IntoSelections` supporting single items, string arrays, heterogeneous tuples up to 12 items, and closure callbacks (`|eb| (...)`).
- **Where**: `src/parser/mod.rs`, `src/parser/select_parser.rs`
- **Depends on**: none
- **Done when**: Single strings, arrays, and tuples of strings mixed with expressions convert into `SmallVec<[Arc<OperationNode>; 4]>`.
- **Gate**: `cargo test test_into_selections`

### TASK-SEL-002: Implement IntoTableSource and select_from constructor
- **REQ**: REQ-SEL-001, REQ-SEL-002
- **Symbols**: `SelectQueryBuilder`, `select_from`, `IntoTableSource`
- **What**: Implement `SelectQueryBuilder` skeleton and `select_from` supporting table names, `"table as alias"`, and subqueries.
- **Where**: `src/parser/table_parser.rs`, `src/query_builder/select_query_builder.rs`, `src/query_builder/mod.rs`
- **Depends on**: TASK-SEL-001
- **Done when**: `select_from("users as u")` creates a valid `SelectQueryBuilder` with `from` clause.
- **Gate**: `cargo test test_select_from`

### TASK-SEL-003: Implement join methods and JoinBuilder
- **REQ**: REQ-SEL-003
- **Symbols**: `JoinBuilder`, `inner_join`, `left_join`, `right_join`, `full_join`
- **What**: Implement `JoinBuilder` and join methods on `SelectQueryBuilder` supporting direct column equality and closure builders (`|jb| jb.on(...).on_ref(...)`).
- **Where**: `src/query_builder/join_builder.rs`, `src/query_builder/select_query_builder.rs`
- **Depends on**: TASK-SEL-002
- **Done when**: Direct joins and closure joins produce the expected `JoinNode`s in the AST.
- **Gate**: `cargo test test_select_joins`

### TASK-SEL-004: Implement filtering methods (where_, where_ref, where_expr, or_where_*)
- **REQ**: REQ-SEL-004
- **Symbols**: `where_`, `where_ref`, `where_expr`, `or_where_`, `or_where_ref`, `or_where_expr`
- **What**: Implement filtering methods with `AND` and `OR` conjunctions, supporting values, column references, and `|eb| ...` closures.
- **Where**: `src/query_builder/select_query_builder.rs`
- **Depends on**: TASK-SEL-003
- **Done when**: Filtering chains combine correctly into `WhereNode` in the AST.
- **Gate**: `cargo test test_select_where`

### TASK-SEL-005: Implement grouping, having, sorting, and pagination
- **REQ**: REQ-SEL-007, REQ-SEL-008, REQ-SEL-009
- **Symbols**: `group_by`, `having`, `having_ref`, `having_expr`, `order_by`, `limit`, `offset`, `distinct`, `distinct_on`
- **What**: Implement `group_by`, `having*`, `order_by`, `limit`, `offset`, `distinct`, and `distinct_on` methods.
- **Where**: `src/query_builder/select_query_builder.rs`
- **Depends on**: TASK-SEL-004
- **Done when**: Clauses populate their respective AST nodes and compile accurately.
- **Gate**: `cargo test test_select_clauses`

### TASK-SEL-006: Implement CTEs (with) and set operations (union, intersect, except)
- **REQ**: REQ-SEL-010
- **Symbols**: `with`, `with_recursive`, `union`, `union_all`, `intersect`, `except`
- **What**: Implement Common Table Expressions and set operations on `SelectQueryBuilder`.
- **Where**: `src/query_builder/select_query_builder.rs`
- **Depends on**: TASK-SEL-005
- **Done when**: CTEs and set operations construct valid AST nodes.
- **Gate**: `cargo test test_select_with_and_set_ops`

### TASK-SEL-007: Comprehensive integration tests for SelectQueryBuilder
- **REQ**: REQ-SEL-001, REQ-SEL-002, REQ-SEL-003, REQ-SEL-004, REQ-SEL-005, REQ-SEL-006, REQ-SEL-007, REQ-SEL-008, REQ-SEL-009, REQ-SEL-010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests compiling realistic SELECT queries with PostgresQueryCompiler and verifying zero warnings.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-SEL-006
- **Done when**: All integration tests pass and `cargo clippy -- -D warnings` succeeds.
- **Gate**: `cargo test --test integration`
