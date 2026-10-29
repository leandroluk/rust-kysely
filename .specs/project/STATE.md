# Project State

## Status
- **Current Milestone**: Milestone 1 (Core AST & Compilador PostgreSQL) — Pronto para execução
- **Active Feature**: [01-core-ast-compiler](../features/01-core-ast-compiler/spec.md)
- **Blockers**: None

## Cost
- Documentação completa do `.specs/` realizada (8 especificações formais de features com requisitos REQ-* mapeados).
- Footprint control e harness de teste inicial validados (`target/` ~250MB, zero `.pdb`s descontrolados).

## Decisions (Recent 5)
1. **Documentação Integral do `.specs/`**: Especificadas todas as 8 fases do projeto cobrindo desde a AST até DDL, Migrations e Plugins, garantindo rastreabilidade total de requisitos (`REQ-AST-*` até `REQ-PLUG-*`).
2. **Tuplas para `.select(...)` Heterogêneo**: Adotada a trait `IntoSelections` para suportar strings e expressões juntas em uma única chamada de `.select((col1, col2, expr))` sem quebrar o encadeamento.
3. **Eliminação do `.into()` em Mutations**: Validações de inserção e atualização utilizam `val: impl IntoValue` com métodos `.set(col, val)` e closure builders `.values(|v| ...)`.
4. **Target Footprint Discipline**: Símbolos `.pdb` suprimidos via `debug = 0`, `strip = "debuginfo"` e `incremental = false` em `.cargo/config.toml` e `Cargo.toml`.
5. **Zero Macros Mandate**: Arquitetura 100% livre de proc-macros, garantindo tempos de compilação instantâneos e clareza no compilador.
