# Project State

## Status
- **Current Milestone**: Milestone 9 (Multi-Dialect Support: SQLite & MySQL Compilers) — Design Phase
- **Active Feature**: [09-multi-dialect-support](../features/09-multi-dialect-support/spec.md)
- **Current Phase**: Design
- **Active Task**: Design Milestone 9
- **Blockers**: None

## Cost
- Milestones 1, 2, 3, 4, 5, 6, 7 e 8 implementadas integralmente com 100% dos requisitos REQ-AST-001..010, REQ-EXPR-001..010, REQ-SEL-001..010, REQ-MUT-001..010, REQ-EXEC-001..010, REQ-DDL-001..010, REQ-MIG-001..008 e REQ-PLUG-001..006 satisfeitos.
- 68 testes de integração e unitários passando em 0.02s.
- `cargo clippy -- -D warnings` limpo sem nenhum aviso.
- Footprint control validado: `target/` em ~388MB, zero `.pdb`s descontrolados, 100% livre de macros procedurais.

## Recent Progress (Last 10)
- 2026-10-29 TASK-PLUG-005 complete. Gate: 68/68 pass. Milestone 8 100% Complete! [REQ-PLUG-001..006]
- 2026-10-29 TASK-PLUG-004 complete. Gate: test_with_schema_plugin pass. [REQ-PLUG-005]
- 2026-10-29 TASK-PLUG-003 complete. Gate: test_deduplicate_joins_plugin pass. [REQ-PLUG-004]
- 2026-10-29 TASK-PLUG-002 complete. Gate: test_camel_case_plugin pass. [REQ-PLUG-002] [REQ-PLUG-003]
- 2026-10-29 TASK-PLUG-001 complete. Gate: test_plugin_pipeline pass. [REQ-PLUG-001] [REQ-PLUG-006]
- 2026-10-29 Milestone 8 designed: design.md and tasks.md created. [REQ-PLUG-001..006]
- 2026-10-29 TASK-MIG-005 complete. Gate: 63/63 pass. Milestone 7 100% Complete! [REQ-MIG-001..008]
- 2026-10-29 TASK-MIG-004 complete. Gate: test_migrator_execution pass. [REQ-MIG-001] [REQ-MIG-007] [REQ-MIG-008]
- 2026-10-29 TASK-MIG-003 complete. Gate: test_file_migration_provider pass. [REQ-MIG-003]
- 2026-10-29 TASK-MIG-002 complete. Gate: test_simple_migration_provider pass. [REQ-MIG-002] [REQ-MIG-004]

## Decisions (Recent 5)
1. **Conclusão da Milestone 8 (Plugin Architecture & Built-in Plugins)**: Implementado o pipeline completo de plugins com `KyselyPlugin` (`transform_query` e `transform_result`), execução bidirecional em `DefaultQueryExecutor` e `SingleConnectionExecutor`, `PluginExecutor` para encadeamento per-query (`builder.with_plugin(...)`), e plugins embutidos: `CamelCasePlugin` (mapeamento bidirecional zero-macro entre camelCase e snake_case), `DeduplicateJoinsPlugin` (eliminação de joins duplicados em queries dinâmicas) e `WithSchemaPlugin` (qualificação de esquemas em tabelas).
2. **Conclusão da Milestone 7 (Migrator, Migration Providers & PostgreSQL Advisory Locks)**: Implementado o sistema de migrações completo com `DialectAdapter` e `PostgresAdapter` para locking distribuído (`KYSELY_MIGRATION_LOCK_ID = 3853314791062309107`), `Migration` e `MigrationProvider` assíncronos, `SimpleMigrationProvider` e `FileMigrationProvider`, `Migrator` gerenciando a tabela `kysely_migration` com execução atômica via transações e rollbacks garantidos.
3. **Design da Milestone 7 (Migrator, Migration Providers & PostgreSQL Advisory Locks)**: Estruturada arquitetura com `DialectAdapter` e `PostgresAdapter` para advisory locks (`pg_advisory_lock(3853314791062309107)`), trait `Migration` e `MigrationProvider`, `SimpleMigrationProvider` (in-memory) e `FileMigrationProvider` (disco), e `Migrator` gerenciando a tabela `kysely_migration` com execução transacional atômica.
4. **Conclusão da Milestone 6 (Schema DDL Builders & PostgreSQL Introspection)**: Implementada a infraestrutura completa de DDL e introspecção com `CreateTableBuilder`, `ColumnDefinitionBuilder` (tipos, constraints, defaults, foreign keys), `AlterTableBuilder`, `DropTableBuilder`, `CreateIndexBuilder`, `DropIndexBuilder`, `CreateViewBuilder`, `DropViewBuilder`, `CreateTypeBuilder`, `DropTypeBuilder`, `SchemaModule` via `db.schema()`, e `PostgresIntrospector` com `information_schema` e `pg_catalog`.
5. **Design da Milestone 6 (Schema DDL Builders & PostgreSQL Introspection)**: Decomposição da arquitetura DDL em nós de AST dedicados (`CreateTableNode`, `ColumnDefinitionNode`, `AlterTableNode`, `CreateIndexNode`, `CreateViewNode`, `CreateTypeNode`), suporte a constraints de tabela e coluna, `SchemaModule` via `db.schema()`, e `PostgresIntrospector` com `information_schema` e `pg_catalog`.

## Todos
- [x] TASK-PLUG-001: Implement KyselyPlugin Trait and Executor Pipeline — Done
- [x] TASK-PLUG-002: Implement CamelCasePlugin — Done
- [x] TASK-PLUG-003: Implement DeduplicateJoinsPlugin — Done
- [x] TASK-PLUG-004: Implement WithSchemaPlugin — Done
- [x] TASK-PLUG-005: Comprehensive Integration Tests for Plugin System — Done
