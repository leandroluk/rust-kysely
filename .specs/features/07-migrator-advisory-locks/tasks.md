# Tasks: 07 — Migrator, Migration Providers & PostgreSQL Advisory Locks

### [x] TASK-MIG-001: Implement DialectAdapter, PostgresAdapter, and Advisory Locking
- **REQ**: REQ-MIG-005, REQ-MIG-006
- **Symbols**: `DialectAdapter`, `PostgresAdapter`, `DefaultAdapter`, `pg_advisory_lock`, `pg_advisory_unlock`
- **What**: Implement dialect adapters with distributed PostgreSQL advisory locking using the upstream Kysely lock ID `3853314791062309107`.
- **Where**: `src/dialect/adapter.rs`, `src/dialect/postgres/adapter.rs`, `src/dialect/mod.rs`
- **Depends on**: Milestone 6
- **Done when**: Lock and unlock commands execute valid advisory lock SQL and release properly.
- **Gate**: `cargo test test_postgres_advisory_lock` — Passed

### [x] TASK-MIG-002: Implement Migration, MigrationProvider, and SimpleMigrationProvider
- **REQ**: REQ-MIG-002, REQ-MIG-004
- **Symbols**: `Migration`, `MigrationProvider`, `SimpleMigrationProvider`, `MigrationInfo`
- **What**: Implement async migration trait and in-memory provider for unit tests and embedded applications.
- **Where**: `src/migration/traits.rs`, `src/migration/simple_provider.rs`, `src/migration/mod.rs`
- **Depends on**: TASK-MIG-001
- **Done when**: In-memory migrations can be registered, listed, and called with `up` and `down`.
- **Gate**: `cargo test test_simple_migration_provider` — Passed

### [x] TASK-MIG-003: Implement FileMigrationProvider for disk migrations
- **REQ**: REQ-MIG-003
- **Symbols**: `FileMigrationProvider`, `from_folder()`
- **What**: Implement file system migration provider discovering and parsing `.up.sql` and `.down.sql` migrations.
- **Where**: `src/migration/file_provider.rs`, `src/migration/mod.rs`
- **Depends on**: TASK-MIG-002
- **Done when**: Directory with `.sql` files discovers and parses migration pairs in lexicographical order.
- **Gate**: `cargo test test_file_migration_provider` — Passed

### [x] TASK-MIG-004: Implement Migrator engine and state tracking
- **REQ**: REQ-MIG-001, REQ-MIG-007, REQ-MIG-008
- **Symbols**: `Migrator`, `MigratorConfig`, `migrate_to_latest()`, `migrate_up()`, `migrate_down()`, `migrate_to()`
- **What**: Implement migrator engine managing `kysely_migration` tracking table, lock acquisition, transactional step execution, and status reporting.
- **Where**: `src/migration/migrator.rs`, `src/migration/mod.rs`
- **Depends on**: TASK-MIG-001, TASK-MIG-002, TASK-MIG-003
- **Done when**: `migrate_to_latest`, `migrate_up`, `migrate_down` and `migrate_to` execute migrations transactionally and update tracking state.
- **Gate**: `cargo test test_migrator_execution` — Passed

### [x] TASK-MIG-005: Comprehensive integration tests for Migrator & Advisory Locks
- **REQ**: REQ-MIG-001..008
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests verifying migration lifecycle, lock acquisition/release, error rollback, and status tracking.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-MIG-004
- **Done when**: Full test suite passes, zero warnings, footprint clean.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings` — Passed
