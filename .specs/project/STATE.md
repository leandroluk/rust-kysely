# Project State

## Status
- **Current Milestone**: Milestone 1 (Core AST & Compilador PostgreSQL) — Pronto para execução
- **Active Feature**: [01-core-ast-compiler](../features/01-core-ast-compiler/spec.md)
- **Blockers**: None

## Cost
- Documentação completa do `.specs/` realizada (9 especificações formais de features com requisitos REQ-* mapeados).
- Adicionado escopo oficial para os 5 dialetos embutidos do Kysely (PostgreSQL, MySQL, MSSQL, SQLite, PGlite).
- Footprint control e harness de teste inicial validados (`target/` ~250MB, zero `.pdb`s descontrolados).

## Decisions (Recent 5)
1. **Suporte aos 5 Dialetos Oficiais do Kysely**: Registrado formalmente que o `rust-kysely` suportará todos os 5 dialetos embutidos (PostgreSQL como baseline inicial, seguido por MySQL, MSSQL, SQLite e PGlite) via Cargo features (`postgres`, `mysql`, `mssql`, `sqlite`, `pglite`).
2. **Documentação Integral do `.specs/`**: Especificadas todas as 9 fases do projeto cobrindo desde a AST até DDL, Migrations, Plugins e Multi-Dialetos (`REQ-AST-*` até `REQ-DIA-*`).
3. **Tuplas para `.select(...)` Heterogêneo**: Adotada a trait `IntoSelections` para suportar strings e expressões juntas em uma única chamada de `.select((col1, col2, expr))` sem quebrar o encadeamento.
4. **Eliminação do `.into()` em Mutations**: Validações de inserção e atualização utilizam `val: impl IntoValue` com métodos `.set(col, val)` e closure builders `.values(|v| ...)`.
5. **Target Footprint Discipline**: Símbolos `.pdb` suprimidos via `debug = 0`, `strip = "debuginfo"` e `incremental = false` em `.cargo/config.toml` e `Cargo.toml`.
