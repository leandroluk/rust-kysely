# Project State

## Status
- **Current Milestone**: Initialization & Architectural Design (Milestone 1 Prep)
- **Active Feature**: Specifying and Mapping Kysely Architecture to Pure Rust (`INSIGHT.md`)
- **Blockers**: None

## Cost
- Initialized empty repository with Graph-Spec-Design pipeline.
- Upstream Kysely cloned for direct AST, compiler, and API parity analysis.

## Decisions (Recent 5)
1. **Zero Macros Mandate**: Absolute elimination of proc-macros (`derive`, `macro_rules! DSLs`, or compile-time DB introspection) to keep compile times fast, error messages crystal clear, and IDE autocompletion fluid.
2. **Postgres First**: Direct focus on PostgreSQL dialect, AST compiler (`PostgresQueryCompiler`), and `tokio-postgres` driver connection integration.
3. **Closure Builders (`|eb| ...`)**: Adopting Rust closure builders to represent Kysely's fluent arrow functions for nested expressions, subqueries, and transactions.
4. **Inspectable AST**: Porting Kysely's `OperationNode` hierarchy as clean Rust structs/enums wrapped in `Arc` with Copy-on-Write for fluent immutability.
5. **Ergonomic Rows**: Supporting both dynamic type-safe `Row` (`row.get::<T>("col")?`) and typed deserialization (`.execute_into::<T>()`) without custom derives.
