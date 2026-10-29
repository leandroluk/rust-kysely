# Query Builders & Raw SQL

`rust-kysely` provides fluent, type-safe builders for all SQL operations.

---

## Select Queries

The `SelectQueryBuilder` lets you compose arbitrarily complex queries:

### Joins & Filtering

```rust
let query = db
    .select_from("users")
    .inner_join("posts", "posts.user_id", "users.id")
    .left_join("profiles", "profiles.user_id", "users.id")
    .select(["users.name", "posts.title", "profiles.avatar_url"])
    .filter("users.is_active", "=", true);

let compiled = query.compile();
println!("SQL: {}", compiled.sql);
```

> [!NOTE]
> **Rust Reserved Keyword: `where`**  
> In TypeScript Kysely, queries use `.where(...)`. Because `where` is a keyword in Rust, `rust-kysely` provides **`.filter(...)`** as the idiomatic choice. You can also write **`.r#where(...)`** (raw identifier) or **`.where_(...)`**.

### Common Table Expressions (CTEs) & Subqueries

```rust
let active_users = db
    .select_from("users")
    .select(["id", "name"])
    .filter("status", "=", "active");

let query = db
    .with("active_users", active_users)
    .select_from("active_users")
    .select(["id", "name"])
    .order_by("name", "ASC");
```

### Set Operations (`UNION`, `INTERSECT`)

```rust
let q1 = db.select_from("customers").select("email");
let q2 = db.select_from("suppliers").select("email");

let unified = q1.union(q2);
```

---

## Insert Queries

### Batch Inserts

```rust
let mut batch = Vec::new();
for i in 1..=100 {
    let mut row = std::collections::HashMap::new();
    row.insert("event_name", format!("event_{}", i).into());
    row.insert("severity", "info".into());
    batch.push(row);
}

db.insert_into("audit_logs")
    .values(batch)
    .execute()
    .await?;
```

### Upsert (`ON CONFLICT` / `ON DUPLICATE KEY`)

```rust
let query = db
    .insert_into("user_preferences")
    .value("user_id", 42)
    .value("theme", "dark")
    .value("notifications_enabled", true)
    .on_conflict(|b| {
        b.column("user_id")
         .do_update_set("theme", "dark")
         .do_update_set("notifications_enabled", true)
    });
```

---

## Update & Delete Queries

### Updates with Complex Filtering

```rust
db.update_table("subscriptions")
    .set("plan", "pro")
    .set("updated_at", "NOW()")
    .where_("user_id", "=", 123)
    .where_("status", "=", "trial")
    .execute()
    .await?;
```

### Deletions with Returning

```rust
let deleted_rows = db
    .delete_from("temp_tokens")
    .where_("expired", "=", true)
    .returning(["token", "user_id"])
    .execute()
    .await?;
```

---

## Raw SQL and Expression Builder

When you need specialized SQL dialect features or custom expressions, `RawBuilder` (`sql`) and `ExpressionBuilder` (`eb`) provide escape hatches without abandoning parameter safety.

### Safe Raw SQL Interpolation

```rust
use rust_kysely::raw_builder::helpers::sql;

let min_age = 21;
let query = sql("SELECT * FROM users WHERE age > ").append(min_age).append(" AND department = ").append("Engineering");
let compiled = query.to_compiled_query();
// Emits parametrized SQL: SELECT * FROM users WHERE age > $1 AND department = $2
```

### Expression Builder (`eb`)

```rust
use rust_kysely::expression::ExpressionBuilder;

let eb = ExpressionBuilder::new();
let expr = eb.and(vec![
    eb.cmp("score", ">=", 90),
    eb.or(vec![
        eb.cmp("role", "=", "admin"),
        eb.cmp("role", "=", "moderator"),
    ]),
]);
```
