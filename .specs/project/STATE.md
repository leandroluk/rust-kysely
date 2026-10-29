# Project State

## Status
- **Current Milestone**: Milestone 4 (Mutation Query Builders: Insert, Update, Delete) — Design Phase
- **Active Feature**: [04-mutation-query-builders](../features/04-mutation-query-builders/spec.md)
- **Current Phase**: Design
- **Active Task**: None (Milestone 3 complete, proceeding to Milestone 4 design)
- **Blockers**: None

## Cost
- Milestone 1 (`01-core-ast-compiler`), Milestone 2 (`02-expressions-raw-sql`), e Milestone 3 (`03-select-query-builder`) implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010 e REQ-SEL-001..010 satisfeitos.
- 34 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~274MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-SEL-007 complete. Gate: 34/34 pass. Milestone 3 100% Complete! [REQ-SEL-001..010]
- 2026-10-29 TASK-SEL-006 complete. Gate: 32/32 pass. [REQ-SEL-010]
- 2026-10-29 TASK-SEL-005 complete. Gate: 31/31 pass. [REQ-SEL-007] [REQ-SEL-008] [REQ-SEL-009]
- 2026-10-29 TASK-SEL-004 complete. Gate: 30/30 pass. [REQ-SEL-004] [REQ-SEL-006]
- 2026-10-29 TASK-SEL-003 complete. Gate: 28/28 pass. [REQ-SEL-003]
- 2026-10-29 TASK-SEL-002 complete. Gate: 27/27 pass. [REQ-SEL-001] [REQ-SEL-002]
- 2026-10-29 TASK-SEL-001 complete. Gate: 25/25 pass. [REQ-SEL-005] [REQ-SEL-006]
- 2026-10-29 TASK-EXPR-007 complete. Gate: 24/24 pass. Footprint: 272 MB. [REQ-EXPR-001..010]
- 2026-10-29 TASK-EXPR-006 complete. Gate: 23/23 pass. [REQ-EXPR-010]
- 2026-10-29 TASK-EXPR-005 complete. Gate: 22/22 pass. [REQ-EXPR-008] [REQ-EXPR-009]

## Decisions (Recent 5)
1. **Design e Conclusão da Milestone 3 (SelectQueryBuilder)**: Implementado `SelectQueryBuilder` com projections heterogêneas via `IntoSelections`, joins com closure `JoinBuilder`, filtros encadeáveis `AND`/`OR`, agrupamento, ordenação com nulls, paginação com limit/offset, CTEs (`with`/`with_recursive`) e operações de conjunto (`union`, `union_all`, `intersect`, `except`).
2. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).
3. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).
4. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).
5. **Documentação Integral do `.specs/`**: Especificadas todas as 9 fases do projeto cobrindo desde a AST até DDL, Migrations, Plugins e Multi-Dialetos (`REQ-AST-*` até `REQ-DIA-*`).

## Todos
- [x] TASK-SEL-001: Implement IntoSelections trait and tuple projections — Execute phase
- [x] TASK-SEL-002: Implement IntoTableSource and select_from constructor — Execute phase
- [x] TASK-SEL-003: Implement join methods and JoinBuilder — Execute phase
- [x] TASK-SEL-004: Implement filtering methods (where_, where_ref, where_expr, or_where_*) — Execute phase
- [x] TASK-SEL-005: Implement grouping, having, sorting, and pagination — Execute phase
- [x] TASK-SEL-006: Implement CTEs (with) and set operations (union, intersect, except) — Execute phase
- [x] TASK-SEL-007: Comprehensive integration tests for SelectQueryBuilder — Execute phase
