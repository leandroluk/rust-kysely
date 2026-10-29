# Tasks: 08 — Plugin Architecture & Built-in Plugins

### TASK-PLUG-001: Implement KyselyPlugin Trait and Executor Pipeline
- **REQ**: REQ-PLUG-001, REQ-PLUG-006
- **Symbols**: `KyselyPlugin`, `DefaultQueryExecutor::with_plugins`, `Kysely::with_plugin`
- **What**: Implement the core `KyselyPlugin` async trait and integrate the plugin execution pipeline into `QueryExecutor` (transforming AST forward, transforming results in reverse).
- **Where**: `src/plugin/plugin.rs`, `src/plugin/mod.rs`, `src/query_executor/executor.rs`, `src/kysely.rs`
- **Depends on**: Milestone 7
- **Done when**: Plugins intercept AST transformations and result transformations cleanly.
- **Gate**: `cargo test test_plugin_pipeline`

### TASK-PLUG-002: Implement CamelCasePlugin
- **REQ**: REQ-PLUG-002, REQ-PLUG-003
- **Symbols**: `CamelCasePlugin`, `CamelCaseOptions`
- **What**: Implement camelCase to snake_case AST conversion for columns and tables, and snake_case to camelCase conversion for QueryResult rows.
- **Where**: `src/plugin/camel_case.rs`, `src/plugin/mod.rs`
- **Depends on**: TASK-PLUG-001
- **Done when**: camelCase column references compile to snake_case SQL, and returned snake_case row columns map back to camelCase.
- **Gate**: `cargo test test_camel_case_plugin`

### TASK-PLUG-003: Implement DeduplicateJoinsPlugin
- **REQ**: REQ-PLUG-004
- **Symbols**: `DeduplicateJoinsPlugin`
- **What**: Implement join deduplication AST transformer removing duplicate `JoinNode`s on `SelectQueryNode`.
- **Where**: `src/plugin/deduplicate_joins.rs`, `src/plugin/mod.rs`
- **Depends on**: TASK-PLUG-001
- **Done when**: Multiple identical joins attached to a select query are collapsed into a single join.
- **Gate**: `cargo test test_deduplicate_joins_plugin`

### TASK-PLUG-004: Implement WithSchemaPlugin
- **REQ**: REQ-PLUG-005
- **Symbols**: `WithSchemaPlugin`
- **What**: Implement schema prefixing plugin that wraps unqualified table names with a specified schema in AST.
- **Where**: `src/plugin/with_schema.rs`, `src/plugin/mod.rs`
- **Depends on**: TASK-PLUG-001
- **Done when**: Table `"users"` compiles to `"custom_schema"."users"` in select, insert, update, and delete.
- **Gate**: `cargo test test_with_schema_plugin`

### TASK-PLUG-005: Comprehensive Integration Tests for Plugin System
- **REQ**: REQ-PLUG-001..006
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests combining multiple plugins (`CamelCasePlugin` + `DeduplicateJoinsPlugin` + `WithSchemaPlugin`) on `Kysely` and per-query.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-PLUG-001..004
- **Done when**: Complete test suite passes, zero warnings, footprint clean.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings`
