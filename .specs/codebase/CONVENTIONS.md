# Conventions

## Code & Design Conventions
- **Requirement Traceability**: Every feature requirement has a `REQ-XXX` ID in `.specs/features/*/spec.md`. Implementing code includes `// @spec REQ-XXX` comments, and commits include `[REQ-XXX]`.
- **Zero Macros Policy**:
  - No proc-macro derives or declarative macro DSLs.
  - Queries are constructed using standard Rust method chaining, closures (`|b| ...`), traits, and typed structs.
- **Idiomatic Naming Mapping**:
  - TypeScript camelCase methods map to Rust snake_case:
    - `selectFrom` -> `select_from`
    - `innerJoin` -> `inner_join`
    - `selectAll` -> `select_all`
    - `orderBy` -> `order_by`
    - `groupBy` -> `group_by`
    - `onConflict` -> `on_conflict`
    - `where` -> `where_` (to avoid Rust keyword collision)
    - `whereRef` -> `where_ref`
    - `executeTakeFirst` -> `execute_take_first`
    - `executeTakeFirstOrThrow` -> `execute_take_first_or_throw`
- **Memory & Allocation Discipline**:
  - Use `Arc` with Copy-on-Write (`Arc::make_mut`) for AST nodes during builder chaining.
  - Use `compact_str::CompactStr` or `&'static str` for identifiers to avoid unnecessary heap allocations.
  - Single integration test binary pattern (`tests/integration.rs`).
