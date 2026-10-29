# Tasks: 06 — Schema DDL Builders & PostgreSQL Introspection

### TASK-DDL-001: Implement DDL AST Operation Nodes and Compiler Extensions [x]
- **REQ**: REQ-DDL-001, REQ-DDL-002, REQ-DDL-006, REQ-DDL-007, REQ-DDL-008, REQ-DDL-009
- **Symbols**: `CreateTableNode`, `ColumnDefinitionNode`, `AlterTableNode`, `DropTableNode`, `CreateIndexNode`, `CreateViewNode`, `CreateTypeNode`, `OnModifyForeignAction`
- **What**: Implement AST nodes for all DDL operations and extend `DefaultQueryCompiler` to compile DDL syntax with dialect quoting.
- **Where**: `src/operation_node/ddl_nodes.rs`, `src/operation_node/mod.rs`, `src/query_compiler/default_compiler.rs`
- **Depends on**: Milestone 5
- **Done when**: All DDL nodes compile into valid ANSI and PostgreSQL SQL with correct quoting and constraints.
- **Gate**: `cargo test test_ddl_compilation`

### TASK-DDL-002: Implement CreateTableBuilder and ColumnDefinitionBuilder [x]
- **REQ**: REQ-DDL-002, REQ-DDL-004
- **Symbols**: `CreateTableBuilder`, `ColumnDefinitionBuilder`, `.add_column()`, `.primary_key()`, `.not_null()`, `.unique()`, `.default_to()`, `.references()`, `.on_delete()`, `.on_update()`, `.check()`
- **What**: Implement fluent builders for table and column definitions with constraint chaining and foreign key referencing.
- **Where**: `src/schema/create_table_builder.rs`, `src/schema/column_definition_builder.rs`, `src/schema/mod.rs`
- **Depends on**: TASK-DDL-001
- **Done when**: Tables and column definitions fluently produce valid `CreateTableNode` AST and compile accurately.
- **Gate**: `cargo test test_create_table_builder`

### TASK-DDL-003: Implement Table Constraints on CreateTableBuilder [x]
- **REQ**: REQ-DDL-003
- **Symbols**: `add_primary_key_constraint`, `add_foreign_key_constraint`, `add_unique_constraint`, `add_check_constraint`
- **What**: Implement multi-column table-level constraints for primary keys, foreign keys, unique keys, and checks.
- **Where**: `src/schema/create_table_builder.rs`, `src/schema/constraint_builder.rs`
- **Depends on**: TASK-DDL-002
- **Done when**: Multi-column constraints compile cleanly into table-level SQL constraints.
- **Gate**: `cargo test test_table_constraints`

### TASK-DDL-004: Implement AlterTableBuilder and DropTableBuilder [x]
- **REQ**: REQ-DDL-005, REQ-DDL-006
- **Symbols**: `AlterTableBuilder`, `DropTableBuilder`, `.alter_table()`, `.drop_table()`, `.add_column()`, `.drop_column()`, `.alter_column()`, `.rename_column()`, `.if_exists()`, `.cascade()`
- **What**: Implement fluent builders for altering tables (column addition/removal/modification/renaming, constraint addition/removal) and dropping tables.
- **Where**: `src/schema/alter_table_builder.rs`, `src/schema/drop_table_builder.rs`, `src/schema/mod.rs`
- **Depends on**: TASK-DDL-001
- **Done when**: ALTER TABLE and DROP TABLE statements compile correctly with IF EXISTS and CASCADE options.
- **Gate**: `cargo test test_alter_and_drop_table_builders`

### TASK-DDL-005: Implement Index, View, and Custom Type DDL builders
- **REQ**: REQ-DDL-007, REQ-DDL-008, REQ-DDL-009
- **Symbols**: `CreateIndexBuilder`, `DropIndexBuilder`, `CreateViewBuilder`, `DropViewBuilder`, `CreateTypeBuilder`, `DropTypeBuilder`
- **What**: Implement builders for creating and dropping indexes (with unique, using method, partial where), views (with or replace, materialized), and PostgreSQL custom types (enums).
- **Where**: `src/schema/index_builder.rs`, `src/schema/view_builder.rs`, `src/schema/type_builder.rs`, `src/schema/mod.rs`
- **Depends on**: TASK-DDL-001
- **Done when**: Index, view, and custom enum DDL compile accurately.
- **Gate**: `cargo test test_index_view_and_type_builders`

### TASK-DDL-006: Implement SchemaModule accessible via db.schema()
- **REQ**: REQ-DDL-001
- **Symbols**: `SchemaModule`, `db.schema()`, `.create_table()`, `.alter_table()`, `.drop_table()`, `.create_index()`, `.drop_index()`, `.create_view()`, `.drop_view()`, `.create_type()`, `.drop_type()`
- **What**: Implement `SchemaModule` attached to `Kysely`, allowing all DDL builders to execute directly via `.execute().await`.
- **Where**: `src/schema/mod.rs`, `src/kysely.rs`, `src/lib.rs`
- **Depends on**: TASK-DDL-002, TASK-DDL-003, TASK-DDL-004, TASK-DDL-005
- **Done when**: `db.schema().create_table(...).execute().await` executes through the driver seamlessly.
- **Gate**: `cargo test test_schema_module`

### TASK-DDL-007: Implement PostgresIntrospector and schema metadata extraction
- **REQ**: REQ-DDL-010
- **Symbols**: `PostgresIntrospector`, `DatabaseIntrospector`, `TableMetadata`, `ColumnMetadata`
- **What**: Implement introspection querying `information_schema` and `pg_catalog` to extract table schemas, column data types, nullability, defaults, and primary/foreign keys.
- **Where**: `src/introspection/mod.rs`, `src/dialect/postgres/introspector.rs`, `src/lib.rs`
- **Depends on**: TASK-DDL-006
- **Done when**: Introspector queries return structured `TableMetadata` and `ColumnMetadata`.
- **Gate**: `cargo test test_postgres_introspection`

### TASK-DDL-008: Comprehensive integration tests for Schema DDL & Introspection
- **REQ**: REQ-DDL-001..010
- **Symbols**: Public exports in `src/lib.rs`
- **What**: End-to-end integration tests verifying DDL generation, execution, table creation, alteration, indexing, custom types, and schema introspection.
- **Where**: `tests/integration.rs`, `src/lib.rs`
- **Depends on**: TASK-DDL-006, TASK-DDL-007
- **Done when**: Full test suite passes, zero warnings, footprint clean.
- **Gate**: `cargo test --test integration`, `cargo clippy -- -D warnings`
