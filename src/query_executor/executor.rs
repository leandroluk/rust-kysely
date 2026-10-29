//! Query execution pipeline coordinating compilation, connection management, and row extraction.

use std::sync::Arc;
use async_trait::async_trait;

use crate::driver::{DatabaseConnection, Driver, DriverError};
use crate::operation_node::OperationNode;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::result::QueryResult;
use crate::query_executor::row::{FromRowValue, Row};

/// Async trait coordinating database query compilation and execution.
// @spec REQ-EXEC-004, REQ-EXEC-007, REQ-EXEC-008
#[async_trait]
pub trait QueryExecutor: Send + Sync {
    /// Compiles an AST node and executes it against the database.
    async fn execute_node(&self, node: &OperationNode) -> Result<QueryResult, DriverError>;

    /// Executes an already compiled SQL query against the database.
    async fn execute_query(&self, query: &CompiledQuery) -> Result<QueryResult, DriverError>;

    /// Compiles an AST node, executes it, and returns the first row if available.
    async fn execute_node_take_first(&self, node: &OperationNode) -> Result<Option<Row>, DriverError> {
        let res = self.execute_node(node).await?;
        Ok(res.rows.into_iter().next())
    }

    /// Compiles an AST node, executes it, and returns the first row or errors if empty.
    async fn execute_node_take_first_or_throw(&self, node: &OperationNode) -> Result<Row, DriverError> {
        self.execute_node_take_first(node).await?.ok_or_else(|| {
            DriverError::ExecutionFailed("Expected at least one row, but query returned none".into())
        })
    }

    /// Executes a compiled query and returns the first row if available.
    async fn execute_take_first(&self, query: &CompiledQuery) -> Result<Option<Row>, DriverError> {
        let res = self.execute_query(query).await?;
        Ok(res.rows.into_iter().next())
    }

    /// Executes a compiled query and returns the first row or errors if empty.
    async fn execute_take_first_or_throw(&self, query: &CompiledQuery) -> Result<Row, DriverError> {
        self.execute_take_first(query).await?.ok_or_else(|| {
            DriverError::ExecutionFailed("Expected at least one row, but query returned none".into())
        })
    }
}

/// Extension trait providing generic convenience execution helpers for any `QueryExecutor`.
// @spec REQ-EXEC-008
#[async_trait]
pub trait QueryExecutorExt {
    /// Executes a compiled query and returns the first scalar value of the first row.
    async fn execute_take_first_val<T: FromRowValue + Send>(&self, query: &CompiledQuery) -> Result<Option<T>, DriverError>;
}

#[async_trait]
impl<E: ?Sized + QueryExecutor> QueryExecutorExt for E {
    async fn execute_take_first_val<T: FromRowValue + Send>(&self, query: &CompiledQuery) -> Result<Option<T>, DriverError> {
        if let Some(row) = self.execute_take_first(query).await? {
            let val = row.get_by_index::<T>(0).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }
}

/// Standard query executor utilizing a pooled driver and query compiler.
// @spec REQ-EXEC-004
pub struct DefaultQueryExecutor {
    driver: Arc<dyn Driver>,
    compiler: Arc<dyn QueryCompiler>,
}

impl DefaultQueryExecutor {
    /// Creates a new `DefaultQueryExecutor` with the specified driver and compiler.
    pub fn new(driver: Arc<dyn Driver>, compiler: Arc<dyn QueryCompiler>) -> Self {
        Self { driver, compiler }
    }

    /// Returns a reference to the underlying driver.
    pub fn driver(&self) -> &Arc<dyn Driver> {
        &self.driver
    }

    /// Returns a reference to the query compiler.
    pub fn compiler(&self) -> &Arc<dyn QueryCompiler> {
        &self.compiler
    }
}

#[async_trait]
impl QueryExecutor for DefaultQueryExecutor {
    async fn execute_node(&self, node: &OperationNode) -> Result<QueryResult, DriverError> {
        let compiled = self.compiler.compile(node);
        self.execute_query(&compiled).await
    }

    async fn execute_query(&self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
        let mut conn = self.driver.acquire_connection().await?;
        let res = conn.execute_query(query).await;
        let rel = self.driver.release_connection(conn).await;
        let res = res?;
        rel?;
        Ok(res)
    }
}

/// Query executor bound to a single active database connection (used during transactions).
// @spec REQ-EXEC-009
pub struct SingleConnectionExecutor {
    connection: Arc<tokio::sync::Mutex<Option<Box<dyn DatabaseConnection>>>>,
    compiler: Arc<dyn QueryCompiler>,
}

impl SingleConnectionExecutor {
    /// Creates a new executor bound to a single connection.
    pub fn new(
        connection: Arc<tokio::sync::Mutex<Option<Box<dyn DatabaseConnection>>>>,
        compiler: Arc<dyn QueryCompiler>,
    ) -> Self {
        Self {
            connection,
            compiler,
        }
    }

    /// Returns a reference to the query compiler.
    pub fn compiler(&self) -> &Arc<dyn QueryCompiler> {
        &self.compiler
    }
}

#[async_trait]
impl QueryExecutor for SingleConnectionExecutor {
    async fn execute_node(&self, node: &OperationNode) -> Result<QueryResult, DriverError> {
        let compiled = self.compiler.compile(node);
        self.execute_query(&compiled).await
    }

    async fn execute_query(&self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
        let mut lock = self.connection.lock().await;
        let conn = lock.as_mut().ok_or_else(|| {
            DriverError::ExecutionFailed("Database connection has already been released".into())
        })?;
        conn.execute_query(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::Value;
    use crate::query_compiler::DefaultQueryCompiler;

    struct MockConnection {
        return_rows: Vec<Row>,
    }

    #[async_trait]
    impl DatabaseConnection for MockConnection {
        async fn execute_query(&mut self, _query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            Ok(QueryResult::with_rows(self.return_rows.clone()))
        }
    }

    struct MockDriver {
        return_rows: Vec<Row>,
    }

    #[async_trait]
    impl Driver for MockDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockConnection {
                return_rows: self.return_rows.clone(),
            }))
        }

        async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_query_executor() {
        use crate::util::QueryId;

        let rows = vec![
            Row::new(vec!["id", "count"], vec![Value::I64(10), Value::I64(100)]),
            Row::new(vec!["id", "count"], vec![Value::I64(20), Value::I64(200)]),
        ];

        let driver = Arc::new(MockDriver {
            return_rows: rows.clone(),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let executor = DefaultQueryExecutor::new(driver, compiler);

        let query = CompiledQuery::new("SELECT * FROM test", vec![], QueryId::generate());

        // execute_query
        let res = executor.execute_query(&query).await.unwrap();
        assert_eq!(res.len(), 2);

        // execute_take_first
        let first = executor.execute_take_first(&query).await.unwrap();
        assert!(first.is_some());
        assert_eq!(first.unwrap().get::<i64>("id").unwrap(), 10);

        // execute_take_first_or_throw
        let first_throw = executor.execute_take_first_or_throw(&query).await.unwrap();
        assert_eq!(first_throw.get::<i64>("count").unwrap(), 100);

        // execute_take_first_val
        let first_val = executor.execute_take_first_val::<i64>(&query).await.unwrap();
        assert_eq!(first_val, Some(10));

        // empty query result behavior
        let empty_driver = Arc::new(MockDriver {
            return_rows: Vec::new(),
        });
        let empty_executor = DefaultQueryExecutor::new(empty_driver, Arc::new(DefaultQueryCompiler::new()));

        let empty_first = empty_executor.execute_take_first(&query).await.unwrap();
        assert_eq!(empty_first, None);

        let empty_first_val = empty_executor.execute_take_first_val::<i64>(&query).await.unwrap();
        assert_eq!(empty_first_val, None);

        let throw_err = empty_executor.execute_take_first_or_throw(&query).await;
        assert!(matches!(throw_err, Err(DriverError::ExecutionFailed(_))));

        // SingleConnectionExecutor test
        let single_conn = Arc::new(tokio::sync::Mutex::new(Some(Box::new(MockConnection {
            return_rows: rows,
        }) as Box<dyn DatabaseConnection>)));
        let single_exec = SingleConnectionExecutor::new(single_conn, Arc::new(DefaultQueryCompiler::new()));
        let single_res = single_exec.execute_query(&query).await.unwrap();
        assert_eq!(single_res.len(), 2);
    }
}
