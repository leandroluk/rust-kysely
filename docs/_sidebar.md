- **Getting Started**
  - [Overview](README.md)
  - [Quickstart](guides/getting-started.md)
  - [Why rust-kysely?](README.md#why-rust-kysely)
  - [TypeScript vs Rust Architecture](README.md#typescript-kysely-vs-rust-kysely)

- **Query Builders**
  - [Select, Joins & CTEs](guides/query-builders.md#select-queries)
  - [Insert & On Conflict (Upsert)](guides/query-builders.md#insert-queries)
  - [Update & Delete](guides/query-builders.md#update--delete-queries)
  - [Raw SQL & Expressions](guides/query-builders.md#raw-sql-and-expression-builder)

- **Schema & DDL**
  - [Schema Module & Tables](guides/schema-and-ddl.md#table-management)
  - [Indexes, Views & Types](guides/schema-and-ddl.md#indexes-views-and-types)
  - [Database Introspection](guides/schema-and-ddl.md#introspection)

- **Migrations & Concurrency**
  - [Migrator & Providers](guides/migrations-and-locks.md#migrator-and-providers)
  - [PostgreSQL Advisory Locks](guides/migrations-and-locks.md#advisory-lock-concurrency)
  - [File Migration Provider](guides/migrations-and-locks.md#file-migrations)

- **Plugins & Interceptors**
  - [Plugin Architecture](guides/plugins-and-interceptors.md#plugin-lifecycle)
  - [CamelCasePlugin](guides/plugins-and-interceptors.md#camelcaseplugin)
  - [DeduplicateJoinsPlugin](guides/plugins-and-interceptors.md#deduplicatejoinsplugin)
  - [WithSchemaPlugin](guides/plugins-and-interceptors.md#withschemaplugin)
  - [Custom Plugins](guides/plugins-and-interceptors.md#authoring-custom-plugins)

- **Multi-Dialect Support**
  - [PostgreSQL & PGlite](guides/multi-dialect-guide.md#postgresql--pglite)
  - [MySQL & MariaDB](guides/multi-dialect-guide.md#mysql)
  - [MSSQL (SQL Server)](guides/multi-dialect-guide.md#microsoft-sql-server)
  - [SQLite](guides/multi-dialect-guide.md#sqlite)

- **Examples**
  - [Web API CRUD (Axum / Tokio)](examples/web-api-crud.md)
  - [Embedded SQLite & In-Memory](examples/embedded-sqlite-migrations.md)
