<div align="center">
  <img src="docs/assets/social-preview.png" alt="rust-kysely" width="100%" />
</div>

<br/>

<div align="center">
  <h1>rust-kysely</h1>
  <p><strong>Pure Rust, type-safe SQL query builder and runner faithfully ported from Kysely (TypeScript) without macros.</strong></p>
</div>

<br/>

<div align="center">
  <a href="https://github.com/leandroluk/rust-kysely/actions">
    <img src="https://github.com/leandroluk/rust-kysely/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI Build Status" />
  </a>
  <a href="https://docs.rs/rust-kysely">
    <img src="https://docs.rs/rust-kysely/badge.svg" alt="Reference / Docs.rs" />
  </a>
  <a href="https://crates.io/crates/rust-kysely">
    <img src="https://img.shields.io/crates/v/rust-kysely.svg?style=flat-square" alt="Version" />
  </a>
  <a href="LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License" />
  </a>
  <img src="https://img.shields.io/badge/Pure%20Rust-100%25-brightgreen.svg" alt="Pure Rust" />
  <img src="https://img.shields.io/badge/Macros-0%25%20(No%20syn%2Fquote)-orange.svg" alt="No Macros" />
</div>

<br/>

📖 **[Read the Documentation](https://leandroluk.github.io/rust-kysely)**

---

## Contents
- [Getting Started](#getting-started)
- [Why rust-kysely?](#why-rust-kysely)
- [Architecture Blueprint](#architecture-blueprint)
- [Dialect Matrix](#dialect-matrix)
- [Feature Highlights](#feature-highlights)
- [Documentation & Guides](#documentation--guides)
- [Contributing & Testing](#contributing--testing)
- [License](#license)

---

## Getting Started

Add `rust-kysely` to your `Cargo.toml`:

```toml
[dependencies]
rust-kysely = { version = "0.1", features = ["postgres"] }
tokio = { version = "1.0", features = ["full"] }
```

### Fluent Query Building

```rust
use rust_kysely::dialect::postgres::PostgresDialect;
use rust_kysely::Kysely;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dialect = PostgresDialect::from_connection_string("host=localhost user=postgres dbname=app")?;
    let db = Kysely::new(dialect);

    let users = db
        .select_from("users")
        .select(["id", "name", "email"])
        .where_("is_active", "=", true)
        .order_by("created_at", "DESC")
        .limit(10)
        .execute()
        .await?;

    for row in users.rows {
        let name: String = row.get("name")?;
        let email: String = row.get("email")?;
        println!("User: {} <{}>", name, email);
    }

    Ok(())
}
```

---

## Why rust-kysely?

TypeScript's [Kysely](https://kysely.dev) provides one of the cleanest developer experiences in modern web programming: intuitive query building, type-safety, and minimal surprises.

In Rust:
- Heavy ORMs like **Diesel** or **SeaORM** rely on procedural macro generation (`syn`/`quote`) which increases compile times and complicates debugging.
- Low-level drivers like **SQLx** excel at execution but lack dynamic composition, runtime AST transformations, database introspection, and cross-dialect portability.

`rust-kysely` delivers the **exact architectural semantics of Kysely** in **pure, macro-free Rust**:
- **0% procedural macros**: No `proc-macro`, no `syn`, no `quote`.
- **Fast compiles**: Pure runtime AST trees with Copy-on-Write (`Cow<'static, str>`) and stack optimizations (`CompactString`, `SmallVec`).
- **Extensible**: Full plugin pipeline (`CamelCasePlugin`, `DeduplicateJoinsPlugin`, `WithSchemaPlugin`) and schema introspection.

---

## Architecture Blueprint

```text
+-----------------------------------------------------------+
|                       Kysely Instance                     |
+-----------------------------------------------------------+
         |                       |                     |
         v                       v                     v
  +--------------+        +--------------+      +--------------+
  | QueryBuilder |        | SchemaModule |      |   Migrator   |
  +--------------+        +--------------+      +--------------+
         |                       |                     |
         +-----------------------+---------------------+
                                 |
                                 v
                       +-------------------+
                       |   OperationNode   | (AST Tree)
                       +-------------------+
                                 |
            +--------------------+--------------------+
            | (Pre-compilation)                       | (Compilation)
            v                                         v
   +-----------------+                       +-----------------+
   |  KyselyPlugin   |                       |  QueryCompiler  |
   | (e.g. CamelCase)|                       | (Postgres/MySQL)|
   +-----------------+                       +-----------------+
            |                                         |
            +--------------------+--------------------+
                                 v
                       +-------------------+
                       |   CompiledQuery   | (SQL + Parameters)
                       +-------------------+
                                 |
                                 v
                       +-------------------+
                       |   QueryExecutor   |
                       +-------------------+
                                 |
                                 v
                       +-------------------+
                       |      Driver       | (Database Connection)
                       +-------------------+
```

---

## Dialect Matrix

| Feature | PostgreSQL | MySQL / MariaDB | MSSQL (SQL Server) | SQLite | PGlite |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Parameter Placeholder** | `$1, $2` | `?` | `@p1, @p2` | `?` | `$1, $2` |
| **Identifier Quote** | `"col"` | `` `col` `` | `[col]` | `"col"` | `"col"` |
| **RETURNING Clause** | ✅ `RETURNING *` | ❌ (emulates via auto_id) | ✅ `OUTPUT INSERTED.*` | ✅ `RETURNING *` | ✅ `RETURNING *` |
| **Upsert Syntax** | `ON CONFLICT (...)` | `ON DUPLICATE KEY UPDATE` | `MERGE` / `ON CONFLICT` | `ON CONFLICT (...)` | `ON CONFLICT (...)` |
| **Advisory Locks** | `pg_advisory_lock` | `GET_LOCK` | `sp_getapplock` | Tx isolation | `pg_advisory_lock` |
| **Schema Introspection** | `information_schema` | `information_schema` | `sys.tables` | `sqlite_master` | `information_schema` |

---

## Documentation & Guides

Comprehensive guides and runnable examples are available in the [docs](docs/) directory:

- [Getting Started Guide](docs/guides/getting-started.md)
- [Query Builders & Raw SQL](docs/guides/query-builders.md)
- [Schema DDL & Introspection](docs/guides/schema-and-ddl.md)
- [Migrations & Concurrency Locks](docs/guides/migrations-and-locks.md)
- [Plugins & Interceptors](docs/guides/plugins-and-interceptors.md)
- [Multi-Dialect Guide](docs/guides/multi-dialect-guide.md)
- [Web API CRUD (Axum Example)](docs/examples/web-api-crud.md)
- [Embedded SQLite & In-Memory](docs/examples/embedded-sqlite-migrations.md)

---

## Contributing & Testing

Run the test suite across all features:

```bash
cargo test --all-features
```

Run lints and formatting checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
