//! Fluent builders for CREATE VIEW and DROP VIEW statements.

use std::sync::Arc;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::expression::Expression;
use crate::operation_node::{CreateViewNode, DropViewNode, OperationNode, SelectQueryNode};
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult};

/// Fluent builder for CREATE VIEW statements.
// @spec REQ-DDL-009
#[derive(Clone)]
pub struct CreateViewBuilder {
    pub(crate) node: CreateViewNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for CreateViewBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateViewBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for CreateViewBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl CreateViewBuilder {
    /// Creates a new `CreateViewBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: CreateViewNode::new(name, Arc::new(OperationNode::SelectQuery(SelectQueryNode::new()))),
            executor: None,
        }
    }

    /// Sets the underlying SELECT query defining the view.
    // @spec REQ-DDL-009
    pub fn as_query(mut self, query: impl Expression) -> Self {
        self.node.query = query.to_operation_node();
        self
    }

    /// Adds the `OR REPLACE` clause.
    // @spec REQ-DDL-009
    pub fn or_replace(mut self) -> Self {
        self.node.or_replace = true;
        self
    }

    /// Marks the view as `MATERIALIZED`.
    // @spec REQ-DDL-009
    pub fn materialized(mut self) -> Self {
        self.node.materialized = true;
        self
    }

    /// Adds the `IF NOT EXISTS` clause.
    // @spec REQ-DDL-009
    pub fn if_not_exists(mut self) -> Self {
        self.node.if_not_exists = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this CREATE VIEW statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::CreateView(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the CREATE VIEW statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on CreateViewBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Fluent builder for DROP VIEW statements.
// @spec REQ-DDL-009
#[derive(Clone)]
pub struct DropViewBuilder {
    pub(crate) node: DropViewNode,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for DropViewBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropViewBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for DropViewBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl DropViewBuilder {
    /// Creates a new `DropViewBuilder`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            node: DropViewNode::new(name),
            executor: None,
        }
    }

    /// Marks the view to drop as `MATERIALIZED`.
    // @spec REQ-DDL-009
    pub fn materialized(mut self) -> Self {
        self.node.materialized = true;
        self
    }

    /// Adds the `IF EXISTS` clause.
    // @spec REQ-DDL-009
    pub fn if_exists(mut self) -> Self {
        self.node.if_exists = true;
        self
    }

    /// Adds the `CASCADE` option.
    // @spec REQ-DDL-009
    pub fn cascade(mut self) -> Self {
        self.node.cascade = true;
        self
    }

    /// Attaches an executor to this builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns the AST node representation of this DROP VIEW statement.
    pub fn to_operation_node(&self) -> OperationNode {
        OperationNode::DropView(self.node.clone())
    }

    /// Compiles the query using the specified compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&self.to_operation_node())
    }

    /// Executes the DROP VIEW statement against the database.
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on DropViewBuilder".into())
        })?;
        executor.execute_node(&self.to_operation_node()).await
    }
}

/// Helper function to start creating a view.
pub fn create_view(name: impl Into<CompactString>) -> CreateViewBuilder {
    CreateViewBuilder::new(name)
}

/// Helper function to start dropping a view.
pub fn drop_view(name: impl Into<CompactString>) -> DropViewBuilder {
    DropViewBuilder::new(name)
}
