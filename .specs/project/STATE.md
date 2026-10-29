# Project State

## Status
- **Current Milestone**: Milestone 5 (PostgreSQL Driver & Query Execution) — Execute Phase
- **Active Feature**: [05-postgres-driver-execution](../features/05-postgres-driver-execution/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-DRV-001
- **Blockers**: None

## Cost
- Milestones 1, 2, 3 e 4 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010 e REQ-MUT-001..010 satisfeitos.
- Design e decomposição em 7 tarefas atômicas concluídos para a Milestone 5 (`TASK-DRV-001` a `TASK-DRV-007`).
- 43 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~352MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 Milestone 5 designed: design.md and tasks.md created. [REQ-EXEC-001..010]
- 2026-10-29 TASK-MUT-007 complete. Gate: 43/43 pass. Milestone 4 100% Complete! [REQ-MUT-001..010]
- 2026-10-29 TASK-MUT-006 complete. Gate: 40/40 pass. [REQ-MUT-010]
- 2026-10-29 TASK-MUT-005 complete. Gate: 39/39 pass. [REQ-MUT-009]
- 2026-10-29 TASK-MUT-004 complete. Gate: 38/38 pass. [REQ-MUT-007] [REQ-MUT-008]
- 2026-10-29 TASK-MUT-003 complete. Gate: 37/37 pass. [REQ-MUT-005] [REQ-MUT-006]
- 2026-10-29 TASK-MUT-002 complete. Gate: 36/36 pass. [REQ-MUT-003] [REQ-MUT-004]
- 2026-10-29 TASK-MUT-001 complete. Gate: 35/35 pass. [REQ-MUT-001] [REQ-MUT-002]
- 2026-10-29 Milestone 4 designed: design.md and tasks.md created. [REQ-MUT-001..010]
- 2026-10-29 TASK-SEL-007 complete. Gate: 34/34 pass. Milestone 3 100% Complete! [REQ-SEL-001..010]

## Decisions (Recent 5)
1. **Design da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Estruturada a arquitetura com traits assíncronas `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com extração tipada e serde, `QueryExecutor`, facade `Kysely`, transações automáticas e savepoints.
2. **Design e Conclusão da Milestone 4 (Mutation Query Builders)**: Implementados `InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`, `OnConflictBuilder`, e `RowValues` com suporte a `set`, `values_batch`, serde structs (`values_record`), `excluded` references, PostgreSQL `ON CONFLICT` e `RETURNING` clauses.
3. **Design e Conclusão da Milestone 3 (SelectQueryBuilder)**: Implementado `SelectQueryBuilder` com projections heterogêneas via `IntoSelections`, joins com closure `JoinBuilder`, filtros encadeáveis `AND`/`OR`, agrupamento, ordenação com nulls, paginação com limit/offset, CTEs (`with`/`with_recursive`) e operações de conjunto (`union`, `union_all`, `intersect`, `except`).
4. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).
5. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).

## Todos
- [ ] TASK-DRV-001: Implement Driver and DatabaseConnection async traits — Execute phase
- [ ] TASK-DRV-002: Implement QueryResult, Row, and type-safe value extraction — Execute phase
- [ ] TASK-DRV-003: Implement QueryExecutor and execution coordinating pipeline — Execute phase
- [ ] TASK-DRV-004: Implement Kysely client struct and builder execution chaining — Execute phase
- [ ] TASK-DRV-005: Implement Transaction and Savepoint management — Execute phase
- [ ] TASK-DRV-006: Implement PostgresDriver with tokio-postgres — Execute phase
- [ ] TASK-DRV-007: Comprehensive integration tests for query execution engine — Execute phase
