# Schema Management, DDL & Introspection

`rust-kysely` provides programmatic DDL schema builders and database metadata introspection.

---

## Table Management

Access the schema module via `db.schema()`:

### Create Table

```rust
db.schema()
    .create_table("users")
    .if_not_exists()
    .add_column("id", "bigserial", |col| col.primary_key())
    .add_column("email", "varchar(255)", |col| col.not_null().unique())
    .add_column("age", "integer", |col| col.check("age >= 0"))
    .add_column("created_at", "timestamptz", |col| col.default_to("NOW()"))
    .execute()
    .await?;
```

### Foreign Keys & Table Constraints

```rust
db.schema()
    .create_table("orders")
    .add_column("id", "bigserial", |col| col.primary_key())
    .add_column("user_id", "bigint", |col| col.not_null())
    .add_column("amount", "decimal(12,2)", |col| col.not_null())
    .add_foreign_key_constraint("fk_orders_user", &["user_id"], "users", &["id"], |fk| {
        fk.on_delete("CASCADE").on_update("CASCADE")
    })
    .execute()
    .await?;
```

### Alter Table

```rust
db.schema()
    .alter_table("users")
    .add_column("avatar_url", "text", |col| col.nullable())
    .drop_column("deprecated_column")
    .rename_column("age", "user_age")
    .execute()
    .await?;
```

### Drop Table

```rust
db.schema()
    .drop_table("temp_cache")
    .if_exists()
    .cascade()
    .execute()
    .await?;
```

---

## Indexes, Views and Types

### Indexes

```rust
// Create unique index
db.schema()
    .create_index("idx_users_email")
    .unique()
    .on("users")
    .column("email")
    .execute()
    .await?;

// Drop index
db.schema()
    .drop_index("idx_users_email")
    .if_exists()
    .execute()
    .await?;
```

### Views

```rust
let view_query = db
    .select_from("orders")
    .select(["user_id", "amount"])
    .where_("amount", ">", 100);

db.schema()
    .create_view("high_value_orders")
    .or_replace()
    .as_(view_query)
    .execute()
    .await?;
```

---

## Introspection

Query real-time database schema information:

```rust
let introspector = db.introspection();

// List all schemas
let schemas = introspector.get_schemas().await?;
println!("Schemas: {:?}", schemas);

// List all tables and inspect columns
let tables = introspector.get_tables().await?;
for table in tables {
    println!("Table: {} (is_view: {})", table.name, table.is_view);
    for col in table.columns {
        println!("  - {}: {} (nullable: {}, default: {})",
            col.name, col.data_type, col.is_nullable, col.has_default_value);
    }
}
```
