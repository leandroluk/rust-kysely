# Tasks: 02 — Expressions, ExpressionBuilder (`eb`) & Macro-Free Raw SQL (`sql`)

### TASK-EXPR-001: Implement Expression trait, ExpressionWrapper, FunctionNode and IntoValue [x]
- **REQ**: REQ-EXPR-001, REQ-EXPR-002
- **Symbols**: `Expression`, `ExpressionWrapper`, `IntoValue`, `FunctionNode`
- **What**: Implement `Expression<T>` trait, `ExpressionWrapper<T>`, `IntoValue` trait for automatic scalar parameter binding without `.into()`, and `FunctionNode` in the AST with visitor and compiler support.
- **Where**: `src/operation_node/value.rs`, `src/operation_node/expression_nodes.rs`, `src/operation_node/mod.rs`, `src/operation_node/visitor.rs`, `src/operation_node/transformer.rs`, `src/query_compiler/default_compiler.rs`, `src/expression/mod.rs`, `src/expression/wrapper.rs`
- **Depends on**: none
- **Done when**: `Expression` trait converts seamlessly to `Arc<OperationNode>` and `IntoValue` allows passing scalar primitives without `.into()`.
- **Gate**: `cargo test test_expression_wrapper test_into_value`

### TASK-EXPR-002: Implement sql fluent builder (RawBuilder) [x]
- **REQ**: REQ-EXPR-002
- **Symbols**: `RawBuilder`, `sql`
- **What**: Implement `sql("...")` constructor and `RawBuilder` with `.bind(val)` and `.append(item)`.
- **Where**: `src/raw_builder/mod.rs`, `src/raw_builder/raw_builder.rs`
- **Depends on**: TASK-EXPR-001
- **Done when**: `sql("SELECT * FROM u WHERE id = ").bind(42)` produces valid parameterized `RawNode` and compiles to expected SQL with parameters.
- **Gate**: `cargo test test_raw_builder`

### TASK-EXPR-003: Implement sql helper functions (raw, id, table, lit, join) [x]
- **REQ**: REQ-EXPR-003, REQ-EXPR-004, REQ-EXPR-005
- **Symbols**: `sql::raw`, `sql::id`, `sql::table`, `sql::lit`, `sql::join`
- **What**: Implement standalone helpers in namespace `sql::*` for verbatim raw snippets, quoted identifiers, table nodes, escaped literals, and delimiter-separated joins.
- **Where**: `src/raw_builder/helpers.rs`, `src/raw_builder/mod.rs`
- **Depends on**: TASK-EXPR-002
- **Done when**: Helpers produce the correct AST nodes and compile accurately through `PostgresQueryCompiler`.
- **Gate**: `cargo test test_sql_helpers`

### TASK-EXPR-004: Implement ExpressionBuilder boolean combinators and comparisons
- **REQ**: REQ-EXPR-006, REQ-EXPR-007
- **Symbols**: `ExpressionBuilder`, `eb.or`, `eb.and`, `eb.not`, `eb.cmp`, `eb.cmp_ref`
- **What**: Implement `ExpressionBuilder` struct providing boolean combinations and comparisons for column-to-value and column-to-column.
- **Where**: `src/expression/expression_builder.rs`, `src/expression/mod.rs`
- **Depends on**: TASK-EXPR-001
- **Done when**: `eb.or([eb.cmp("age", ">", 18), eb.cmp("name", "=", "Bob")])` produces parenthesized boolean AST.
- **Gate**: `cargo test test_eb_boolean test_eb_cmp`

### TASK-EXPR-005: Implement eb.between and eb.exists
- **REQ**: REQ-EXPR-008, REQ-EXPR-009
- **Symbols**: `eb.between`, `eb.between_symmetric`, `eb.exists`, `eb.not_exists`
- **What**: Implement `BETWEEN`, `BETWEEN SYMMETRIC`, and subquery `EXISTS` / `NOT EXISTS` combinators on `ExpressionBuilder`.
- **Where**: `src/expression/expression_builder.rs`
- **Depends on**: TASK-EXPR-004
- **Done when**: `eb.between("age", 18, 65)` and `eb.exists(...)` compile properly.
- **Gate**: `cargo test test_eb_between test_eb_exists`

### TASK-EXPR-006: Implement FunctionModule (eb.fn / db.fn)
- **REQ**: REQ-EXPR-010
- **Symbols**: `FunctionModule`, `eb.fn()`, `count`, `count_all`, `max`, `min`, `avg`, `sum`, `coalesce`
- **What**: Implement `FunctionModule` providing standard SQL functions with `.as_("alias")` support.
- **Where**: `src/expression/function_module.rs`, `src/expression/mod.rs`
- **Depends on**: TASK-EXPR-005
- **Done when**: `eb.fn().count("id").as_("total")` compiles to `COUNT("id") AS "total"`.
- **Gate**: `cargo test test_function_module`

### TASK-EXPR-007: Integration tests for expressions and raw SQL
- **REQ**: REQ-EXPR-001, REQ-EXPR-002, REQ-EXPR-003, REQ-EXPR-004, REQ-EXPR-005, REQ-EXPR-006, REQ-EXPR-007, REQ-EXPR-008, REQ-EXPR-009, REQ-EXPR-010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests verifying expressions, raw SQL building, and closures in `tests/integration.rs`.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-EXPR-006
- **Done when**: All integration tests pass and `cargo clippy -- -D warnings` succeeds.
- **Gate**: `cargo test --test integration`
