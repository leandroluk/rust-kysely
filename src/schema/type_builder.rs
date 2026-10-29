//! Fluent builders for CREATE TYPE AS ENUM and DROP TYPE statements.

use std::sync::Arc;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::operation_node::{CreateTypeNode, DropTypeNode, OperationNode};
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};

/// Fluent builder for CREATE TYPE AS ENUM statements.
// @spec REQ-DDL-008
#[derive(Clone)]
pub struct CreateTypeBuilder {
    pub(crate) node: CreateTypeNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for CreateTypeBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateTypeBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for CreateTypeBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl CreateTypeBuilder {
    /// Creates a new `CreateTypeBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: CreateTypeNode::new(name),
            executor: None,
        }
    }

    /// Configures the type as an ENUM with the specified variant values.
    // @spec REQ-DDL-008
    pub fn as_enum(mut self, values: impl IntoIterator<Item = impl Into<CompactString>>) -> Self {
        self.node.enum_values = values.into_iter().map(Into::into).collect();
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this CREATE TYPE statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::CreateType(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the CREATE TYPE statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on CreateTypeBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Fluent builder for DROP TYPE statements.
// @spec REQ-DDL-008
#[derive(Clone)]
pub struct DropTypeBuilder {
    pub(crate) node: DropTypeNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for DropTypeBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropTypeBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for DropTypeBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl DropTypeBuilder {
    /// Creates a new `DropTypeBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: DropTypeNode::new(name),
            executor: None,
        }
    }

    /// Adds the `IF EXISTS` clause.
    // @spec REQ-DDL-008
    pub fn if_exists(mut self) -> Self {
        self.node.if_exists = true;
        self
    }

    /// Adds the `CASCADE` option.
    // @spec REQ-DDL-008
    pub fn cascade(mut self) -> Self {
        self.node.cascade = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this DROP TYPE statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::DropType(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the DROP TYPE statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on DropTypeBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start creating a custom type.
pub fn create_type(name: impl Into<CompactString>) -> CreateTypeBuilder {
    CreateTypeBuilder::new(name)
}

/// Helper function to start dropping a custom type.
pub fn drop_type(name: impl Into<CompactString>) -> DropTypeBuilder {
    DropTypeBuilder::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_builder::select_from;
    use crate::query_compiler::DefaultQueryCompiler;
    use crate::raw_builder::sql;
    use crate::schema::index_builder::{create_index, drop_index};
    use crate::schema::view_builder::{create_view, drop_view};

    #[test]
    fn test_index_view_and_type_builders() {
        let compiler = DefaultQueryCompiler::new();

        // 1. CREATE INDEX with using and partial where
        let idx_q = create_index("users_age_idx")
            .if_not_exists()
            .on("users")
            .column("age")
            .using("btree")
            .where_(sql("age >= 18"))
            .compile(&compiler);

        assert_eq!(
            idx_q.sql,
            "CREATE INDEX IF NOT EXISTS \"users_age_idx\" ON \"users\" USING btree (\"age\") WHERE age >= 18"
        );

        // 2. DROP INDEX
        let drop_idx_q = drop_index("users_age_idx").if_exists().cascade().compile(&compiler);
        assert_eq!(drop_idx_q.sql, "DROP INDEX IF EXISTS \"users_age_idx\" CASCADE");

        // 3. CREATE VIEW
        let select_q = select_from("users").select(["id", "name"]);
        let view_q = create_view("active_users")
            .or_replace()
            .materialized()
            .as_query(select_q)
            .compile(&compiler);

        assert_eq!(
            view_q.sql,
            "CREATE OR REPLACE MATERIALIZED VIEW \"active_users\" AS SELECT \"id\", \"name\" FROM \"users\""
        );

        // 4. DROP VIEW
        let drop_view_q = drop_view("active_users").materialized().if_exists().compile(&compiler);
        assert_eq!(drop_view_q.sql, "DROP MATERIALIZED VIEW IF EXISTS \"active_users\"");

        // 5. CREATE TYPE
        let type_q = create_type("mood").as_enum(["happy", "neutral", "sad"]).compile(&compiler);
        assert_eq!(type_q.sql, "CREATE TYPE \"mood\" AS ENUM ('happy', 'neutral', 'sad')");

        // 6. DROP TYPE
        let drop_type_q = drop_type("mood").if_exists().cascade().compile(&compiler);
        assert_eq!(drop_type_q.sql, "DROP TYPE IF EXISTS \"mood\" CASCADE");
    }
}
