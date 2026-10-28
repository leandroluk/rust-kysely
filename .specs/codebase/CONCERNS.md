# Concerns & Technical Risks

## 1. Type Inference without TypeScript's Structural Typing
- In TypeScript, Kysely leverages string literal types (`'person.id'`) and mapped types (`Pick<DB[TB], K>`) to dynamically infer the shape of query results at compile time.
- In Rust, types are nominal. We solve this by providing:
  - Strong row mapping traits (`FromRow`, `TryFrom<Row>`).
  - Dynamic type-safe `Row` with `.get::<T>("col")?` and `.get_opt::<T>("col")?`.
  - `.execute_into::<T>()` for deserializing directly into user structs.
  - Optional table-marker structs (`PersonTable`) with typed column constants for compile-time column checking.

## 2. Dynamic AST Allocation Overhead
- Kysely creates lots of small AST nodes per query.
- In Rust, heap allocations (`Box`/`Arc`) could affect performance if unchecked.
- Mitigation: Use `CompactStr` for identifiers, `SmallVec` for arguments/selections (which inline 2-4 items on the stack), and `Arc::make_mut` so chaining in a single builder modifies nodes in-place.

## 3. Asynchronous Execution & Connection Pooling
- Unlike Node.js where connection pools are often external (`pg.Pool`), Rust needs clean async pooling.
- We support both direct connections (`tokio-postgres::Client`) and connection poolers (`deadpool-postgres` or an abstract `ConnectionProvider` trait) mirroring Kysely's `ConnectionProvider`.
