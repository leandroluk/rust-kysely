# Multi-Dialect Support

`rust-kysely` provides tailored compilers, introspectors, and adapters for all major relational database engines.

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

## PostgreSQL & PGlite

```rust
use rust_kysely::dialect::postgres::PostgresDialect;
use rust_kysely::dialect::pglite::PGliteDialect;

// Native PostgreSQL connection
let pg_db = Kysely::new(PostgresDialect::from_connection_string("postgres://user:pass@localhost/db")?);

// In-Memory or WASM PGlite
let pglite_db = Kysely::new(PGliteDialect::new(mock_driver));
```

---

## MySQL

```rust
use rust_kysely::dialect::mysql::MysqlDialect;

let mysql_db = Kysely::new(MysqlDialect::new(driver));
```

Compiles backtick quotes and `?` positional parameters.

---

## Microsoft SQL Server

```rust
use rust_kysely::dialect::mssql::MssqlDialect;

let mssql_db = Kysely::new(MssqlDialect::new(driver));
```

Compiles square bracket identifiers (`[users].[id]`) and `@p1`, `@p2` parameters.

---

## SQLite

```rust
use rust_kysely::dialect::sqlite::SqliteDialect;

let sqlite_db = Kysely::new(SqliteDialect::new(driver));
```

Supports in-memory databases, file paths, and pragma introspection.
