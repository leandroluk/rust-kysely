# Design: 02 — Expressions, ExpressionBuilder (`eb`) & Macro-Free Raw SQL (`sql`)

## Architecture Overview

The expression subsystem bridges raw SQL literals, functional expressions, and boolean logic into typed `OperationNode` representations without procedural macros.

```
┌────────────────────────────────────────────────────────┐
│               Expression<T> Trait                      │
│     - to_operation_node(&self) -> Arc<OperationNode>   │
│     - PhantomData<T> for typed query inference         │
└───────────────────────────┬────────────────────────────┘
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
┌──────────────┐   ┌─────────────────┐   ┌────────────────┐
│  RawBuilder  │   │ExpressionBuilder│   │ FunctionModule │
│  (sql(...))  │   │     (`eb`)      │   │  (eb.fn / db)  │
└──────┬───────┘   └────────┬────────┘   └───────┬────────┘
       │                    │                    │
       │ .bind(val)         │ eb.or / eb.and     │ count / sum
       │ sql::id / lit      │ eb.cmp / between   │ coalesce / avg
       │ sql::join          │ eb.exists          │ .as_("alias")
       ▼                    ▼                    ▼
┌────────────────────────────────────────────────────────┐
│                   OperationNode AST                    │
│      (RawNode, BinaryOperationNode, FunctionNode)      │
└────────────────────────────────────────────────────────┘
```

## Dependency Paths (from nexspec)
- REQ-EXPR-001 → `src/expression/mod.rs` (`Expression` trait, `ExpressionWrapper<T>`, `IntoValue` trait)
- REQ-EXPR-002 → `src/raw_builder/mod.rs` (`RawBuilder`, `sql(...)`, `.bind(v)`)
- REQ-EXPR-003, REQ-EXPR-004, REQ-EXPR-005 → `src/raw_builder/helpers.rs` (`sql::raw`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`)
- REQ-EXPR-006, REQ-EXPR-007 → `src/expression/expression_builder.rs` (`ExpressionBuilder`, boolean logic, `cmp`)
- REQ-EXPR-008, REQ-EXPR-009 → `src/expression/expression_builder.rs` (`between`, `exists`)
- REQ-EXPR-010 → `src/expression/function_module.rs` (`FunctionModule`, `AggregateFunctionNode`)

## New Components

| Component | Responsibility | Location |
|---|---|---|
| `Expression<T>` | Trait returning an `Arc<OperationNode>` with phantom result type `T` | `src/expression/mod.rs` |
| `ExpressionWrapper<T>` | Generic concrete wrapper around any `Arc<OperationNode>` | `src/expression/wrapper.rs` |
| `IntoValue` | Trait for frictionless conversion of scalar Rust values into `Value` without `.into()` | `src/operation_node/value.rs` |
| `RawBuilder` | Fluent builder returned by `sql("...")` with `.bind()` and `.append()` | `src/raw_builder/mod.rs` |
| `sql::*` helpers | Helper functions for identifiers, tables, literals, and joins inside raw SQL | `src/raw_builder/helpers.rs` |
| `ExpressionBuilder` | Expression context passed to closures (`|eb| ...`) for boolean logic and comparisons | `src/expression/expression_builder.rs` |
| `FunctionModule` | Factory for SQL aggregation and scalar functions (`count`, `max`, `coalesce`, etc.) | `src/expression/function_module.rs` |
| `FunctionNode` | AST node representing SQL function calls `name(args...)` | `src/operation_node/expression_nodes.rs` |

## Modified Components

| Component | Change | Risk |
|---|---|---|
| `src/operation_node/value.rs` | Add `IntoValue` trait blanket implementation | Low |
| `src/operation_node/expression_nodes.rs` | Add `FunctionNode` AST variant | Low |
| `src/operation_node/mod.rs` | Expose `FunctionNode` in `OperationNode` | Low |
| `src/operation_node/visitor.rs` | Add `visit_function` | Low |
| `src/operation_node/transformer.rs` | Add `transform_function` | Low |
| `src/query_compiler/default_compiler.rs` | Add SQL formatting for `FunctionNode` (`func(arg1, arg2)`) | Low |
| `src/lib.rs` | Expose `expression` and `raw_builder` modules and `sql` function | Low |

## Risks & Mitigation
- **SQL Injection in Raw SQL**: Raw strings could introduce injection vulnerabilities if not parameterized.
  *Mitigation*: `sql("...")` uses `.bind(v)` producing `ValueNode` parameters. `sql::raw(...)` is an explicit opt-in only.
- **Ergonomics with Type Inference**: Expressions return phantom type `T` which could require excessive annotations.
  *Mitigation*: Provide default phantom types (`ExpressionWrapper<()>` or `Value`) and helper methods `.as_("alias")` preserving ergonomics.

## Decision Log
- **No Procedural Macros**: Rejected `sql!("...")` procedural macro in favor of fluent `sql("...").bind(...)` to guarantee sub-millisecond compilation, transparent compilation errors, and complete IDE inlay hints.
- **Function Calls in AST**: Model function invocations as `FunctionNode { name: CompactString, arguments: SmallVec<[Arc<OperationNode>; 2]> }` rather than raw strings, enabling multi-dialect compiler transformations.
