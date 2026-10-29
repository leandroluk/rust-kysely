# Project State

## Status
- **Current Milestone**: Milestone 6 (Schema DDL Builders & PostgreSQL Introspection) — Design & Execute Phase
- **Active Feature**: [06-schema-ddl-introspection](../features/06-schema-ddl-introspection/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-DDL-001
- **Blockers**: None

## Cost
- Milestones 1, 2, 3, 4 e 5 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010, REQ-MUT-001..010 e REQ-EXEC-001..010 satisfeitos.
- Design e decomposição em 8 tarefas atômicas concluídos para a Milestone 6 (`TASK-DDL-001` a `TASK-DDL-008`).
- 50 testes de integração e unitários passando em 0.04s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~362MB, zero `.pdb`s descontrolados, 100% livre de macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 Milestone 6 designed: design.md and tasks.md created. [REQ-DDL-001..010]
- 2026-10-29 TASK-DRV-007 complete. Gate: 50/50 pass. Milestone 5 100% Complete! [REQ-EXEC-001..010]
- 2026-10-29 TASK-DRV-006 complete. Gate: 49/49 pass. [REQ-EXEC-002] [REQ-EXEC-003]
- 2026-10-29 TASK-DRV-005 complete. Gate: 48/48 pass. [REQ-EXEC-009] [REQ-EXEC-010]
- 2026-10-29 TASK-DRV-004 complete. Gate: 47/47 pass. [REQ-EXEC-003] [REQ-EXEC-004] [REQ-EXEC-006]
- 2026-10-29 TASK-DRV-003 complete. Gate: 46/46 pass. [REQ-EXEC-004] [REQ-EXEC-007] [REQ-EXEC-008]
- 2026-10-29 TASK-DRV-002 complete. Gate: 45/45 pass. [REQ-EXEC-005] [REQ-EXEC-006]
- 2026-10-29 TASK-DRV-001 complete. Gate: 44/44 pass. [REQ-EXEC-001]
- 2026-10-29 Milestone 5 designed: design.md and tasks.md created. [REQ-EXEC-001..010]
- 2026-10-29 TASK-MUT-007 complete. Gate: 43/43 pass. Milestone 4 100% Complete! [REQ-MUT-001..010]

## Decisions (Recent 5)
1. **Design da Milestone 6 (Schema DDL Builders & PostgreSQL Introspection)**: Decomposição da arquitetura DDL em nós de AST dedicados (`CreateTableNode`, `ColumnDefinitionNode`, `AlterTableNode`, `CreateIndexNode`, `CreateViewNode`, `CreateTypeNode`), suporte a constraints de tabela e coluna, `SchemaModule` via `db.schema()`, e `PostgresIntrospector` com `information_schema` e `pg_catalog`.
2. **Conclusão da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Implementadas traits `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com getters tipados (`get::<T>`, `get_opt::<T>`, case-insensitive) e deserialização serde (`into_record`), `QueryExecutor` e `SingleConnectionExecutor`, facade `Kysely`, transações atômicas com `execute(move |trx| ...)` e savepoints aninhados (`savepoint_block`), e `PostgresDriver` com `tokio-postgres` e conversão dinâmica de parâmetros.
3. **Design da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Estruturada a arquitetura com traits assíncronas `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com extração tipada e serde, `QueryExecutor`, facade `Kysely`, transações automáticas e savepoints.
4. **Design e Conclusão da Milestone 4 (Mutation Query Builders)**: Implementados `InsertQueryBuilder`, `UpdateQueryBuilder`, `DeleteQueryBuilder`, `OnConflictBuilder`, e `RowValues` com suporte a `set`, `values_batch`, serde structs (`values_record`), `excluded` references, PostgreSQL `ON CONFLICT` e `RETURNING` clauses.
5. **Design e Conclusão da Milestone 3 (SelectQueryBuilder)**: Implementado `SelectQueryBuilder` com projections heterogêneas via `IntoSelections`, joins com closure `JoinBuilder`, filtros encadeáveis `AND`/`OR`, agrupamento, ordenação com nulls, paginação com limit/offset, CTEs (`with`/`with_recursive`) e operações de conjunto (`union`, `union_all`, `intersect`, `except`).

## Todos
- [ ] TASK-DDL-001: Implement DDL AST Operation Nodes and Compiler Extensions — Execute phase
- [ ] TASK-DDL-002: Implement CreateTableBuilder and ColumnDefinitionBuilder — Execute phase
- [ ] TASK-DDL-003: Implement Table Constraints on CreateTableBuilder — Execute phase
- [ ] TASK-DDL-004: Implement AlterTableBuilder and DropTableBuilder — Execute phase
- [ ] TASK-DDL-005: Implement Index, View, and Custom Type DDL builders — Execute phase
- [ ] TASK-DDL-006: Implement SchemaModule accessible via db.schema() — Execute phase
- [ ] TASK-DDL-007: Implement PostgresIntrospector and schema metadata extraction — Execute phase
- [ ] TASK-DDL-008: Comprehensive integration tests for Schema DDL & Introspection — Execute phase
