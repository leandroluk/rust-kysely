# Stack & Footprint Policy

## Language & Toolchain
- **Rust Edition**: 2021 (MSRV 1.85+)
- **Async Runtime**: Tokio (`tokio::sync`, `tokio::rt`)
- **Initial Dialect**: PostgreSQL (`tokio-postgres`)

## Disk & `target/` Footprint Control (Strict Policy)
Rust workspaces on Windows/MSVC quickly balloon into tens of gigabytes due to `.pdb` debug symbols, incremental compilation caches, and heavy proc-macro trees. To prevent disk bloat:
1. **Zero Debug Symbols in Dev/Test**: Configured via `.cargo/config.toml` and root `Cargo.toml`:
   - `debug = 0` and `strip = "debuginfo"` in `[profile.dev]` and `[profile.test]` (eliminates massive `.pdb` files).
   - `incremental = false` (prevents unbounded growth in `target/debug/incremental/`).
2. **100% Macro-Free Architecture**:
   - Zero procedural macros (`no syn`, `no quote`, `no proc-macro2`, `no macro_rules!` DSLs).
   - All builders, queries, expressions, and schema operations use pure Rust structs, traits, and closure builders (`|eb| ...`).
3. **Single Integration Test Binary Pattern**:
   - Avoid dozens of separate test binaries in `tests/*.rs`. Integration tests are consolidated into `tests/integration.rs` with submodules.
