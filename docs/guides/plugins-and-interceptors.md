# Plugins & Query Interceptors

The `rust-kysely` plugin system allows rewriting query ASTs (`OperationNode`) before compilation and post-processing database results (`QueryResult`) before returning to the caller.

---

## Plugin Lifecycle

Plugins implement the `KyselyPlugin` trait:

```rust
use async_trait::async_trait;
use rust_kysely::plugin::KyselyPlugin;
use rust_kysely::operation_node::OperationNode;
use rust_kysely::query_executor::row::QueryResult;
use rust_kysely::util::query_id::QueryId;
use rust_kysely::driver::DriverError;

#[async_trait]
pub trait KyselyPlugin: Send + Sync {
    /// Transforms the query AST before compilation.
    fn transform_query(&self, args: PluginTransformArgs) -> OperationNode;

    /// Transforms the execution result before returning to the caller.
    async fn transform_result(
        &self,
        args: PluginTransformResultArgs,
    ) -> Result<QueryResult, DriverError> {
        Ok(args.result)
    }
}
```

---

## Built-in Plugins

### CamelCasePlugin

Transforms snake_case database columns into camelCase identifiers in memory, and converts camelCase queries into snake_case SQL.

```rust
use rust_kysely::plugin::CamelCasePlugin;

let db = db.with_plugin(CamelCasePlugin::new());

// Automatically translates "first_name" -> "firstName" in results
```

### DeduplicateJoinsPlugin

Ensures that if multiple dynamic query fragments or subroutines attach the same `JOIN`, it is only emitted once into the compiled SQL.

```rust
use rust_kysely::plugin::DeduplicateJoinsPlugin;

let db = db.with_plugin(DeduplicateJoinsPlugin::new());
```

### WithSchemaPlugin

Prepends a default schema name to all unqualified tables in the query tree.

```rust
use rust_kysely::plugin::WithSchemaPlugin;

let db = db.with_plugin(WithSchemaPlugin::new("tenant_42"));
// SELECT * FROM users -> SELECT * FROM tenant_42.users
```

---

## Authoring Custom Plugins

You can author custom security plugins, multitenancy filters, or audit loggers by implementing `KyselyPlugin`:

```rust
use async_trait::async_trait;
use rust_kysely::plugin::{KyselyPlugin, PluginTransformArgs, PluginTransformResultArgs};
use rust_kysely::operation_node::OperationNode;
use rust_kysely::query_executor::row::QueryResult;
use rust_kysely::driver::DriverError;

#[derive(Debug, Default)]
pub struct AuditLogPlugin;

#[async_trait]
impl KyselyPlugin for AuditLogPlugin {
    fn transform_query(&self, args: PluginTransformArgs) -> OperationNode {
        // Inspect or transform AST
        args.node
    }

    async fn transform_result(
        &self,
        args: PluginTransformResultArgs,
    ) -> Result<QueryResult, DriverError> {
        println!("Executed query {:?} returned {} rows", args.query_id, args.result.rows.len());
        Ok(args.result)
    }
}
```
