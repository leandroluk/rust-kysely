//! Fluent builder for CREATE TABLE statements.

use std::sync::Arc;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::operation_node::{CreateTableNode, OperationNode};
use crate::parser::IntoTableSource;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};
use crate::schema::column_definition_builder::ColumnDefinitionBuilder;

/// Fluent builder for creating database tables.
// @spec REQ-DDL-002, REQ-DDL-003
#[derive(Clone)]
pub struct CreateTableBuilder {
    pub(crate) node: CreateTableNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for CreateTableBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateTableBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for CreateTableBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl CreateTableBuilder {
    /// Creates a new `CreateTableBuilder` for the specified table.
    pub fn new(table: impl IntoTableSource) -> Self {
        let table_source = table.into_table_source();
        let name_node = match table_source.as_ref() {
            OperationNode::Table(t) => Arc::new(OperationNode::Table(t.clone())),
            _ => table_source,
        };

        Self {
            node: CreateTableNode::new(name_node),
            executor: None,
        }
    }

    /// Adds the `IF NOT EXISTS` clause.
    // @spec REQ-DDL-002
    pub fn if_not_exists(mut self) -> Self {
        self.node.if_not_exists = true;
        self
    }

    /// Marks the table as `TEMPORARY`.
    // @spec REQ-DDL-002
    pub fn temporary(mut self) -> Self {
        self.node.temporary = true;
        self
    }

    /// Adds a column definition with a closure builder.
    // @spec REQ-DDL-002
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
        self.node.columns.push(built);
        self
    }

    /// Adds a table-level PRIMARY KEY constraint.
    // @spec REQ-DDL-003
    pub fn add_primary_key_constraint(
        mut self,
        name: impl Into<CompactString>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
    ) -> Self {
        self.node.constraints.push(crate::operation_node::TableConstraintNode::PrimaryKey {
            name: Some(name.into()),
            columns: columns.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Adds a table-level UNIQUE constraint.
    // @spec REQ-DDL-003
    pub fn add_unique_constraint(
        mut self,
        name: impl Into<CompactString>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
    ) -> Self {
        self.node.constraints.push(crate::operation_node::TableConstraintNode::Unique {
            name: Some(name.into()),
            columns: columns.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Adds a table-level CHECK constraint.
    // @spec REQ-DDL-003
    pub fn add_check_constraint(
        mut self,
        name: impl Into<CompactString>,
        expr: impl crate::expression::Expression,
    ) -> Self {
        self.node.constraints.push(crate::operation_node::TableConstraintNode::Check {
            name: Some(name.into()),
            expr: expr.to_operation_node(),
        });
        self
    }

    /// Adds a table-level FOREIGN KEY constraint with closure builder for cascading actions.
    // @spec REQ-DDL-003
    pub fn add_foreign_key_constraint<F>(
        mut self,
        name: impl Into<CompactString>,
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
        target_table: impl Into<CompactString>,
        target_columns: impl IntoIterator<Item = impl Into<CompactString>>,
        builder: F,
    ) -> Self
    where
        F: FnOnce(crate::schema::constraint_builder::ForeignKeyConstraintBuilder) -> crate::schema::constraint_builder::ForeignKeyConstraintBuilder,
    {
        let fk_builder = crate::schema::constraint_builder::ForeignKeyConstraintBuilder::new(
            Some(name.into()),
            columns,
            target_table,
            target_columns,
        );
        let built = builder(fk_builder).build();
        self.node.constraints.push(built);
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns a reference to the inner AST node.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::CreateTable(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the CREATE TABLE statement against the database.
    // @spec REQ-DDL-001
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on CreateTableBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start creating a table.
pub fn create_table(table: impl IntoTableSource) -> CreateTableBuilder {
    CreateTableBuilder::new(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::OnModifyForeignAction;
    use crate::query_compiler::DefaultQueryCompiler;

    #[test]
    fn test_create_table_builder() {
        let compiler = DefaultQueryCompiler::new();

        let query = create_table("users")
            .if_not_exists()
            .add_column("id", "serial", |col| col.primary_key())
            .add_column("email", "varchar(255)", |col| col.not_null().unique())
            .add_column("role", "varchar(50)", |col| col.default_to("user"))
            .add_column("org_id", "int8", |col| {
                col.references("organizations.id")
                    .on_delete(OnModifyForeignAction::Cascade)
                    .on_update(OnModifyForeignAction::Restrict)
            })
            .compile(&compiler);

        assert_eq!(
            query.sql,
            "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" serial PRIMARY KEY, \"email\" varchar(255) NOT NULL UNIQUE, \"role\" varchar(50) DEFAULT ?, \"org_id\" int8 REFERENCES \"organizations\" (\"id\") ON DELETE CASCADE ON UPDATE RESTRICT)"
        );
        assert_eq!(query.parameters, vec![crate::operation_node::Value::from("user")]);
    }

    #[test]
    fn test_table_constraints() {
        use crate::raw_builder::sql;

        let compiler = DefaultQueryCompiler::new();

        let query = create_table("order_items")
            .add_column("order_id", "int8", |c| c.not_null())
            .add_column("item_id", "int8", |c| c.not_null())
            .add_column("quantity", "int4", |c| c.not_null())
            .add_primary_key_constraint("pk_order_items", ["order_id", "item_id"])
            .add_foreign_key_constraint(
                "fk_order_items_order",
                ["order_id"],
                "orders",
                ["id"],
                |fk| fk.on_delete(OnModifyForeignAction::Cascade),
            )
            .add_check_constraint("chk_qty_positive", sql("quantity > 0"))
            .compile(&compiler);

        assert_eq!(
            query.sql,
            "CREATE TABLE \"order_items\" (\"order_id\" int8 NOT NULL, \"item_id\" int8 NOT NULL, \"quantity\" int4 NOT NULL, CONSTRAINT \"pk_order_items\" PRIMARY KEY (\"order_id\", \"item_id\"), CONSTRAINT \"fk_order_items_order\" FOREIGN KEY (\"order_id\") REFERENCES \"orders\" (\"id\") ON DELETE CASCADE, CONSTRAINT \"chk_qty_positive\" CHECK (quantity > 0))"
        );
    }
}
