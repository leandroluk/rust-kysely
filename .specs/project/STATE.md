# Project State

## Status
- **Current Milestone**: Milestone 5 (PostgreSQL Driver & Query Execution) — Design Phase
- **Active Feature**: [05-postgres-driver-execution](../features/05-postgres-driver-execution/spec.md)
- **Current Phase**: Design
- **Active Task**: None (Milestone 4 complete, proceeding to Milestone 5 design)
- **Blockers**: None

## Cost
- Milestones 1, 2, 3 e 4 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010 e REQ-MUT-001..010 satisfeitos.
- 43 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~352MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-MUT-007 complete. Gate: 43/43 pass. Milestone 4 100% Complete! [REQ-MUT-001..010]
- 2026-10-29 TASK-MUT-006 complete. Gate: 40/40 pass. [REQ-MUT-010]
- 2026-10-29 TASK-MUT-005 complete. Gate: 39/39 pass. [REQ-MUT-009]
- 2026-10-29 TASK-MUT-004 complete. Gate: 38/38 pass. [REQ-MUT-007] [REQ-MUT-008]
- 2026-10-29 TASK-MUT-003 complete. Gate: 37/37 pass. [REQ-MUT-005] [REQ-MUT-006]
- 2026-10-29 TASK-MUT-002 complete. Gate: 36/36 pass. [REQ-MUT-003] [REQ-MUT-004]
- 2026-10-29 TASK-MUT-001 complete. Gate: 35/35 pass. [REQ-MUT-001] [REQ-MUT-002]
- 2026-10-29 Milestone 4 designed: design.md and tasks.md created. [REQ-MUT-001..010]
- 2026-10-29 TASK-SEL-007 complete. Gate: 34/34 pass. Milestone 3 100% Complete! [REQ-SEL-001..010]
- 2026-10-29 TASK-SEL-006 complete. Gate: 32/32 pass. [REQ-SEL-010]

## Decisions (Recent 5)
1. **Design e Conclusão da Milestone 4 (Mutation Query Builders)**: Implementados `InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`, `OnConflictBuilder`, e `RowValues` com suporte a `set`, `values_batch`, serde structs (`values_record`), `excluded` references, PostgreSQL `ON CONFLICT` e `RETURNING` clauses.
2. **Design e Conclusão da Milestone 3 (SelectQueryBuilder)**: Implementado `SelectQueryBuilder` com projections heterogêneas via `IntoSelections`, joins com closure `JoinBuilder`, filtros encadeáveis `AND`/`OR`, agrupamento, ordenação com nulls, paginação com limit/offset, CTEs (`with`/`with_recursive`) e operações de conjunto (`union`, `union_all`, `intersect`, `except`).
3. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).
4. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).
5. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).

## Todos
- [x] TASK-MUT-001: Implement InsertQueryBuilder and chained set method — Execute phase
- [x] TASK-MUT-002: Implement values, values_batch, and values_record — Execute phase
- [x] TASK-MUT-003: Implement OnConflictBuilder and PostgreSQL upsert — Execute phase
- [x] TASK-MUT-004: Implement UpdateQueryBuilder with joins and filters — Execute phase
- [x] TASK-MUT-005: Implement DeleteQueryBuilder with using joins and filters — Execute phase
- [x] TASK-MUT-006: Implement returning and returning_all on all mutation builders — Execute phase
- [x] TASK-MUT-007: Comprehensive integration tests for DML builders — Execute phase
