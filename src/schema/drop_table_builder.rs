//! Fluent builder for DROP TABLE statements.

use std::sync::Arc;

use crate::driver::DriverError;
use crate::operation_node::{DropTableNode, OperationNode};
use crate::parser::IntoTableSource;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};

/// Fluent builder for dropping database tables.
// @spec REQ-DDL-006
#[derive(Clone)]
pub struct DropTableBuilder {
    pub(crate) node: DropTableNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for DropTableBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropTableBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for DropTableBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl DropTableBuilder {
    /// Creates a new `DropTableBuilder`.
    pub fn new(table: impl IntoTableSource) -> Self {
        Self {
            node: DropTableNode::new(table.into_table_source()),
            executor: None,
        }
    }

    /// Adds the `IF EXISTS` clause.
    // @spec REQ-DDL-006
    pub fn if_exists(mut self) -> Self {
        self.node.if_exists = true;
        self
    }

    /// Adds the `CASCADE` option to drop dependent objects.
    // @spec REQ-DDL-006
    pub fn cascade(mut self) -> Self {
        self.node.cascade = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this DROP TABLE statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::DropTable(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the DROP TABLE statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on DropTableBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start dropping a table.
pub fn drop_table(table: impl IntoTableSource) -> DropTableBuilder {
    DropTableBuilder::new(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_compiler::DefaultQueryCompiler;
    use crate::schema::alter_table_builder::alter_table;

    #[test]
    fn test_alter_and_drop_table_builders() {
        let compiler = DefaultQueryCompiler::new();

        // 1. ALTER TABLE
        let alter_q = alter_table("users")
            .add_column("age", "int4", |c| c.not_null())
            .drop_column_if_exists("legacy_col")
            .alter_column("bio", "text")
            .rename_column("first_name", "given_name")
            .add_unique_constraint("uq_users_given_name", ["given_name"])
            .drop_constraint_if_exists("old_uq")
            .compile(&compiler);

        assert_eq!(
            alter_q.sql,
            "ALTER TABLE \"users\" ADD COLUMN \"age\" int4 NOT NULL, DROP COLUMN IF EXISTS \"legacy_col\", ALTER COLUMN \"bio\" TYPE text, RENAME COLUMN \"first_name\" TO \"given_name\", ADD CONSTRAINT \"uq_users_given_name\" UNIQUE (\"given_name\"), DROP CONSTRAINT IF EXISTS \"old_uq\""
        );

        // 2. DROP TABLE
        let drop_q = drop_table("users").if_exists().cascade().compile(&compiler);
        assert_eq!(drop_q.sql, "DROP TABLE IF EXISTS \"users\" CASCADE");
    }
}
