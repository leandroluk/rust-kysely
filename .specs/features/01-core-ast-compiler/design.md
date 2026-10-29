# Design: 01 — Core OperationNode AST, Visitors & PostgreSQL Compiler

## Architecture Overview

The core layer of `rust-kysely` models database operations as an immutable Abstract Syntax Tree (AST), centered around the `OperationNode` enum. All builder interfaces produce and manipulate this tree. Query execution and SQL generation compile this tree into dialect-specific `CompiledQuery` representations.

```
┌────────────────────────────────────────────────────────┐
│                   Query Builder API                    │
│      (SelectQueryBuilder, InsertQueryBuilder, etc.)    │
└───────────────────────────┬────────────────────────────┘
                            │ Arc::make_mut (CoW)
                            ▼
┌────────────────────────────────────────────────────────┐
│                   OperationNode AST                    │
│  - Queries: SelectQueryNode, InsertQueryNode, ...      │
│  - Clauses: FromNode, JoinNode, WhereNode, ...         │
│  - Expressions: BinaryOperationNode, CaseNode, ...     │
│  - Terminals: TableNode, ColumnNode, ValueNode, ...    │
│  (Zero-copy Ident via CompactStr, SmallVec for lists)  │
└─────────────┬────────────────────────────┬─────────────┘
              │                            │
              ▼                            ▼
┌───────────────────────────┐┌───────────────────────────┐
│   OperationNodeVisitor    ││ OperationNodeTransformer  │
│  (Inspection / Analysis)  ││ (Query Plugins / Rewrites)│
└─────────────┬─────────────┘└───────────────────────────┘
              │
              ▼
┌────────────────────────────────────────────────────────┐
│                  QueryCompiler Trait                   │
│   ├── DefaultQueryCompiler (ANSI Standard, `?`)        │
│   └── PostgresQueryCompiler (PostgreSQL, `$N`, `""`)   │
└───────────────────────────┬────────────────────────────┘
                            │ compile(&node)
                            ▼
┌────────────────────────────────────────────────────────┐
│                     CompiledQuery                      │
│   - sql: String ("SELECT \"name\" FROM \"users\" ...")     │
│   - parameters: Vec<Value>                             │
│   - query_id: QueryId (64-bit token)                   │
└────────────────────────────────────────────────────────┘
```

## Dependency Paths (from nexspec)
- REQ-AST-001 → `src/operation_node/mod.rs` & specialized submodules (New AST hierarchy)
- REQ-AST-002 → `Arc<OperationNode>` with Copy-on-Write (`Arc::make_mut`)
- REQ-AST-003 → `compact_str::CompactStr` for identifiers
- REQ-AST-004 → `smallvec::SmallVec` for selections, joins, and parameters
- REQ-AST-005 → `src/util/query_id.rs` (`QueryId` token generation)
- REQ-AST-006 → `src/operation_node/visitor.rs` (`OperationNodeVisitor` trait)
- REQ-AST-007 → `src/operation_node/transformer.rs` (`OperationNodeTransformer` trait)
- REQ-AST-008 → `src/query_compiler/mod.rs` & `DefaultQueryCompiler`
- REQ-AST-009 → `src/dialect/postgres/query_compiler.rs` (`PostgresQueryCompiler`)
- REQ-AST-010 → Footprint check & zero proc-macros validation

## New Components

| Component | Responsibility | Location |
|---|---|---|
| `QueryId` | 64-bit identifier generator for tracking queries across lifecycle and plugins | `src/util/query_id.rs` |
| `Value` | Strongly-typed SQL scalar literal representation (Null, Bool, Int, Float, Text, Bytes) | `src/operation_node/value.rs` |
| `OperationNode` | Top-level enum representing any node in the AST | `src/operation_node/mod.rs` |
| Query Nodes | `SelectQueryNode`, `InsertQueryNode`, `UpdateQueryNode`, `DeleteQueryNode` | `src/operation_node/query_nodes.rs` |
| Clause Nodes | `FromNode`, `JoinNode`, `WhereNode`, `HavingNode`, `GroupByNode`, `OrderByNode`, `LimitNode`, `OffsetNode`, `WithNode` | `src/operation_node/clause_nodes.rs` |
| Expression Nodes | `BinaryOperationNode`, `UnaryOperationNode`, `CaseNode`, `WhenNode`, `CastNode`, `AliasNode`, `ParensNode` | `src/operation_node/expression_nodes.rs` |
| Terminal Nodes | `TableNode`, `ColumnNode`, `ReferenceNode`, `IdentifierNode`, `RawNode` | `src/operation_node/terminal_nodes.rs` |
| `OperationNodeVisitor` | Trait for walking the AST with parent node tracking | `src/operation_node/visitor.rs` |
| `OperationNodeTransformer` | Trait for transforming and rewriting AST nodes | `src/operation_node/transformer.rs` |
| `CompiledQuery` | Encapsulates final SQL string, parameter values, and QueryId | `src/query_compiler/compiled_query.rs` |
| `DefaultQueryCompiler` | ANSI baseline SQL generator with `?` placeholders | `src/query_compiler/default_compiler.rs` |
| `PostgresQueryCompiler` | PostgreSQL dialect generator with `$1, $2, ...` and `"identifier"` escaping | `src/dialect/postgres/query_compiler.rs` |

## Modified Components

| Component | Change | Risk |
|---|---|---|
| `src/lib.rs` | Export modules `operation_node`, `query_compiler`, `dialect`, `util`, and public types | Low — initial skeleton |
| `tests/integration.rs` | Add comprehensive unit and integration tests for AST, Visitors, and Compilers | Low — dev test suite |

## Risks & Mitigation
- **Deep Recursion in AST Traversal**: SQL queries can contain deeply nested subqueries or expression trees.
  *Mitigation*: Implement standard iterative loops over collections where possible in `OperationNodeVisitor`, and keep stack frames lean.
- **Allocation Overhead in Query Compilation**: String concatenation during SQL generation can trigger frequent reallocations.
  *Mitigation*: Pre-allocate capacity in `String` buffer (e.g. 128 bytes baseline) and use `CompactStr` for inline identifier serialization without heap touches.

## Decision Log
- **Single Enum vs Trait Objects for AST**: Used `enum OperationNode` instead of `Box<dyn OperationNode>` because enums allow fast pattern matching, exhaustive `match` checking in `Visitor` and `Compiler`, and zero virtual dispatch overhead.
- **Copy-on-Write Strategy**: Use `Arc<T>` wrappers on node pointers so builders can share existing subtrees until a mutation occurs (`Arc::make_mut`), providing thread-safe, zero-copy query construction.
- **Identifier Stack Storage**: Used `compact_str::CompactStr` allowing column and table names up to 24 bytes to stay entirely on the stack.
