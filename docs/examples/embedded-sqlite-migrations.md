# Embedded SQLite & In-Memory Migrations

This example illustrates using `rust-kysely` with SQLite for local desktop applications, embedded devices, or integration testing.

---

## Embedded SQLite Setup

```rust
use rust_kysely::dialect::sqlite::SqliteDialect;
use rust_kysely::migration::{Migration, Migrator, SimpleMigrationProvider};
use rust_kysely::Kysely;
use async_trait::async_trait;

struct CreateSchemaMigration;

#[async_trait]
impl Migration for CreateSchemaMigration {
    async fn up(&self, db: &Kysely) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.schema()
            .create_table("settings")
            .if_not_exists()
            .add_column("key", "text", |c| c.primary_key())
            .add_column("value", "text", |c| c.not_null())
            .execute()
            .await?;
        Ok(())
    }

    async fn down(&self, db: &Kysely) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.schema().drop_table("settings").execute().await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize SQLite dialect with driver
    let dialect = SqliteDialect::new(driver);
    let db = Kysely::new(dialect);

    // 2. Setup migrations
    let mut provider = SimpleMigrationProvider::new();
    provider.add_migration("001_initial_settings", Box::new(CreateSchemaMigration));

    let migrator = Migrator::new(db.clone(), provider);
    migrator.migrate_to_latest().await?;

    // 3. Insert and query configuration
    db.insert_into("settings")
        .value("key", "theme")
        .value("value", "nord")
        .execute()
        .await?;

    let row = db
        .select_from("settings")
        .select(["key", "value"])
        .where_("key", "=", "theme")
        .execute_take_first()
        .await?
        .expect("setting found");

    println!("Theme: {}", row.get::<String>("value")?);

    Ok(())
}
```
