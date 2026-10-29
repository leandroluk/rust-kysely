# Spec: 02 — Expressions, ExpressionBuilder (`eb`) & Macro-Free Raw SQL (`sql`)

## Summary
Implements the expression subsystem and raw SQL construction for `rust-kysely`, ported from upstream `src/expression/` and `src/raw-builder/`. Replaces TypeScript's tagged template literal with a fluent, injection-safe `sql("...")` builder with parameter binding (`.bind(v)`), and helper utilities (`sql::id`, `sql::table`, `sql::lit`, `sql::raw`, `sql::join`). Delivers the `ExpressionBuilder` (`eb`) struct passed to closures (`|eb| ...`), enabling boolean logic (`eb.or`, `eb.and`, `eb.not`), comparisons (`eb.cmp`), subqueries (`eb.select_from`), and standard SQL aggregation functions (`eb.fn.count`, `eb.fn.max`, `eb.fn.coalesce`).

## Requirements
- REQ-EXPR-001: Implement the `Expression<T>` trait and `ExpressionWrapper<T>` wrapping an `OperationNode` with phantom output type `T`.
- REQ-EXPR-002: Implement `sql("...")` builder with method `.bind(value)` supporting any type implementing `IntoValue`, automatically appending `ValueNode` to the AST and escaping parameters.
- REQ-EXPR-003: Provide `sql::raw(str)` for verbatim SQL injection without parameterization when explicitly intended.
- REQ-EXPR-004: Provide `sql::id(ident)` and `sql::table(name)` constructing sanitized `IdentifierNode` and `TableNode` inside raw SQL fragments.
- REQ-EXPR-005: Provide `sql::lit(value)` for escaped SQL literal representation and `sql::join(items, separator)` for joining expressions with delimiters.
- REQ-EXPR-006: Implement `ExpressionBuilder` providing boolean combinators `eb.or(iter)`, `eb.and(iter)`, and `eb.not(expr)` compiling to `ParensNode`, `OrNode`, and `AndNode`.
- REQ-EXPR-007: Implement `eb.cmp(left, op, right)` supporting column-to-value (`BinaryOperationNode`), column-to-column (`eb.cmp_ref`), and column-to-expression comparisons.
- REQ-EXPR-008: Implement `eb.between(expr, min, max)` and `eb.between_symmetric(expr, min, max)`.
- REQ-EXPR-009: Implement `eb.exists(subquery)` and `eb.not_exists(subquery)` supporting closure-based subqueries (`|eb| eb.select_from(...)`).
- REQ-EXPR-010: Implement `FunctionModule` (`db.fn()` and `eb.fn()`) supporting `count`, `count_all`, `max`, `min`, `avg`, `sum`, and `coalesce` with aliasing (`.as_("alias")`).

## Affected Components
- `src/raw_builder/` — `Sql`, `RawBuilder`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`
- `src/expression/` — `Expression`, `ExpressionWrapper`, `ExpressionBuilder`
- `src/query_builder/function_module.rs` — `FunctionModule` (`count`, `sum`, `avg`, etc.)

## Out of Scope
- Procedural `sql!("...")` template macro (replaced by pure fluent `sql(...)`).
- Dialect-specific custom functions outside standard ANSI/Postgres.

## Open Questions
- None.
