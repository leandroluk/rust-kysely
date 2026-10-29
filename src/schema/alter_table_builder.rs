//! Fluent builder for ALTER TABLE statements.

use compact_str::CompactString;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::operation_node::{AlterTableAction, AlterTableNode, OperationNode, TableConstraintNode};
use crate::parser::IntoTableSource;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};
use crate::schema::column_definition_builder::ColumnDefinitionBuilder;

/// Fluent builder for modifying existing database tables.
// @spec REQ-DDL-005
#[derive(Clone)]
pub struct AlterTableBuilder {
    pub(crate) node: AlterTableNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for AlterTableBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlterTableBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for AlterTableBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl AlterTableBuilder {
    /// Creates a new `AlterTableBuilder`.
    pub fn new(table: impl IntoTableSource) -> Self {
        Self {
            node: AlterTableNode::new(table.into_table_source()),
            executor: None,
        }
    }

    /// Adds a new column to the table.
    // @spec REQ-DDL-005
    pub fn add_column<F>(
        mut self,
        name: impl Into<CompactString>,
        data_type: impl Into<CompactString>,
        builder: F,
    ) -> Self
    where
        F: FnOnce(ColumnDefinitionBuilder) -> ColumnDefinitionBuilder,
    {
        let col_builder = ColumnDefinitionBuilder::new(name, data_type);
        let built = builder(col_builder).build();
        self.node.actions.push(AlterTableAction::AddColumn(built));
        self
    }

    /// Drops an existing column from the table.
    // @spec REQ-DDL-005
    pub fn drop_column(mut self, name: impl Into<CompactString>) -> Self {
        self.node.actions.push(AlterTableAction::DropColumn {
            name: name.into(),
            if_exists: false,
        });
        self
    }

    /// Drops an existing column if it exists.
    // @spec REQ-DDL-005
    pub fn drop_column_if_exists(mut self, name: impl Into<CompactString>) -> Self {
        self.node.actions.push(AlterTableAction::DropColumn {
            name: name.into(),
            if_exists: true,
        });
        self
    }

    /// Alters the data type of an existing column.
    // @spec REQ-DDL-005
    pub fn alter_column(
        mut self,
        name: impl Into<CompactString>,
        data_type: impl Into<CompactString>,
    ) -> Self {
        self.node.actions.push(AlterTableAction::AlterColumnType {
            name: name.into(),
            data_type: data_type.into(),
        });
        self
    }

    /// Renames an existing column to a new name.
    // @spec REQ-DDL-005
    pub fn rename_column(
        mut self,
        old_name: impl Into<CompactString>,
        new_name: impl Into<CompactString>,
    ) -> Self {
        self.node.actions.push(AlterTableAction::RenameColumn {
            old_name: old_name.into(),
            new_name: new_name.into(),
        });
        self
    }

    /// Adds a foreign key constraint to the table.
    // @spec REQ-DDL-005
    pub fn add_foreign_key_constraint<F>(
        mut self,
        name: impl Into<CompactString>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
        target_table: impl Into<CompactString>,
        target_columns: impl IntoIterator<Item = impl Into<CompactString>>,
        builder: F,
    ) -> Self
    where
        F: FnOnce(
            crate::schema::constraint_builder::ForeignKeyConstraintBuilder,
        ) -> crate::schema::constraint_builder::ForeignKeyConstraintBuilder,
    {
        let fk_builder = crate::schema::constraint_builder::ForeignKeyConstraintBuilder::new(
            Some(name.into()),
            columns,
            target_table,
            target_columns,
        );
        let built = builder(fk_builder).build();
        self.node
            .actions
            .push(AlterTableAction::AddConstraint(built));
        self
    }

    /// Adds a unique constraint to the table.
    // @spec REQ-DDL-005
    pub fn add_unique_constraint(
        mut self,
        name: impl Into<CompactString>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
    ) -> Self {
        self.node.actions.push(AlterTableAction::AddConstraint(
            TableConstraintNode::Unique {
                name: Some(name.into()),
                columns: columns.into_iter().map(Into::into).collect(),
            },
        ));
        self
    }

    /// Drops a named constraint from the table.
    // @spec REQ-DDL-005
    pub fn drop_constraint(mut self, name: impl Into<CompactString>) -> Self {
        self.node.actions.push(AlterTableAction::DropConstraint {
            name: name.into(),
            if_exists: false,
        });
        self
    }

    /// Drops a named constraint if it exists.
    // @spec REQ-DDL-005
    pub fn drop_constraint_if_exists(mut self, name: impl Into<CompactString>) -> Self {
        self.node.actions.push(AlterTableAction::DropConstraint {
            name: name.into(),
            if_exists: true,
        });
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this ALTER TABLE statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::AlterTable(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the ALTER TABLE statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on AlterTableBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start altering a table.
pub fn alter_table(table: impl IntoTableSource) -> AlterTableBuilder {
    AlterTableBuilder::new(table)
}
