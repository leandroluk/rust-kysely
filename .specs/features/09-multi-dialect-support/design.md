# Design: 09 — Multi-Dialect Support: MySQL, MSSQL, SQLite & PGlite

## Architecture Overview

Kysely officially supports 5 built-in database dialects:
1. **PostgreSQL** (`PostgresDialect`, `PostgresQueryCompiler`, `PostgresAdapter`, `PostgresIntrospector`)
2. **MySQL** (`MysqlDialect`, `MysqlQueryCompiler`, `MysqlAdapter`, `MysqlIntrospector`)
3. **Microsoft SQL Server / MSSQL** (`MssqlDialect`, `MssqlQueryCompiler`, `MssqlAdapter`, `MssqlIntrospector`)
4. **SQLite** (`SqliteDialect`, `SqliteQueryCompiler`, `SqliteAdapter`, `SqliteIntrospector`)
5. **PGlite** (`PGliteDialect` — lightweight embedded / in-process PostgreSQL)

Each dialect implements:
- `QueryCompiler`: SQL formatting with dialect-specific identifier quoting (double quotes `"` for Postgres/SQLite, backticks `` ` `` for MySQL, brackets `[...]` for MSSQL), parameter styles (`$1` for Postgres, `?` for MySQL/SQLite, `@p1` for MSSQL), and syntax extensions (`ON DUPLICATE KEY UPDATE` in MySQL, `TOP` / `OUTPUT` in MSSQL, `RETURNING` in Postgres/SQLite).
- `DialectAdapter`: Concurrency and migration locks (PostgreSQL `pg_advisory_lock`, MySQL `GET_LOCK` / `RELEASE_LOCK`, MSSQL `sp_getapplock` / `sp_releaseapplock`, SQLite no-op / file lock).
- `DatabaseIntrospector`: Catalog and metadata queries (`information_schema` for Postgres/MySQL/MSSQL, `PRAGMA table_info` / `PRAGMA foreign_key_list` for SQLite).
- `Dialect`: Facade creating the driver, query compiler, and adapter.

```
                  ┌───────────────────────────────┐
                  │         Dialect Trait         │
                  │   create_driver()             │
                  │   create_query_compiler()     │
                  │   create_adapter()            │
                  │   create_introspector()       │
                  └───────────────┬───────────────┘
                                  │
         ┌────────────────┬───────┴────────┬───────────────┬───────────────┐
         ▼                ▼                ▼               ▼               ▼
  PostgresDialect   MysqlDialect     MssqlDialect    SqliteDialect   PGliteDialect
  - Compiler ($1)   - Compiler (?)   - Compiler (@p) - Compiler (?)  - (Reuses PG)
  - pg_advisory     - GET_LOCK       - sp_getapplock - DefaultLock   - PostgresAdapter
  - pg_catalog      - info_schema    - sys/schema    - PRAGMA        - PostgresIntrospector
```

## Key Components

### 1. `MysqlQueryCompiler`, `MysqlAdapter`, `MysqlIntrospector` (`src/dialect/mysql/`)
- Identifier quoting: `` `name` ``
- Parameter style: `?`
- ON DUPLICATE KEY UPDATE syntax compilation for upserts
- `MysqlAdapter`: Uses `GET_LOCK('kysely_migration_lock', 60)` and `RELEASE_LOCK('kysely_migration_lock')`
- `MysqlIntrospector`: Queries `information_schema.tables` and `information_schema.columns`

### 2. `MssqlQueryCompiler`, `MssqlAdapter`, `MssqlIntrospector` (`src/dialect/mssql/`)
- Identifier quoting: `[name]`
- Parameter style: `@p1, @p2, ...`
- TOP (n) pagination and OUTPUT inserted.* / deleted.* clause generation
- `MssqlAdapter`: Uses `sp_getapplock` and `sp_releaseapplock`
- `MssqlIntrospector`: Queries `INFORMATION_SCHEMA.TABLES` and `INFORMATION_SCHEMA.COLUMNS`

### 3. `SqliteQueryCompiler`, `SqliteAdapter`, `SqliteIntrospector` (`src/dialect/sqlite/`)
- Identifier quoting: `"name"`
- Parameter style: `?`
- SQLite 3.35+ `RETURNING` clause support
- `SqliteAdapter`: No-op advisory lock (SQLite uses single-writer database file locks)
- `SqliteIntrospector`: Queries `PRAGMA table_info(table_name)` and `sqlite_master`

### 4. `PGliteDialect` (`src/dialect/pglite/`)
- Reuses `PostgresQueryCompiler` and `PostgresAdapter`
- Provides an embedded PostgreSQL runner abstraction

### 5. Cargo Feature Gates
- Preserves clean zero-bloat discipline: `postgres`, `mysql`, `mssql`, `sqlite`, `pglite`.
