# Design: 05 — PostgreSQL Driver, Connection Pooling & Execution Engine

## Architecture Overview

The execution engine connects compiled queries (`CompiledQuery`) with physical database connections. It abstracts driver I/O, manages connection acquisition and release, converts query parameters into wire format, maps raw response rows into strongly-typed `Row` structs, handles transactions and savepoints, and provides high-level execution methods on query builders (`.execute()`, `.execute_take_first()`, `.execute_into::<T>()`).

```
┌────────────────────────────────────────────────────────┐
│             Kysely Client / QueryBuilder               │
│ (.execute(), .execute_take_first(), .execute_into())   │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                  QueryExecutor                         │
│ (Coordinates compilation, plugins, logging, execution) │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│           Driver & DatabaseConnection Traits           │
│ (async acquire_connection, execute_query, release)     │
└──────────────┬──────────────────────────┬──────────────┘
               │                          │
               ▼                          ▼
┌───────────────────────────┐  ┌─────────────────────────┐
│     PostgresDriver        │  │     Mock/MemoryDriver   │
│ (tokio-postgres client)   │  │ (Zero-dependency tests)  │
└───────────────────────────┘  └─────────────────────────┘
```

## Key Components

### 1. Driver & Connection Traits (`src/driver/mod.rs`)
- `DatabaseConnection`: Async trait for executing a `CompiledQuery` and returning a `QueryResult`.
  ```rust
  #[async_trait]
  pub trait DatabaseConnection: Send + Sync {
      async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError>;
  }
  ```
- `Driver`: Async trait for managing connections.
  ```rust
  #[async_trait]
  pub trait Driver: Send + Sync {
      async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError>;
      async fn release_connection(&self, connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError>;
      async fn destroy(&self) -> Result<(), DriverError>;
  }
  ```

### 2. Execution Results & Row Mapping (`src/query_executor/`)
- `QueryResult`: Contains `rows: Vec<Row>`, `num_affected_rows: Option<u64>`, `insert_id: Option<i64>`.
- `Row`: Map-like container mapping column names (as `CompactString`) to `Value` scalars.
  - `.get::<T>(col)` -> extracts strongly-typed value or returns `ColumnError`.
  - `.get_opt::<T>(col)` -> extracts optional value.
  - `.into_record::<T>()` -> deserializes row into serde struct.

### 3. Kysely Top-Level Client (`src/kysely.rs`)
- `Kysely<D: Dialect>` acts as the main facade:
  - `db.select_from(table)`
  - `db.insert_into(table)`
  - `db.update_table(table)`
  - `db.delete_from(table)`
  - `db.transaction().execute(|trx| ...)`
- Query builders store an optional `Arc<dyn QueryExecutor>`, allowing direct chaining:
  - `query.execute().await?`
  - `query.execute_take_first().await?`
  - `query.execute_take_first_or_throw().await?`
  - `query.execute_into::<T>().await?`

### 4. Transactions and Savepoints (`src/transaction.rs`)
- `Transaction`: Wraps an acquired `DatabaseConnection` with automatic rollback on error / abort, and atomic commit on successful closure return.
- Savepoints: `trx.savepoint("sp1")`, `trx.rollback_to_savepoint("sp1")`, `trx.release_savepoint("sp1")`.

### 5. PostgreSQL Driver (`src/dialect/postgres/`)
- `PostgresDriver`: Implements `Driver` using `tokio-postgres::Client`. Converts `kysely::Value` parameters to `tokio-postgres::types::ToSql` dynamically at runtime.
- In-memory mock connection for CI and offline testing without live Postgres container.
