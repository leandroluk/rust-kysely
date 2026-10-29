# Tasks: 01 — Core OperationNode AST, Visitors & PostgreSQL Compiler

### TASK-AST-001: Implement QueryId generator and Value scalar representation [x]
- **REQ**: REQ-AST-005, REQ-AST-001
- **Symbols**: `QueryId`, `Value`
- **What**: Create 64-bit atomic/random `QueryId` in `src/util/query_id.rs` and the SQL literal `Value` enum (Null, Bool, I64, F64, String, Bytes) in `src/operation_node/value.rs`.
- **Where**: `src/util/mod.rs`, `src/util/query_id.rs`, `src/operation_node/mod.rs`, `src/operation_node/value.rs`
- **Depends on**: none
- **[P]**: wave-1
- **Done when**: Unit tests pass verifying unique `QueryId` generation and `Value` formatting/conversions.
- **Gate**: `cargo test test_query_id test_value`

### TASK-AST-002: Implement Terminal & Core Identifier AST nodes
- **REQ**: REQ-AST-001, REQ-AST-003
- **Symbols**: `IdentifierNode`, `TableNode`, `ColumnNode`, `ReferenceNode`, `RawNode`, `OperationNode`
- **What**: Implement terminal AST nodes using `compact_str::CompactStr` for zero-allocation identifier storage and define the top-level `OperationNode` enum.
- **Where**: `src/operation_node/terminal_nodes.rs`, `src/operation_node/mod.rs`
- **Depends on**: TASK-AST-001
- **Done when**: Terminal nodes construct cleanly, wrap in `OperationNode`, and fit identifiers <= 24 bytes inline.
- **Gate**: `cargo test test_terminal_nodes`

### TASK-AST-003: Implement Clause and Expression AST nodes
- **REQ**: REQ-AST-001, REQ-AST-004
- **Symbols**: `FromNode`, `JoinNode`, `SelectionNode`, `WhereNode`, `HavingNode`, `GroupByNode`, `OrderByNode`, `LimitNode`, `OffsetNode`, `WithNode`, `BinaryOperationNode`, `UnaryOperationNode`, `CaseNode`, `WhenNode`, `CastNode`, `AliasNode`, `ParensNode`
- **What**: Implement all clause and expression nodes using `SmallVec<[T; 4]>` for selections, joins, and parameters.
- **Where**: `src/operation_node/clause_nodes.rs`, `src/operation_node/expression_nodes.rs`, `src/operation_node/mod.rs`
- **Depends on**: TASK-AST-002
- **Done when**: Clauses and binary/unary expressions construct properly and integrate into `OperationNode`.
- **Gate**: `cargo test test_clause_nodes test_expression_nodes`

### TASK-AST-004: Implement Query AST nodes with Copy-on-Write support
- **REQ**: REQ-AST-001, REQ-AST-002
- **Symbols**: `SelectQueryNode`, `InsertQueryNode`, `UpdateQueryNode`, `DeleteQueryNode`
- **What**: Implement the primary query nodes with `Arc` wrapping and builder mutation helpers using `Arc::make_mut`.
- **Where**: `src/operation_node/query_nodes.rs`, `src/operation_node/mod.rs`
- **Depends on**: TASK-AST-003
- **Done when**: Query nodes can be cloned cheaply via `Arc` and mutated in-place when uniquely referenced.
- **Gate**: `cargo test test_query_nodes_cow`

### TASK-AST-005: Implement OperationNodeVisitor and OperationNodeTransformer traits
- **REQ**: REQ-AST-006, REQ-AST-007
- **Symbols**: `OperationNodeVisitor`, `OperationNodeTransformer`
- **What**: Implement recursive tree visitor with parent tracking stack and AST transformer for immutable query rewriting.
- **Where**: `src/operation_node/visitor.rs`, `src/operation_node/transformer.rs`, `src/operation_node/mod.rs`
- **Depends on**: TASK-AST-004
- **Done when**: Visitor correctly traverses all AST variants with parent stack tracking, and Transformer successfully rewrites target nodes.
- **Gate**: `cargo test test_visitor test_transformer`

### TASK-AST-006: Implement CompiledQuery and DefaultQueryCompiler
- **REQ**: REQ-AST-008
- **Symbols**: `CompiledQuery`, `QueryCompiler`, `DefaultQueryCompiler`
- **What**: Implement `CompiledQuery { sql, parameters, query_id }` and the ANSI SQL `DefaultQueryCompiler` with parameter tracking and `?` placeholders.
- **Where**: `src/query_compiler/compiled_query.rs`, `src/query_compiler/default_compiler.rs`, `src/query_compiler/mod.rs`
- **Depends on**: TASK-AST-005
- **Done when**: `DefaultQueryCompiler` produces valid ANSI SQL for SELECT, INSERT, UPDATE, DELETE queries with `?` parameters.
- **Gate**: `cargo test test_default_compiler`

### TASK-AST-007: Implement PostgresQueryCompiler
- **REQ**: REQ-AST-009
- **Symbols**: `PostgresQueryCompiler`
- **What**: Implement PostgreSQL query compiler with sequential dollar placeholders (`$1, $2, ...`), double-quoted identifier escaping (`"col"`), and dialect-specific clauses (e.g. `RETURNING`).
- **Where**: `src/dialect/postgres/query_compiler.rs`, `src/dialect/postgres/mod.rs`, `src/dialect/mod.rs`
- **Depends on**: TASK-AST-006
- **Done when**: `PostgresQueryCompiler` compiles complex queries with `$N` parameters and correctly escaped `"double_quotes"`.
- **Gate**: `cargo test test_postgres_compiler`

### TASK-AST-008: Comprehensive Integration Tests and Footprint Verification
- **REQ**: REQ-AST-010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: Add complete test harness in `tests/integration.rs` covering multi-table JOINs, subqueries, CTEs (`WITH`), CASE expressions, and verify zero proc-macros and low `target/` size.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-AST-007
- **Done when**: Full test suite passes without warnings, and `target/` size remains controlled.
- **Gate**: `cargo test --test integration`
