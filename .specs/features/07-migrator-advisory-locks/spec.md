# Spec: 07 — Migrator, Migration Providers & PostgreSQL Advisory Locks

## Summary
Implements schema migration orchestration for `rust-kysely`, ported from upstream `src/migration/` and `src/dialect/postgres/postgres-adapter.ts`. Manages schema version history using an internal tracking table (`kysely_migration`), ensures concurrent execution safety across multiple application instances using PostgreSQL session-level advisory locks (`pg_advisory_lock` / `pg_advisory_unlock`), and provides both file-system and in-memory `MigrationProvider` implementations with transactional execution (`migrate_to_latest`, `migrate_up`, `migrate_down`, `migrate_to`).

## Requirements
- REQ-MIG-001: Implement `Migrator` managing migration execution, lock acquisition, and status reporting (`get_migrations()`).
- REQ-MIG-002: Implement `MigrationProvider` async trait returning a map or list of migrations (`up(&db)`, `down(&db)`).
- REQ-MIG-003: Implement `FileMigrationProvider` discovering and loading SQL or compiled migration files from a directory.
- REQ-MIG-004: Implement in-memory migration provider for unit tests and embedded applications.
- REQ-MIG-005: Implement PostgreSQL distributed lock acquisition in `PostgresAdapter` using `pg_advisory_lock(3853314791062309107)` and timeout configuration (`set_config('lock_timeout', ...)`).
- REQ-MIG-006: Implement safe release of the migration lock using `pg_advisory_unlock(3853314791062309107)` guaranteed via Drop guard or `finally` semantics on error/panic.
- REQ-MIG-007: Implement migration table auto-creation (`kysely_migration` tracking name and execution timestamp) inside a transactional block.
- REQ-MIG-008: Implement `migrate_to_latest()`, `migrate_up()`, `migrate_down()`, and `migrate_to(name)` returning structured `MigrationResult` lists with error reporting on failure.

## Affected Components
- `src/migration/` — `Migrator`, `MigrationProvider`, `FileMigrationProvider`, `MigrationResult`
- `src/dialect/postgres/postgres_adapter.rs` — Advisory lock logic (`acquire_migration_lock`, `release_migration_lock`)

## Out of Scope
- Code generators reading live DB schemas (belongs in external tooling/CLI).
- Non-transactional migrations in PostgreSQL unless explicitly opted out.

## Open Questions
- None.
