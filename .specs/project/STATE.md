# Project State

## Status
- **Current Milestone**: Milestone 1 (Core AST & Compilador PostgreSQL) — Em Execução
- **Active Feature**: [01-core-ast-compiler](../features/01-core-ast-compiler/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-AST-005
- **Blockers**: None

## Cost
- Documentação completa do `.specs/` realizada (9 especificações formais de features com requisitos REQ-* mapeados).
- Design e quebra em 8 tarefas atômicas concluídos para a Milestone 1 (`TASK-AST-001` a `TASK-AST-008`).
- Footprint control e harness de teste inicial validados (`target/` ~250MB, zero `.pdb`s descontrolados).

## Recent Progress (Last 10)
- 2026-10-29 TASK-AST-004 complete. Gate: 7/7 pass. [REQ-AST-001] [REQ-AST-002]
- 2026-10-29 TASK-AST-003 complete. Gate: 6/6 pass. [REQ-AST-001] [REQ-AST-004]
- 2026-10-29 TASK-AST-002 complete. Gate: 4/4 pass. [REQ-AST-001] [REQ-AST-003]
- 2026-10-29 TASK-AST-001 complete. Gate: 3/3 pass. [REQ-AST-005] [REQ-AST-001]

## Decisions (Recent 5)
1. **Design e Decomposição em Tasks da Milestone 1**: Definida a arquitetura completa do AST em enum `OperationNode`, identificadores zero-alocação com `CompactStr`, otimizações via `SmallVec`, CoW com `Arc::make_mut` e compilador de dois níveis (`DefaultQueryCompiler` ANSI e `PostgresQueryCompiler`).
2. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).
3. **Documentação Integral do `.specs/`**: Especificadas todas as 9 fases do projeto cobrindo desde a AST até DDL, Migrations, Plugins e Multi-Dialetos (`REQ-AST-*` até `REQ-DIA-*`).
4. **Tuplas para `.select(...)` Heterogêneo**: Adotada a trait `IntoSelections` para suportar strings e expressões juntas em uma única chamada de `.select((col1, col2, expr))` sem quebrar o encadeamento.
5. **Eliminação do `.into()` em Mutations**: Validações de inserção e atualização utilizam `val: impl IntoValue` com métodos `.set(col, val)` e closure builders `.values(|v| ...)`.

## Todos
- [x] TASK-AST-001: Implement QueryId generator and Value scalar representation — Execute phase
- [x] TASK-AST-002: Implement Terminal & Core Identifier AST nodes — Execute phase
- [x] TASK-AST-003: Implement Clause and Expression AST nodes — Execute phase
- [x] TASK-AST-004: Implement Query AST nodes with Copy-on-Write support — Execute phase
- [ ] TASK-AST-005: Implement OperationNodeVisitor and OperationNodeTransformer traits — Execute phase
- [ ] TASK-AST-006: Implement CompiledQuery and DefaultQueryCompiler — Execute phase
- [ ] TASK-AST-007: Implement PostgresQueryCompiler — Execute phase
- [ ] TASK-AST-008: Comprehensive Integration Tests and Footprint Verification — Execute phase
