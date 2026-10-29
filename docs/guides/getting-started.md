# Getting Started with rust-kysely

This guide covers setting up `rust-kysely`, connecting to your database, running queries, and managing transactions.

---

## 1. Installation & Feature Flags

In your `Cargo.toml`:

```toml
[dependencies]
# Standard setup with PostgreSQL
rust-kysely = { version = "0.1", features = ["postgres"] }
tokio = { version = "1.0", features = ["full"] }
```

### Available Features

| Feature | Description |
| :--- | :--- |
| `postgres` *(default)* | Tokio-postgres driver and PostgreSQL dialect compiler |
| `sqlite` *(default)* | SQLite dialect compiler and introspector |
| `mysql` *(default)* | MySQL dialect compiler and introspector |
| `mssql` *(default)* | Microsoft SQL Server (T-SQL) compiler and introspector |
| `pglite` *(default)* | WASM / in-memory PostgreSQL support |
| `json` *(default)* | `serde` and `serde_json` support for structured JSON columns |

---

## 2. Initializing Kysely

Create a dialect matching your target database and wrap it in `Kysely`:

```rust
use rust_kysely::dialect::postgres::PostgresDialect;
use rust_kysely::Kysely;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dialect = PostgresDialect::from_connection_string("host=127.0.0.1 user=postgres password=secret dbname=dev")?;
    let db = Kysely::new(dialect);

    // db is thread-safe and cloneable (Arc-backed)
    Ok(())
}
```

---

## 3. Running Queries

### Select

```rust
let rows = db
    .select_from("orders")
    .select(["id", "total_cents", "status"])
    .filter("status", "=", "completed")
    .filter("total_cents", ">", 5000)
    .order_by("created_at", "DESC")
    .limit(20)
    .execute()
    .await?;

for row in rows.rows {
    let id: i64 = row.get("id")?;
    let total: i64 = row.get("total_cents")?;
    println!("Order #{}: ${:.2}", id, total as f64 / 100.0);
}
```

> [!TIP]
> **Filtering: `.filter(...)` vs `.where(...)`**  
> Since `where` is a reserved language keyword in Rust, `rust-kysely` offers `.filter(...)` as the clean and idiomatic Rust standard. If you prefer exact Kysely naming, you can also use `.r#where(...)` or `.where_(...)`.

### Insert

```rust
let res = db
    .insert_into("users")
    .value("name", "Alice")
    .value("email", "alice@example.com")
    .returning(["id", "created_at"])
    .execute_take_first()
    .await?;

if let Some(row) = res {
    let new_id: i64 = row.get("id")?;
    println!("Created user with id {}", new_id);
}
```

### Update

```rust
let updated_count = db
    .update_table("users")
    .set("is_active", false)
    .where_("last_login_at", "<", "2025-01-01")
    .execute()
    .await?
    .num_affected_rows;

println!("Deactivated {} accounts", updated_count);
```

### Delete

```rust
db.delete_from("sessions")
    .where_("expires_at", "<", "NOW()")
    .execute()
    .await?;
```

---

## 4. Transactions & Savepoints

`rust-kysely` supports nested transactions and automatic rollback on failure:

```rust
db.transaction(|trx| Box::pin(async move {
    // 1. Debit account A
    trx.update_table("accounts")
        .set("balance", 900)
        .where_("id", "=", 1)
        .execute()
        .await?;

    // 2. Credit account B
    trx.update_table("accounts")
        .set("balance", 1100)
        .where_("id", "=", 2)
        .execute()
        .await?;

    // If an error is returned here, trx automatically rolls back
    Ok(())
})).await?;
```
