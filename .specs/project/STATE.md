# Project State

## Status
- **Current Milestone**: Milestone 7 (Migrator, Migration Providers & PostgreSQL Advisory Locks) — Design & Execute Phase
- **Active Feature**: [07-migrator-advisory-locks](../features/07-migrator-advisory-locks/spec.md)
- **Current Phase**: Execute
- **Active Task**: TASK-MIG-001
- **Blockers**: None

## Cost
- Milestones 1, 2, 3, 4, 5 e 6 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010, REQ-MUT-001..010, REQ-EXEC-001..010 e REQ-DDL-001..010 satisfeitos.
- Design e decomposição em 5 tarefas atômicas concluídos para a Milestone 7 (`TASK-MIG-001` a `TASK-MIG-005`).
- 58 testes de integração e unitários passando em 0.01s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~364MB, zero `.pdb`s descontrolados, 100% livre de macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 Milestone 7 designed: design.md and tasks.md created. [REQ-MIG-001..008]
- 2026-10-29 TASK-DDL-008 complete. Gate: 58/58 pass. Milestone 6 100% Complete! [REQ-DDL-001..010]
- 2026-10-29 TASK-DDL-007 complete. Gate: 57/57 pass. [REQ-DDL-010]
- 2026-10-29 TASK-DDL-006 complete. Gate: 56/56 pass. [REQ-DDL-001]
- 2026-10-29 TASK-DDL-005 complete. Gate: 55/55 pass. [REQ-DDL-007..009]
- 2026-10-29 TASK-DDL-004 complete. Gate: 54/54 pass. [REQ-DDL-005] [REQ-DDL-006]
- 2026-10-29 TASK-DDL-003 complete. Gate: 53/53 pass. [REQ-DDL-003]
- 2026-10-29 TASK-DDL-002 complete. Gate: 52/52 pass. [REQ-DDL-002] [REQ-DDL-004]
- 2026-10-29 TASK-DDL-001 complete. Gate: 51/51 pass. [REQ-DDL-001] [REQ-DDL-002] [REQ-DDL-006..009]
- 2026-10-29 Milestone 6 designed: design.md and tasks.md created. [REQ-DDL-001..010]

## Decisions (Recent 5)
1. **Design da Milestone 7 (Migrator, Migration Providers & PostgreSQL Advisory Locks)**: Estruturada arquitetura com `DialectAdapter` e `PostgresAdapter` para advisory locks (`pg_advisory_lock(3853314791062309107)`), trait `Migration` e `MigrationProvider`, `SimpleMigrationProvider` (in-memory) e `FileMigrationProvider` (disco), e `Migrator` gerenciando a tabela `kysely_migration` com execução transacional atômica.
2. **Conclusão da Milestone 6 (Schema DDL Builders & PostgreSQL Introspection)**: Implementada a infraestrutura completa de DDL e introspecção com `CreateTableBuilder`, `ColumnDefinitionBuilder` (tipos, constraints, defaults, foreign keys), `AlterTableBuilder`, `DropTableBuilder`, `CreateIndexBuilder`, `DropIndexBuilder`, `CreateViewBuilder`, `DropViewBuilder`, `CreateTypeBuilder`, `DropTypeBuilder`, `SchemaModule` via `db.schema()`, e `PostgresIntrospector` com `information_schema` e `pg_catalog`.
3. **Design da Milestone 6 (Schema DDL Builders & PostgreSQL Introspection)**: Decomposição da arquitetura DDL em nós de AST dedicados (`CreateTableNode`, `ColumnDefinitionNode`, `AlterTableNode`, `CreateIndexNode`, `CreateViewNode`, `CreateTypeNode`), suporte a constraints de tabela e coluna, `SchemaModule` via `db.schema()`, e `PostgresIntrospector` com `information_schema` e `pg_catalog`.
4. **Conclusão da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Implementadas traits `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com getters tipados (`get::<T>`, `get_opt::<T>`, case-insensitive) e deserialização serde (`into_record`), `QueryExecutor` e `SingleConnectionExecutor`, facade `Kysely`, transações atômicas com `execute(move |trx| ...)` e savepoints aninhados (`savepoint_block`), e `PostgresDriver` com `tokio-postgres` e conversão dinâmica de parâmetros.
5. **Design da Milestone 5 (PostgreSQL Driver & Execution Engine)**: Estruturada a arquitetura com traits assíncronas `Driver` e `DatabaseConnection`, `QueryResult`, `Row` com extração tipada e serde, `QueryExecutor`, facade `Kysely`, transações automáticas e savepoints.

## Todos
- [ ] TASK-MIG-001: Implement DialectAdapter, PostgresAdapter, and Advisory Locking — Execute phase
- [ ] TASK-MIG-002: Implement Migration, MigrationProvider, and SimpleMigrationProvider — Execute phase
- [ ] TASK-MIG-003: Implement FileMigrationProvider for disk migrations — Execute phase
- [ ] TASK-MIG-004: Implement Migrator engine and state tracking — Execute phase
- [ ] TASK-MIG-005: Comprehensive integration tests for Migrator & Advisory Locks — Execute phase
