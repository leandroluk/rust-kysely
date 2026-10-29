# Tasks: 05 — PostgreSQL Driver, Connection Pooling & Execution Engine

### TASK-DRV-001: Implement Driver and DatabaseConnection async traits [x]
- **REQ**: REQ-EXEC-001
- **Symbols**: `Driver`, `DatabaseConnection`, `DriverError`
- **What**: Implement async core traits for database drivers, connections, and error handling.
- **Where**: `src/driver/mod.rs`, `src/driver/traits.rs`, `src/driver/error.rs`
- **Depends on**: none
- **Done when**: Traits compile with `async_trait` and support acquiring, executing, and releasing connections.
- **Gate**: `cargo test test_driver_traits`

### TASK-DRV-002: Implement QueryResult, Row, and type-safe value extraction [x]
- **REQ**: REQ-EXEC-005, REQ-EXEC-006
- **Symbols**: `QueryResult`, `Row`, `ColumnError`, `.get()`, `.get_opt()`, `.into_record()`
- **What**: Implement `Row` holding column-to-Value mappings with typed getters (`get::<i64>`, `get::<String>`, `get::<bool>`, etc.) and serde deserialization.
- **Where**: `src/query_executor/result.rs`, `src/query_executor/row.rs`, `src/query_executor/mod.rs`
- **Depends on**: TASK-DRV-001
- **Done when**: Row lookups succeed for valid types and return clear error messages on type/column mismatch.
- **Gate**: `cargo test test_row_extraction`

### TASK-DRV-003: Implement QueryExecutor and execution coordinating pipeline [x]
- **REQ**: REQ-EXEC-004, REQ-EXEC-007, REQ-EXEC-008
- **Symbols**: `QueryExecutor`, `DefaultQueryExecutor`, `execute_take_first`, `execute_take_first_val`
- **What**: Implement executor compiling queries, acquiring driver connections, executing, transforming results, and extracting first items.
- **Where**: `src/query_executor/executor.rs`, `src/query_executor/mod.rs`
- **Depends on**: TASK-DRV-002
- **Done when**: Executor compiles and executes queries against a driver, correctly returning `Option<Row>` or first scalar.
- **Gate**: `cargo test test_query_executor`

### TASK-DRV-004: Implement Kysely client struct and builder execution chaining
- **REQ**: REQ-EXEC-003, REQ-EXEC-004, REQ-EXEC-006
- **Symbols**: `Kysely`, `Dialect`, `.execute()`, `.execute_take_first()`, `.execute_into()`
- **What**: Implement main `Kysely` facade and connect `Arc<dyn QueryExecutor>` to `SelectQueryBuilder`, `InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`.
- **Where**: `src/kysely.rs`, `src/dialect/mod.rs`, `src/query_builder/`
- **Depends on**: TASK-DRV-003
- **Done when**: Builders can execute directly via `.execute().await` and `.execute_into::<T>().await`.
- **Gate**: `cargo test test_kysely_builder_execution`

### TASK-DRV-005: Implement Transaction and Savepoint management
- **REQ**: REQ-EXEC-009, REQ-EXEC-010
- **Symbols**: `Transaction`, `db.transaction().execute()`, `.savepoint()`, `.rollback_to_savepoint()`
- **What**: Implement atomic transaction wrapper committing on `Ok` and rolling back on `Err`, with named savepoints.
- **Where**: `src/transaction.rs`, `src/kysely.rs`
- **Depends on**: TASK-DRV-004
- **Done when**: Transactions execute BEGIN/COMMIT on success, and BEGIN/ROLLBACK on error.
- **Gate**: `cargo test test_transaction_and_savepoints`

### TASK-DRV-006: Implement PostgresDriver with tokio-postgres
- **REQ**: REQ-EXEC-002, REQ-EXEC-003
- **Symbols**: `PostgresDriver`, `PostgresDialect`
- **What**: Implement `PostgresDriver` converting `kysely::Value` parameters to `tokio-postgres` types and streaming query execution.
- **Where**: `src/dialect/postgres/driver.rs`, `src/dialect/postgres/dialect.rs`, `src/dialect/postgres/mod.rs`
- **Depends on**: TASK-DRV-001, TASK-DRV-004
- **Done when**: `PostgresDriver` compiles and converts parameters to wire format.
- **Gate**: `cargo test test_postgres_driver_parameter_conversion`

### TASK-DRV-007: Comprehensive integration tests for query execution engine
- **REQ**: REQ-EXEC-001..010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests verifying execution, row extraction, serde deserialization, transactions, and savepoints.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-DRV-005, TASK-DRV-006
- **Done when**: Full test suite passes, zero warnings, footprint clean.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings`
