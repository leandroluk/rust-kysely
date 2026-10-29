# Project State

## Status
- **Current Milestone**: Milestone 5 (PostgreSQL Driver & Query Execution) — Complete!
- **Active Feature**: [05-postgres-driver-execution](../features/05-postgres-driver-execution/spec.md)
- **Current Phase**: Verification & Next Milestone Planning
- **Active Task**: Milestone 5 Complete (Next: Milestone 6 DDL & Schema)
- **Blockers**: None

## Cost
- Milestones 1, 2, 3, 4 e 5 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010, REQ-MUT-001..010 e REQ-EXEC-001..010 satisfeitos.
- 50 testes de integração e unitários passando em 0.04s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~362MB, zero `.pdb`s descontrolados, 100% livre de macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-DRV-007 complete. Gate: 50/50 pass. Milestone 5 100% Complete! [REQ-EXEC-001..010]
- 2026-10-29 TASK-DRV-006 complete. Gate: 49/49 pass. [REQ-EXEC-002] [REQ-EXEC-003]
- 2026-10-29 TASK-DRV-005 complete. Gate: 48/48 pass. [REQ-EXEC-009] [REQ-EXEC-010]
- 2026-10-29 TASK-DRV-004 complete. Gate: 47/47 pass. [REQ-EXEC-003] [REQ-EXEC-004] [REQ-EXEC-006]
- 2026-10-29 TASK-DRV-003 complete. Gate: 46/46 pass. [REQ-EXEC-004] [REQ-EXEC-007] [REQ-EXEC-008]
- 2026-10-29 TASK-DRV-002 complete. Gate: 45/45 pass. [REQ-EXEC-005] [REQ-EXEC-006]
- 2026-10-29 TASK-DRV-001 complete. Gate: 44/44 pass. [REQ-EXEC-001]
- 2026-10-29 Milestone 5 designed: design.md and tasks.md created. [REQ-EXEC-001..010]
- 2026-10-29 TASK-MUT-007 complete. Gate: 43/43 pass. Milestone 4 100% Complete! [REQ-MUT-001..010]
- 2026-10-29 TASK-MUT-006 complete. Gate: 40/40 pass. [REQ-MUT-010]

## Decisions (Recent 5)
1. **Conclusão da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Implementadas traits `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com getters tipados (`get::<T>`, `get_opt::<T>`, case-insensitive) e deserialização serde (`into_record`), `QueryExecutor` e `SingleConnectionExecutor`, facade `Kysely`, transações atômicas com `execute(move |trx| ...)` e savepoints aninhados (`savepoint_block`), e `PostgresDriver` com `tokio-postgres` e conversão dinâmica de parâmetros.
2. **Design da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Estruturada a arquitetura com traits assíncronas `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com extração tipada e serde, `QueryExecutor`, facade `Kysely`, transações automáticas e savepoints.
3. **Design e Conclusão da Milestone 4 (Mutation Query Builders)**: Implementados `InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`, `OnConflictBuilder`, e `RowValues` com suporte a `set`, `values_batch`, serde structs (`values_record`), `excluded` references, PostgreSQL `ON CONFLICT` e `RETURNING` clauses.
4. **Design e Conclusão da Milestone 3 (SelectQueryBuilder)**: Implementado `SelectQueryBuilder` com projections heterogêneas via `IntoSelections`, joins com closure `JoinBuilder`, filtros encadeáveis `AND`/`OR`, agrupamento, ordenação com nulls, paginação com limit/offset, CTEs (`with`/`with_recursive`) e operações de conjunto (`union`, `union_all`, `intersect`, `except`).
5. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).

## Todos
- [x] TASK-DRV-001: Implement Driver and DatabaseConnection async traits — Execute phase
- [x] TASK-DRV-002: Implement QueryResult, Row, and type-safe value extraction — Execute phase
- [x] TASK-DRV-003: Implement QueryExecutor and execution coordinating pipeline — Execute phase
- [x] TASK-DRV-004: Implement Kysely client struct and builder execution chaining — Execute phase
- [x] TASK-DRV-005: Implement Transaction and Savepoint management — Execute phase
- [x] TASK-DRV-006: Implement PostgresDriver with tokio-postgres — Execute phase
- [x] TASK-DRV-007: Comprehensive integration tests for query execution engine — Execute phase
