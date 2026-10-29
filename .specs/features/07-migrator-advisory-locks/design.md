# Design: 07 — Migrator, Migration Providers & PostgreSQL Advisory Locks

## Architecture Overview

The schema migration system provides robust, transactional database versioning with distributed locking. Ported from Kysely TypeScript (`src/migration/` and `src/dialect/postgres/postgres-adapter.ts`), it orchestrates migration discovery, tracking in a dedicated internal table (`kysely_migration`), transactional execution of `up` and `down` migration steps, and concurrency protection via PostgreSQL advisory locks.

```
┌────────────────────────────────────────────────────────┐
│                        Migrator                        │
│   (migrate_to_latest, migrate_up, migrate_down)        │
└───────────┬────────────────────────────────┬───────────┘
            │                                │
            ▼                                ▼
┌───────────────────────┐        ┌───────────────────────┐
│   MigrationProvider   │        │     DialectAdapter    │
│(File / Memory Provider)│        │   (PostgresAdapter)   │
└───────────┬───────────┘        └───────────┬───────────┘
            │                                │
            │                                ▼
            │                    ┌───────────────────────┐
            │                    │ pg_advisory_lock      │
            │                    │ pg_advisory_unlock    │
            │                    └───────────────────────┘
            ▼
┌────────────────────────────────────────────────────────┐
│                 Transactional Runner                   │
│   1. Ensure kysely_migration table exists              │
│   2. Determine unexecuted / rollback migrations        │
│   3. Execute step (up/down) inside transaction         │
│   4. Record/delete execution in kysely_migration       │
└────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Dialect Adapter & Advisory Locks (`src/dialect/adapter.rs`, `src/dialect/postgres/adapter.rs`)
- `DialectAdapter`: Async trait for dialect-specific capabilities:
  - `acquire_migration_lock(&self, db: &Kysely) -> Result<(), DriverError>`
  - `release_migration_lock(&self, db: &Kysely) -> Result<(), DriverError>`
- `PostgresAdapter`:
  - `pg_advisory_lock(3853314791062309107)` — Kysely upstream standard 64-bit integer lock ID.
  - `pg_advisory_unlock(3853314791062309107)` — Guarantees release even on error.
- `DefaultAdapter`: No-op lock for non-locking dialects / in-memory tests.

### 2. Migration Models & Providers (`src/migration/`)
- `Migration`: Async trait with `up(&self, db: &Kysely)` and `down(&self, db: &Kysely)`.
- `MigrationProvider`: Async trait returning `BTreeMap<String, Box<dyn Migration>>`.
- `SimpleMigrationProvider`: In-memory migration registry for unit testing and embedded apps.
- `FileMigrationProvider`: Discovers and executes `.up.sql` and `.down.sql` files from the filesystem.

### 3. Migrator Engine (`src/migration/migrator.rs`)
- `Migrator`:
  - Initializes tracking table `kysely_migration (name varchar(255) PRIMARY KEY, timestamp varchar(255) NOT NULL)`
  - `get_migrations(&self) -> Result<Vec<MigrationInfo>, DriverError>`
  - `migrate_to_latest(&self) -> MigrationResultSet`
  - `migrate_up(&self) -> MigrationResultSet`
  - `migrate_down(&self) -> MigrationResultSet`
  - `migrate_to(&self, target_name: &str) -> MigrationResultSet`
  - All migrations run inside atomic transactions with automatic rollback on error.
