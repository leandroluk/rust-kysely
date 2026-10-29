# Migrations & Advisory Locks

`rust-kysely` features an enterprise-grade migration engine capable of tracking applied schema versions, rolling forward or backward, and preventing concurrent migration races across distributed services.

---

## Migrator and Providers

A migration in `rust-kysely` implements the `Migration` trait:

```rust
use rust_kysely::migration::{Migration, Migrator, SimpleMigrationProvider};
use rust_kysely::Kysely;
use async_trait::async_trait;

struct Migration001;

#[async_trait]
impl Migration for Migration001 {
    async fn up(&self, db: &Kysely) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.schema()
            .create_table("users")
            .add_column("id", "bigserial", |c| c.primary_key())
            .add_column("email", "varchar(255)", |c| c.not_null().unique())
            .execute()
            .await?;
        Ok(())
    }

    async fn down(&self, db: &Kysely) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.schema().drop_table("users").execute().await?;
        Ok(())
    }
}
```

### Running Migrations

```rust
let mut provider = SimpleMigrationProvider::new();
provider.add_migration("001_create_users", Box::new(Migration001));

let migrator = Migrator::new(db.clone(), provider);

// 1. Run all pending migrations
let result = migrator.migrate_to_latest().await?;
println!("Applied: {:?}", result.results);

// 2. Step down by 1 migration
let down_result = migrator.migrate_down().await?;

// 3. Roll back completely
use rust_kysely::migration::NO_MIGRATIONS;
migrator.migrate_to(NO_MIGRATIONS).await?;
```

---

## Advisory Lock Concurrency

When running microservices or serverless replicas, multiple instances might spin up and attempt to run migrations simultaneously.

`rust-kysely` prevents duplicate migration runs via database-native advisory locks:

- **PostgreSQL**: `pg_advisory_lock(3853314791062309107)` (exact lock ID used by TypeScript Kysely).
- **MySQL**: `GET_LOCK('kysely_migration_lock', 60)`.
- **MSSQL**: `sp_getapplock @Resource = 'kysely_migration_lock', @LockMode = 'Exclusive'`.
- **SQLite**: Automatic transaction locking.

The lock is automatically acquired before checking migration tables and released in a `finally` block when done.

---

## File Migrations

You can also load SQL migration files directly from the filesystem using `FileMigrationProvider`:

Given a directory structure:
```text
migrations/
  ├── 001_create_users.up.sql
  ├── 001_create_users.down.sql
  ├── 002_create_posts.up.sql
  └── 002_create_posts.down.sql
```

Load and execute:

```rust
use rust_kysely::migration::{FileMigrationProvider, Migrator};
use std::path::Path;

let provider = FileMigrationProvider::from_folder(Path::new("./migrations"));
let migrator = Migrator::new(db, provider);

migrator.migrate_to_latest().await?;
```
