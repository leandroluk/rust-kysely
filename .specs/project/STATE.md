# Project State

## Status
- **Current Milestone**: Milestone 2 (Expressions, ExpressionBuilder & Raw SQL) — Em Execução
- **Active Feature**: [02-expressions-raw-sql](../features/02-expressions-raw-sql/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-EXPR-002
- **Blockers**: None

## Cost
- Milestone 1 implementada integralmente com 100% dos requisitos REQ-AST-001 a REQ-AST-010 satisfeitos.
- Design e decomposição em 7 tarefas atômicas concluídos para a Milestone 2 (`TASK-EXPR-001` a `TASK-EXPR-007`).
- 16 testes de integração e unitários passando em 0.00s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~253MB, zero `.pdb`s descontrolados, sem macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-EXPR-001 complete. Gate: 16/16 pass. [REQ-EXPR-001] [REQ-EXPR-002]
- 2026-10-29 TASK-AST-008 complete. Gate: 15/15 pass. Footprint: 253 MB. [REQ-AST-010]
- 2026-10-29 TASK-AST-007 complete. Gate: 11/11 pass. [REQ-AST-009]
- 2026-10-29 TASK-AST-006 complete. Gate: 10/10 pass. [REQ-AST-008]
- 2026-10-29 TASK-AST-005 complete. Gate: 9/9 pass. [REQ-AST-006] [REQ-AST-007]
- 2026-10-29 TASK-AST-004 complete. Gate: 7/7 pass. [REQ-AST-001] [REQ-AST-002]
- 2026-10-29 TASK-AST-003 complete. Gate: 6/6 pass. [REQ-AST-001] [REQ-AST-004]
- 2026-10-29 TASK-AST-002 complete. Gate: 4/4 pass. [REQ-AST-001] [REQ-AST-003]
- 2026-10-29 TASK-AST-001 complete. Gate: 3/3 pass. [REQ-AST-005] [REQ-AST-001]

## Decisions (Recent 5)
1. **Design da Milestone 2 (Expressions & Raw SQL)**: Estruturada a arquitetura com trait `Expression<T>`, `ExpressionWrapper<T>`, `IntoValue` sem `.into()`, construtor fluente `sql(...)`, namespace `sql::*`, `ExpressionBuilder` (`eb`) e `FunctionModule` (`eb.fn()`).
2. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).
3. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).
4. **Documentação Integral do `.specs/`**: Especificadas todas as 9 fases do projeto cobrindo desde a AST até DDL, Migrations, Plugins e Multi-Dialetos (`REQ-AST-*` até `REQ-DIA-*`).
5. **Tuplas para `.select(...)` Heterogêneo**: Adotada a trait `IntoSelections` para suportar strings e expressões juntas em uma única chamada de `.select((col1, col2, expr))` sem quebrar o encadeamento.

## Todos
- [x] TASK-EXPR-001: Implement Expression trait, ExpressionWrapper, FunctionNode and IntoValue — Execute phase
- [ ] TASK-EXPR-002: Implement sql fluent builder (RawBuilder) — Execute phase
- [ ] TASK-EXPR-003: Implement sql helper functions (raw, id, table, lit, join) — Execute phase
- [ ] TASK-EXPR-004: Implement ExpressionBuilder boolean combinators and comparisons — Execute phase
- [ ] TASK-EXPR-005: Implement eb.between and eb.exists — Execute phase
- [ ] TASK-EXPR-006: Implement FunctionModule (eb.fn / db.fn) — Execute phase
- [ ] TASK-EXPR-007: Integration tests for expressions and raw SQL — Execute phase
