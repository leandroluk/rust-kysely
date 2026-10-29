# Spec: 05 — PostgreSQL Driver, Connection Pooling & Execution Engine

## Summary
Implements the database execution engine and PostgreSQL dialect driver for `rust-kysely`, ported from upstream `src/driver/`, `src/dialect/postgres/`, and `src/query-executor/`. Bridges compiled queries (`CompiledQuery`) with live database connections via `tokio-postgres`. Introduces the `DatabaseConnection`, `Driver`, and `ConnectionProvider` abstractions, query logging (`LogConfig`), result sets (`QueryResult`, `Row`, `InsertResult`, `UpdateResult`, `DeleteResult`), type-safe value extraction (`row.get::<T>("col")?`, `row.get_opt::<T>("col")?`), direct deserialization (`.execute_into::<T>()`), single-item execution (`execute_take_first()`, `execute_take_first_or_throw()`), and transactional blocks (`db.transaction().execute(|trx| async move { ... })`) with savepoints.

## Requirements
- REQ-EXEC-001: Implement `DatabaseConnection` and `Driver` async traits abstracting raw database communication and query streaming.
- REQ-EXEC-002: Implement `PostgresDriver` wrapping `tokio-postgres::Client` (and connection pools) with parameter type conversion from `kysely::Value` to PostgreSQL wire format.
- REQ-EXEC-003: Implement `PostgresDialect` providing factory methods for `PostgresDriver`, `PostgresQueryCompiler`, `PostgresAdapter`, and `PostgresIntrospector`.
- REQ-EXEC-004: Implement `DefaultQueryExecutor` coordinating plugin transformations (`transform_query`), compilation, execution, logging, and result post-processing (`transform_result`).
- REQ-EXEC-005: Implement `Row` providing column index and name lookup with `.get::<T>(col)` and `.get_opt::<T>(col)` returning descriptive `ColumnError` on type or index mismatches.
- REQ-EXEC-006: Implement `.execute_into::<T>()` on query builders, converting returned rows directly into user domain structs implementing `serde::de::DeserializeOwned`.
- REQ-EXEC-007: Implement `.execute_take_first()` returning `Result<Option<Row>, KyselyError>` and `.execute_take_first_or_throw()` returning `Result<Row, NoResultError>`.
- REQ-EXEC-008: Implement `.execute_take_first_val::<T>()` returning the first scalar value of the first row (e.g. for `SELECT count(*)`).
- REQ-EXEC-009: Implement transaction support: `db.transaction().execute(|trx| async move { ... })` with atomic commit on `Ok` and rollback on `Err` or panic.
- REQ-EXEC-010: Implement savepoint management within transactions (`trx.savepoint("sp1")`, `rollback_to_savepoint`, `release_savepoint`).

## Affected Components
- `src/driver/` — `Driver`, `DatabaseConnection`, `ConnectionProvider`
- `src/dialect/postgres/` — `PostgresDialect`, `PostgresDriver`, `PostgresAdapter`
- `src/query_executor/` — `QueryExecutor`, `DefaultQueryExecutor`, `QueryResult`, `Row`
- `src/kysely.rs` — Main `Kysely` client struct and `Transaction`

## Out of Scope
- Synchronous blocking drivers (all database communication is asynchronous via `tokio`).
- Direct dependencies on specific third-party connection pools outside trait abstraction.

## Open Questions
- None.
