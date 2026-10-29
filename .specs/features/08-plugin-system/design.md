# Design: 08 — Plugin Architecture & Built-in Plugins

## Architecture Overview

The Kysely plugin architecture enables non-intrusive interception and rewriting of queries at the AST level before compilation, and query results after driver execution.

```
┌────────────────────────────────────────────────────────┐
│                        Kysely                          │
│               (with_plugin(plugin))                    │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│                     QueryExecutor                      │
│                                                        │
│  1. AST RootNode                                       │
│         │                                              │
│         ▼                                              │
│     [Plugin 1.transform_query]                         │
│         │                                              │
│         ▼                                              │
│     [Plugin 2.transform_query]                         │
│         │                                              │
│         ▼                                              │
│  2. Compile Query (QueryCompiler)                      │
│         │                                              │
│         ▼                                              │
│  3. Driver Execution (execute_query)                   │
│         │                                              │
│         ▼                                              │
│  4. QueryResult                                        │
│         │                                              │
│         ▼                                              │
│     [Plugin 2.transform_result]                        │
│         │                                              │
│         ▼                                              │
│     [Plugin 1.transform_result]                        │
│         │                                              │
│         ▼                                              │
│  5. Final QueryResult to Caller                        │
└────────────────────────────────────────────────────────┘
```

## Key Components

### 1. `KyselyPlugin` Trait (`src/plugin/plugin.rs`)
- `KyselyPlugin`:
  - `fn transform_query(&self, node: Arc<OperationNode>, query_id: QueryId) -> Arc<OperationNode>`
  - `async fn transform_result(&self, result: QueryResult, query_id: QueryId) -> Result<QueryResult, DriverError>`
  - Default implementations pass through without modification.
- Dyn-compatible and thread-safe (`Send + Sync + 'static`).

### 2. Built-in Plugins (`src/plugin/`)
- `CamelCasePlugin`:
  - Transforms camelCase column/table/reference names to snake_case in query AST using `OperationNodeTransformer`.
  - Transforms snake_case column names back to camelCase in `QueryResult` rows.
  - Zero external macro dependencies, fast inline byte-level ASCII manipulation.
- `DeduplicateJoinsPlugin`:
  - Analyzes joins on `SelectQueryNode` and removes identical join operations added dynamically.
- `WithSchemaPlugin`:
  - Adds schema prefixing to unqualified tables in queries (e.g. `users` -> `"public"."users"`).

### 3. Query Executor & Client Integration
- `Kysely::with_plugin(&self, plugin: impl KyselyPlugin + 'static) -> Kysely`
- Chaining plugins on `DefaultQueryExecutor` and query builders.
