# Tasks: 09 — Multi-Dialect Support: MySQL, MSSQL, SQLite & PGlite

### [x] TASK-DIA-001: Implement SQLite Dialect (Compiler, Adapter, Introspector)
- **REQ**: REQ-DIA-003, REQ-DIA-005, REQ-DIA-006, REQ-DIA-007
- **Symbols**: `SqliteDialect`, `SqliteQueryCompiler`, `SqliteAdapter`, `SqliteIntrospector`
- **What**: Implement SQLite query compiler (`?` parameters, `"name"` quotes, RETURNING support), no-op migration lock adapter, and PRAGMA table/column introspector.
- **Where**: `src/dialect/sqlite/`, `src/dialect/mod.rs`
- **Depends on**: Milestone 8
- **Done when**: SQLite compiler produces valid SQLite syntax and introspector queries table schema via PRAGMA.
- **Gate**: `cargo test test_sqlite_dialect` — Passed

### [x] TASK-DIA-002: Implement MySQL Dialect (Compiler, Adapter, Introspector)
- **REQ**: REQ-DIA-001, REQ-DIA-005, REQ-DIA-006, REQ-DIA-007
- **Symbols**: `MysqlDialect`, `MysqlQueryCompiler`, `MysqlAdapter`, `MysqlIntrospector`
- **What**: Implement MySQL query compiler (`?` parameters, backtick quotes, ON DUPLICATE KEY UPDATE), `GET_LOCK`/`RELEASE_LOCK` adapter, and information_schema introspector.
- **Where**: `src/dialect/mysql/`, `src/dialect/mod.rs`
- **Depends on**: TASK-DIA-001
- **Done when**: MySQL compiler produces valid backtick and upsert syntax, adapter executes lock functions.
- **Gate**: `cargo test test_mysql_dialect` — Passed

### [x] TASK-DIA-003: Implement MSSQL Dialect (Compiler, Adapter, Introspector)
- **REQ**: REQ-DIA-002, REQ-DIA-005, REQ-DIA-006, REQ-DIA-007
- **Symbols**: `MssqlDialect`, `MssqlQueryCompiler`, `MssqlAdapter`, `MssqlIntrospector`
- **What**: Implement MSSQL query compiler (`@p1` parameters, bracket quotes `[name]`, TOP pagination, OUTPUT clause), `sp_getapplock` adapter, and INFORMATION_SCHEMA introspector.
- **Where**: `src/dialect/mssql/`, `src/dialect/mod.rs`
- **Depends on**: TASK-DIA-002
- **Done when**: MSSQL compiler produces valid bracket quotes and @p parameter placeholders.
- **Gate**: `cargo test test_mssql_dialect` — Passed

### [x] TASK-DIA-004: Implement PGlite Dialect
- **REQ**: REQ-DIA-004, REQ-DIA-005
- **Symbols**: `PGliteDialect`
- **What**: Implement PGlite dialect adapting embedded PostgreSQL environments by reusing PostgresQueryCompiler, PostgresAdapter, and PostgresIntrospector.
- **Where**: `src/dialect/pglite/`, `src/dialect/mod.rs`
- **Depends on**: TASK-DIA-001
- **Done when**: PGlite dialect correctly delegates to PostgreSQL compiler and adapter.
- **Gate**: `cargo test test_pglite_dialect` — Passed

### [x] TASK-DIA-005: Multi-Dialect End-to-End Integration Suite & Final Validation
- **REQ**: REQ-DIA-001..007
- **Symbols**: Public exports in `src/lib.rs`
- **What**: Comprehensive integration tests across all 5 dialects (Postgres, MySQL, MSSQL, SQLite, PGlite) validating compilation, locking, introspection, clippy, and target footprint.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-DIA-001..004
- **Done when**: Full test suite passes across all dialects, zero warnings, footprint verified.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings` — Passed
