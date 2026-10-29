# Project State

## Status
- **Current Milestone**: Milestone 3 (Select Query Builder, Joins & Projections) — Em Execução
- **Active Feature**: [03-select-query-builder](../features/03-select-query-builder/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-SEL-004
- **Blockers**: None

## Cost
- Milestone 1 e Milestone 2 implementadas integralmente com 100% dos requisitos REQ-AST-001 a REQ-AST-010 e REQ-EXPR-001 a REQ-EXPR-010 satisfeitos.
- Design e decomposição em 7 tarefas atômicas concluídos para a Milestone 3 (`TASK-SEL-001` a `TASK-SEL-007`).
- 28 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~272MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-SEL-003 complete. Gate: 28/28 pass. [REQ-SEL-003]
- 2026-10-29 TASK-SEL-002 complete. Gate: 27/27 pass. [REQ-SEL-001] [REQ-SEL-002]
- 2026-10-29 TASK-SEL-001 complete. Gate: 25/25 pass. [REQ-SEL-005] [REQ-SEL-006]
- 2026-10-29 TASK-EXPR-007 complete. Gate: 24/24 pass. Footprint: 272 MB. [REQ-EXPR-001..010]
- 2026-10-29 TASK-EXPR-006 complete. Gate: 23/23 pass. [REQ-EXPR-010]
- 2026-10-29 TASK-EXPR-005 complete. Gate: 22/22 pass. [REQ-EXPR-008] [REQ-EXPR-009]
- 2026-10-29 TASK-EXPR-004 complete. Gate: 20/20 pass. [REQ-EXPR-006] [REQ-EXPR-007]
- 2026-10-29 TASK-EXPR-003 complete. Gate: 18/18 pass. [REQ-EXPR-003] [REQ-EXPR-004] [REQ-EXPR-005]
- 2026-10-29 TASK-EXPR-002 complete. Gate: 17/17 pass. [REQ-EXPR-002]
- 2026-10-29 TASK-EXPR-001 complete. Gate: 16/16 pass. [REQ-EXPR-001] [REQ-EXPR-002]
- 2026-10-29 TASK-AST-008 complete. Gate: 15/15 pass. Footprint: 253 MB. [REQ-AST-010]
- 2026-10-29 TASK-AST-007 complete. Gate: 11/11 pass. [REQ-AST-009]
- 2026-10-29 TASK-AST-006 complete. Gate: 10/10 pass. [REQ-AST-008]

## Decisions (Recent 5)
1. **Design da Milestone 3 (SelectQueryBuilder)**: Estruturada a arquitetura com `SelectQueryBuilder`, `JoinBuilder`, `IntoSelections` para tuplas/arrays, e parsing de `"table as alias"`.
2. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).
3. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).
4. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).
5. **Documentação Integral do `.specs/`**: Especificadas todas as 9 fases do projeto cobrindo desde a AST até DDL, Migrations, Plugins e Multi-Dialetos (`REQ-AST-*` até `REQ-DIA-*`).

## Todos
- [x] TASK-SEL-001: Implement IntoSelections trait and tuple projections — Execute phase
- [x] TASK-SEL-002: Implement IntoTableSource and select_from constructor — Execute phase
- [x] TASK-SEL-003: Implement join methods and JoinBuilder — Execute phase
- [ ] TASK-SEL-004: Implement filtering methods (where_, where_ref, where_expr, or_where_*) — Execute phase
- [ ] TASK-SEL-005: Implement grouping, having, sorting, and pagination — Execute phase
- [ ] TASK-SEL-006: Implement CTEs (with) and set operations (union, intersect, except) — Execute phase
- [ ] TASK-SEL-007: Comprehensive integration tests for SelectQueryBuilder — Execute phase
