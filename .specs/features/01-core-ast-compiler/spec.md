# Spec: 01 — Core OperationNode AST, Visitors & PostgreSQL Compiler

## Summary
Establishes the fundamental SQL Abstract Syntax Tree (`OperationNode`) and query compilation infrastructure for `rust-kysely`, ported faithfully from upstream Kysely (`src/operation-node/` and `src/query-compiler/`). Implements immutable tree representation wrapped in `Arc` with Copy-on-Write (`Arc::make_mut`), zero-copy identifiers with `compact_str::CompactStr`, optimized argument collections with `smallvec::SmallVec`, query execution IDs (`QueryId`), and the `OperationNodeVisitor` trait. Provides both `DefaultQueryCompiler` (ANSI baseline) and `PostgresQueryCompiler` producing `CompiledQuery` with parameterized placeholders (`$1, $2, ...`) and sanitized identifier escaping (`""`), completely free of procedural macros and heavy build footprints.

## Requirements
- REQ-AST-001: Implement the core `OperationNode` enum and structural nodes covering queries (`SelectQueryNode`, `InsertQueryNode`, `UpdateQueryNode`, `DeleteQueryNode`), clauses (`FromNode`, `JoinNode`, `WhereNode`, `HavingNode`, `GroupByNode`, `OrderByNode`, `LimitNode`, `OffsetNode`, `WithNode`), expressions (`BinaryOperationNode`, `UnaryOperationNode`, `CaseNode`, `WhenNode`, `CastNode`), and terminals (`TableNode`, `ColumnNode`, `ReferenceNode`, `ValueNode`, `RawNode`, `IdentifierNode`).
- REQ-AST-002: Implement `Arc<OperationNode>` with Copy-on-Write (`Arc::make_mut`) across all builder transformations to guarantee immutable query reuse with zero redundant reallocations.
- REQ-AST-003: Use `compact_str::CompactStr` for identifier names (tables, columns, schemas, aliases) so identifiers under 24 bytes live inline on the stack without heap allocation.
- REQ-AST-004: Use `smallvec::SmallVec<[T; 4]>` for selection lists, joins, and parameters to avoid heap allocations for common small-sized queries.
- REQ-AST-005: Implement `QueryId` generator producing sequential or cryptographically random 64-bit query tracking tokens.
- REQ-AST-006: Implement the `OperationNodeVisitor` trait providing default recursive traversal across all node variants with node stack tracking (`parentNode`).
- REQ-AST-007: Implement the `OperationNodeTransformer` trait enabling immutable AST cloning and mutation for query rewriting plugins.
- REQ-AST-008: Implement `DefaultQueryCompiler` translating the AST into parameterized SQL with `add_parameter` accumulation and `CompiledQuery { sql, parameters, query_id }`.
- REQ-AST-009: Implement `PostgresQueryCompiler` overriding identifier wrapping to double quotes (`"identifier"` with `""` escaping) and parameter placeholders to sequential dollar tokens (`$1`, `$2`, `$3`).
- REQ-AST-010: Ensure zero procedural macros (`no syn`, `no quote`, `no proc-macro`) and adhere to the strict `target/` footprint policy (`debug = 0`, `strip = "debuginfo"`, `incremental = false`).

## Affected Components
- `src/operation_node/` — Unified AST enum and specialized node structs
- `src/query_compiler/` — `QueryCompiler`, `CompiledQuery`, `DefaultQueryCompiler`, `OperationNodeVisitor`
- `src/dialect/postgres/` — `PostgresQueryCompiler`
- `src/util/query_id.rs` — `QueryId` generation

## Out of Scope
- Procedural derive macros or compiler-time schema validation against a live DB.
- MySQL / SQLite / MSSQL specific dialect compilers (reserved for future milestones).
- Non-parameterized raw string concatenation (injection risk).

## Open Questions
- None.
