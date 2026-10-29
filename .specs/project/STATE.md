# Project State

## Status
- **Current Milestone**: Milestone 4 (Mutation Query Builders: Insert, Update, Delete) — Execute Phase
- **Active Feature**: [04-mutation-query-builders](../features/04-mutation-query-builders/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-MUT-007
- **Blockers**: None

## Cost
- Milestone 1 (`01-core-ast-compiler`), Milestone 2 (`02-expressions-raw-sql`), e Milestone 3 (`03-select-query-builder`) implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010 e REQ-SEL-001..010 satisfeitos.
- Design e decomposição em 7 tarefas atômicas concluídos para a Milestone 4 (`TASK-MUT-001` a `TASK-MUT-007`).
- 40 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~274MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-MUT-006 complete. Gate: 40/40 pass. [REQ-MUT-010]
- 2026-10-29 TASK-MUT-005 complete. Gate: 39/39 pass. [REQ-MUT-009]
- 2026-10-29 TASK-MUT-004 complete. Gate: 38/38 pass. [REQ-MUT-007] [REQ-MUT-008]
- 2026-10-29 TASK-MUT-003 complete. Gate: 37/37 pass. [REQ-MUT-005] [REQ-MUT-006]
- 2026-10-29 TASK-MUT-002 complete. Gate: 36/36 pass. [REQ-MUT-003] [REQ-MUT-004]
- 2026-10-29 TASK-MUT-001 complete. Gate: 35/35 pass. [REQ-MUT-001] [REQ-MUT-002]
- 2026-10-29 Milestone 4 designed: design.md and tasks.md created. [REQ-MUT-001..010]
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
- [x] TASK-MUT-001: Implement InsertQueryBuilder and chained set method — Execute phase
- [x] TASK-MUT-002: Implement values, values_batch, and values_record — Execute phase
- [x] TASK-MUT-003: Implement OnConflictBuilder and PostgreSQL upsert — Execute phase
- [x] TASK-MUT-004: Implement UpdateQueryBuilder with joins and filters — Execute phase
- [x] TASK-MUT-005: Implement DeleteQueryBuilder with using joins and filters — Execute phase
- [x] TASK-MUT-006: Implement returning and returning_all on all mutation builders — Execute phase
- [ ] TASK-MUT-007: Comprehensive integration tests for DML builders — Execute phase
