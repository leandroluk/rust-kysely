//! Fluent builders for CREATE INDEX and DROP INDEX statements.

use compact_str::CompactString;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::expression::Expression;
use crate::operation_node::{CreateIndexNode, DropIndexNode, OperationNode, TableNode};
use crate::parser::IntoTableSource;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};

/// Fluent builder for CREATE INDEX statements.
// @spec REQ-DDL-007
#[derive(Clone)]
pub struct CreateIndexBuilder {
    pub(crate) node: CreateIndexNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for CreateIndexBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateIndexBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for CreateIndexBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl CreateIndexBuilder {
    /// Creates a new `CreateIndexBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: CreateIndexNode::new(name, Arc::new(OperationNode::Table(TableNode::new("")))),
            executor: None,
        }
    }

    /// Specifies the target table for the index.
    // @spec REQ-DDL-007
    pub fn on(mut self, table: impl IntoTableSource) -> Self {
        self.node.table = table.into_table_source();
        self
    }

    /// Appends a column to the index definition.
    // @spec REQ-DDL-007
    pub fn column(mut self, col: impl Into<CompactString>) -> Self {
        self.node.columns.push(col.into());
        self
    }

    /// Appends multiple columns to the index definition.
    // @spec REQ-DDL-007
    pub fn columns(mut self, cols: impl IntoIterator<Item = impl Into<CompactString>>) -> Self {
        self.node.columns.extend(cols.into_iter().map(Into::into));
        self
    }

    /// Marks the index as `UNIQUE`.
    // @spec REQ-DDL-007
    pub fn unique(mut self) -> Self {
        self.node.unique = true;
        self
    }

    /// Specifies the index access method (`USING btree`, `USING gin`, `USING gist`, etc.).
    // @spec REQ-DDL-007
    pub fn using(mut self, method: impl Into<CompactString>) -> Self {
        self.node.using_method = Some(method.into());
        self
    }

    /// Specifies a partial index `WHERE` filter.
    // @spec REQ-DDL-007
    pub fn where_(mut self, expr: impl Expression) -> Self {
        self.node.where_clause = Some(expr.to_operation_node());
        self
    }

    /// Adds the `IF NOT EXISTS` clause.
    // @spec REQ-DDL-007
    pub fn if_not_exists(mut self) -> Self {
        self.node.if_not_exists = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this CREATE INDEX statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::CreateIndex(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the CREATE INDEX statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on CreateIndexBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Fluent builder for DROP INDEX statements.
// @spec REQ-DDL-007
#[derive(Clone)]
pub struct DropIndexBuilder {
    pub(crate) node: DropIndexNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for DropIndexBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropIndexBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for DropIndexBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl DropIndexBuilder {
    /// Creates a new `DropIndexBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: DropIndexNode::new(name),
            executor: None,
        }
    }

    /// Adds the `IF EXISTS` clause.
    // @spec REQ-DDL-007
    pub fn if_exists(mut self) -> Self {
        self.node.if_exists = true;
        self
    }

    /// Adds the `CASCADE` option.
    // @spec REQ-DDL-007
    pub fn cascade(mut self) -> Self {
        self.node.cascade = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this DROP INDEX statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::DropIndex(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the DROP INDEX statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on DropIndexBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start creating an index.
pub fn create_index(name: impl Into<CompactString>) -> CreateIndexBuilder {
    CreateIndexBuilder::new(name)
}

/// Helper function to start dropping an index.
pub fn drop_index(name: impl Into<CompactString>) -> DropIndexBuilder {
    DropIndexBuilder::new(name)
}
