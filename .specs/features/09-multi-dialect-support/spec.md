# Spec: 09 — Multi-Dialect Support: MySQL, MSSQL, SQLite & PGlite

## Summary
Extends `rust-kysely` beyond the initial PostgreSQL baseline to achieve 100% parity with all 5 built-in dialects of Kysely (PostgreSQL, MySQL, Microsoft SQL Server / MSSQL, SQLite, and PGlite). Implements dialect-specific AST compilers, query adapters, introspectors, and driver connectors, conditionally enabled via Cargo feature gates (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`) to maintain zero unnecessary dependency bloat and strict footprint control.

## Requirements
- REQ-DIA-001: Implement `MysqlQueryCompiler` generating `?` parameter placeholders, backtick identifier quotes (`` `table`.`col` ``), MySQL pagination (`LIMIT offset, count`), and `ON DUPLICATE KEY UPDATE`.
- REQ-DIA-002: Implement `MssqlQueryCompiler` generating `@p1, @p2, ...` placeholders, bracket identifier quotes (`[table].[col]`), `TOP (n)` select modifiers, and `OUTPUT` clauses for mutation results.
- REQ-DIA-003: Implement `SqliteQueryCompiler` generating `?` placeholders, double quotes (`"table"."col"`), SQLite 3.35+ `RETURNING` clauses, and PRAGMA-based operations.
- REQ-DIA-004: Implement `PGliteDialect` reusing the `PostgresQueryCompiler` and `PostgresAdapter` in embedded, in-memory, or WASM environments.
- REQ-DIA-005: Feature-gate all drivers and dialect dependencies in `Cargo.toml` (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`), allowing users to compile only the dialects they actually use.
- REQ-DIA-006: Implement dialect-specific adapters (`MysqlAdapter`, `MssqlAdapter`, `SqliteAdapter`) declaring dialect capabilities (`supportsReturning`, `supportsTransactionalDdl`, `acquireMigrationLock`).
- REQ-DIA-007: Implement database introspectors (`MysqlIntrospector`, `MssqlIntrospector`, `SqliteIntrospector`) extracting table, column, and constraint metadata via dialect-specific catalog queries (e.g. `PRAGMA table_info` for SQLite, `INFORMATION_SCHEMA` for MySQL/MSSQL).

## Affected Components
- `src/dialect/mysql/` — `MysqlDialect`, `MysqlQueryCompiler`, `MysqlAdapter`, `MysqlIntrospector`
- `src/dialect/mssql/` — `MssqlDialect`, `MssqlQueryCompiler`, `MssqlAdapter`, `MssqlIntrospector`
- `src/dialect/sqlite/` — `SqliteDialect`, `SqliteQueryCompiler`, `SqliteAdapter`, `SqliteIntrospector`
- `src/dialect/pglite/` — `PGliteDialect`
- `Cargo.toml` — Feature gates for each dialect

## Out of Scope
- Oracle or DB2 (community dialect scope, not part of Kysely's 5 official built-in dialects).

## Open Questions
- None.
