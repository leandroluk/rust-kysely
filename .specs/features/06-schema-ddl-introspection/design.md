# Design: 06 — Schema DDL Builders & PostgreSQL Introspection

## Architecture Overview

The DDL and schema introspection system brings complete type-safe schema definition and migration capabilities to `rust-kysely`. Modeled after Kysely TypeScript's `SchemaModule` and `PostgresIntrospector`, it enables programmatic creation, alteration, and deletion of tables, columns, constraints, indexes, views, and custom PostgreSQL types (enums), followed by runtime schema inspection.

```
┌────────────────────────────────────────────────────────┐
│                      db.schema()                       │
│                     (SchemaModule)                     │
└──────────┬─────────────────┬──────────────────┬────────┘
           │                 │                  │
           ▼                 ▼                  ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│CreateTableBuilder│ │ AlterTableBuilder│ │CreateIndexBuilder│
│(ColumnDefinition)│ │  DropTableBuilder│ │ CreateViewBuilder│
└──────────┬───────┘ └─────────┬────────┘ └────────┬─────────┘
           │                   │                   │
           └───────────────────┼───────────────────┘
                               ▼
┌────────────────────────────────────────────────────────┐
│                   DDL OperationNodes                   │
│ (CreateTableNode, AlterTableNode, CreateIndexNode...)  │
└──────────────────────────────┬─────────────────────────┘
                               ▼
┌────────────────────────────────────────────────────────┐
│               QueryCompiler (DDL Visitor)              │
│       (Compiles AST into CREATE/ALTER/DROP SQL)        │
└──────────────────────────────┬─────────────────────────┘
                               ▼
┌────────────────────────────────────────────────────────┐
│                     QueryExecutor                      │
│                  (.execute().await)                    │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│                  PostgresIntrospector                  │
│       (Queries information_schema & pg_catalog)        │
│       ==> Returns TableMetadata, ColumnMetadata        │
└────────────────────────────────────────────────────────┘
```

## Key Components

### 1. DDL Operation Nodes (`src/operation_node/ddl_nodes.rs`)
- `CreateTableNode`: Table name, columns list, table-level constraints list, `if_not_exists`, `temporary`.
- `ColumnDefinitionNode`: Column name, data type, nullable, unique, primary key, default expression, check expression, foreign key reference.
- `ForeignKeyConstraintNode`: Constraint name, local columns, foreign table, foreign columns, `on_delete`, `on_update`.
- `AlterTableNode`: Table name, list of alter actions (add column, drop column, alter column type, drop constraint, etc.).
- `CreateIndexNode`: Index name, target table, column list, unique flag, using method (`btree`, `gin`, `gist`), partial where filter.
- `CreateViewNode` / `DropViewNode`: View name, select query, `or_replace`, `materialized`.
- `CreateTypeNode` / `DropTypeNode`: Custom enum name, variant list.

### 2. DDL Compilation (`src/query_compiler/`)
- `QueryCompiler` compiles DDL nodes into standard SQL with dialect quoting:
  - `CREATE TABLE "users" ("id" BIGSERIAL PRIMARY KEY, "email" VARCHAR(255) NOT NULL UNIQUE, ...)`
  - `ALTER TABLE "users" ADD COLUMN "age" INT4`
  - `CREATE UNIQUE INDEX "users_email_idx" ON "users" ("email")`
  - `CREATE TYPE "user_status" AS ENUM ('active', 'inactive')`

### 3. Fluent Builders (`src/schema/`)
- `create_table(name).if_not_exists().add_column("id", "serial", |c| c.primary_key())`
- `alter_table("users").add_column("score", "float8", |c| c.default_to(0.0))`
- `drop_table("users").if_exists().cascade()`
- `create_index("idx_users_age").on("users").column("age").where_("age", ">", 18)`
- `create_view("active_users").as(select_query)`
- `create_type("mood").as_enum(["happy", "sad"])`

### 4. Database Introspection (`src/dialect/postgres/introspector.rs`)
- `PostgresIntrospector` executes SQL queries against `information_schema.tables`, `columns`, and `pg_catalog`:
  - `get_tables()` -> `Vec<TableMetadata>`
  - `TableMetadata` holds `name`, `is_view`, `columns: Vec<ColumnMetadata>`
  - `ColumnMetadata` holds `name`, `data_type`, `is_nullable`, `has_default_value`
