//! Atomic transaction management, isolation levels, and savepoints.

use std::sync::Arc;
use compact_str::CompactString;

use crate::driver::{Driver, DriverError};
use crate::parser::IntoTableSource;
use crate::query_builder::{
    delete_from, insert_into, select_from, update_table, DeleteQueryBuilder, InsertQueryBuilder,
    SelectQueryBuilder, UpdateQueryBuilder,
};
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult, SingleConnectionExecutor};
use crate::schema::SchemaModule;
use crate::util::QueryId;

/// SQL transaction isolation level.
// @spec REQ-EXEC-009
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted changes
    ReadUncommitted,
    /// Read committed changes (default in PostgreSQL)
    ReadCommitted,
    /// Repeatable reads within the transaction
    RepeatableRead,
    /// Full serializability
    Serializable,
}

impl IsolationLevel {
    /// Returns the SQL clause representation of this isolation level.
    pub fn as_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Builder configuring and executing database transactions.
// @spec REQ-EXEC-009
pub struct TransactionBuilder {
    driver: Arc<dyn Driver>,
    compiler: Arc<dyn QueryCompiler>,
    isolation_level: Option<IsolationLevel>,
}

impl TransactionBuilder {
    /// Creates a new `TransactionBuilder`.
    pub fn new(driver: Arc<dyn Driver>, compiler: Arc<dyn QueryCompiler>) -> Self {
        Self {
            driver,
            compiler,
            isolation_level: None,
        }
    }

    /// Sets the transaction isolation level.
    pub fn isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = Some(level);
        self
    }

    /// Executes a transaction block with automatic commit on success and rollback on error.
    // @spec REQ-EXEC-009
    pub async fn execute<T, F, Fut>(&self, callback: F) -> Result<T, DriverError>
    where
        F: FnOnce(Transaction) -> Fut,
        Fut: std::future::Future<Output = Result<T, DriverError>>,
    {
        let mut conn = self.driver.acquire_connection().await?;

        let begin_sql = if let Some(level) = self.isolation_level {
            format!("BEGIN TRANSACTION ISOLATION LEVEL {}", level.as_sql())
        } else {
            "BEGIN".to_string()
        };

        let begin_query = CompiledQuery::new(begin_sql, vec![], QueryId::generate());
        if let Err(e) = conn.execute_query(&begin_query).await {
            let _ = self.driver.release_connection(conn).await;
            return Err(e);
        }

        let conn_arc = Arc::new(tokio::sync::Mutex::new(Some(conn)));
        let executor = Arc::new(SingleConnectionExecutor::new(
            Arc::clone(&conn_arc),
            Arc::clone(&self.compiler),
        ));

        let trx = Transaction { executor };

        let result = callback(trx).await;

        let mut lock = conn_arc.lock().await;
        if let Some(mut conn) = lock.take() {
            match result {
                Ok(val) => {
                    let commit_query = CompiledQuery::new("COMMIT", vec![], QueryId::generate());
                    let commit_res = conn.execute_query(&commit_query).await;
                    let rel_res = self.driver.release_connection(conn).await;
                    commit_res?;
                    rel_res?;
                    Ok(val)
                }
                Err(err) => {
                    let rollback_query = CompiledQuery::new("ROLLBACK", vec![], QueryId::generate());
                    let _ = conn.execute_query(&rollback_query).await;
                    let _ = self.driver.release_connection(conn).await;
                    Err(err)
                }
            }
        } else {
            Err(DriverError::ExecutionFailed(
                "Transaction connection was lost or closed prematurely".into(),
            ))
        }
    }
}

/// Transaction client holding a dedicated database connection.
// @spec REQ-EXEC-009, REQ-EXEC-010
#[derive(Clone)]
pub struct Transaction {
    executor: Arc<SingleConnectionExecutor>,
}

impl Transaction {
    /// Returns the underlying query executor for this transaction.
    pub fn executor(&self) -> Arc<dyn QueryExecutor> {
        Arc::clone(&self.executor) as Arc<dyn QueryExecutor>
    }

    /// Returns the schema module for executing DDL operations inside this transaction.
    // @spec REQ-DDL-001
    pub fn schema(&self) -> SchemaModule {
        SchemaModule::new(Arc::clone(&self.executor) as Arc<dyn QueryExecutor>)
    }

    /// Starts building a SELECT query with a FROM table source inside this transaction.
    pub fn select_from(&self, table: impl IntoTableSource) -> SelectQueryBuilder {
        select_from(table).with_executor(Arc::clone(&self.executor) as Arc<dyn crate::query_executor::QueryExecutor>)
    }

    /// Starts building a SELECT query selecting all columns (`*`) inside this transaction.
    pub fn select_all_from(&self, table: impl Into<CompactString>) -> SelectQueryBuilder {
        let tbl = table.into();
        select_from(tbl.as_str())
            .select_all()
            .with_executor(Arc::clone(&self.executor) as Arc<dyn crate::query_executor::QueryExecutor>)
    }

    /// Starts building an INSERT statement inside this transaction.
    pub fn insert_into(&self, table: impl IntoTableSource) -> InsertQueryBuilder {
        insert_into(table).with_executor(Arc::clone(&self.executor) as Arc<dyn crate::query_executor::QueryExecutor>)
    }

    /// Starts building an UPDATE statement inside this transaction.
    pub fn update_table(&self, table: impl IntoTableSource) -> UpdateQueryBuilder {
        update_table(table).with_executor(Arc::clone(&self.executor) as Arc<dyn crate::query_executor::QueryExecutor>)
    }

    /// Starts building a DELETE statement inside this transaction.
    pub fn delete_from(&self, table: impl IntoTableSource) -> DeleteQueryBuilder {
        delete_from(table).with_executor(Arc::clone(&self.executor) as Arc<dyn crate::query_executor::QueryExecutor>)
    }

    /// Executes an already compiled query directly inside this transaction.
    pub async fn execute_query(&self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
        self.executor.execute_query(query).await
    }

    /// Creates a named savepoint.
    // @spec REQ-EXEC-010
    pub async fn savepoint(&self, name: &str) -> Result<(), DriverError> {
        let escaped = name.replace('"', "\"\"");
        let sql = format!("SAVEPOINT \"{}\"", escaped);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        self.executor.execute_query(&query).await.map(|_| ())
    }

    /// Rolls back to a previously created named savepoint.
    // @spec REQ-EXEC-010
    pub async fn rollback_to_savepoint(&self, name: &str) -> Result<(), DriverError> {
        let escaped = name.replace('"', "\"\"");
        let sql = format!("ROLLBACK TO SAVEPOINT \"{}\"", escaped);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        self.executor.execute_query(&query).await.map(|_| ())
    }

    /// Releases a previously created named savepoint.
    // @spec REQ-EXEC-010
    pub async fn release_savepoint(&self, name: &str) -> Result<(), DriverError> {
        let escaped = name.replace('"', "\"\"");
        let sql = format!("RELEASE SAVEPOINT \"{}\"", escaped);
        let query = CompiledQuery::new(sql, vec![], QueryId::generate());
        self.executor.execute_query(&query).await.map(|_| ())
    }

    /// Executes a closure inside a named savepoint, releasing on `Ok` and rolling back on `Err`.
    // @spec REQ-EXEC-010
    pub async fn savepoint_block<T, F, Fut>(&self, name: &str, callback: F) -> Result<T, DriverError>
    where
        F: FnOnce(Transaction) -> Fut,
        Fut: std::future::Future<Output = Result<T, DriverError>>,
    {
        self.savepoint(name).await?;
        match callback(self.clone()).await {
            Ok(val) => {
                self.release_savepoint(name).await?;
                Ok(val)
            }
            Err(err) => {
                let _ = self.rollback_to_savepoint(name).await;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use crate::dialect::CustomDialect;
    use crate::driver::DatabaseConnection;
    use crate::kysely::Kysely;
    use crate::query_compiler::DefaultQueryCompiler;

    struct RecordingConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for RecordingConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct RecordingDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for RecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(RecordingConnection {
                queries: Arc::clone(&self.queries),
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
    async fn test_transaction_and_savepoints() {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(RecordingDriver {
            queries: Arc::clone(&recorded),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        // 1. Successful transaction
        let res = db
            .transaction()
            .execute(|trx| async move {
                trx.delete_from("users").where_("id", "=", 1).execute().await?;
                Ok(42)
            })
            .await;
        assert_eq!(res.unwrap(), 42);

        {
            let q = recorded.lock().unwrap();
            assert_eq!(q[0], "BEGIN");
            assert!(q[1].starts_with("DELETE FROM"));
            assert_eq!(q[2], "COMMIT");
        }

        // 2. Failed transaction (automatic rollback)
        recorded.lock().unwrap().clear();
        let fail_res: Result<(), DriverError> = db
            .transaction()
            .execute(|trx| async move {
                trx.insert_into("users").set("name", "Bob").execute().await?;
                Err(DriverError::ExecutionFailed("intentional failure".into()))
            })
            .await;
        assert!(fail_res.is_err());

        {
            let q = recorded.lock().unwrap();
            assert_eq!(q[0], "BEGIN");
            assert!(q[1].starts_with("INSERT INTO"));
            assert_eq!(q[2], "ROLLBACK");
        }

        // 3. Isolation level configuration
        recorded.lock().unwrap().clear();
        let _ = db
            .transaction()
            .isolation_level(IsolationLevel::Serializable)
            .execute(|_trx| async move { Ok(()) })
            .await;

        {
            let q = recorded.lock().unwrap();
            assert_eq!(q[0], "BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE");
            assert_eq!(q[1], "COMMIT");
        }

        // 4. Savepoints and savepoint blocks
        recorded.lock().unwrap().clear();
        let _ = db
            .transaction()
            .execute(|trx| async move {
                trx.savepoint("sp1").await?;
                trx.rollback_to_savepoint("sp1").await?;
                trx.release_savepoint("sp1").await?;

                // Successful savepoint block
                trx.savepoint_block("sp_block_ok", |_sub| async move { Ok(()) }).await?;

                // Failed savepoint block
                let _ = trx
                    .savepoint_block("sp_block_err", |_sub| async move {
                        Err::<(), _>(DriverError::ExecutionFailed("err".into()))
                    })
                    .await;

                Ok(())
            })
            .await;

        {
            let q = recorded.lock().unwrap();
            assert_eq!(q[0], "BEGIN");
            assert_eq!(q[1], "SAVEPOINT \"sp1\"");
            assert_eq!(q[2], "ROLLBACK TO SAVEPOINT \"sp1\"");
            assert_eq!(q[3], "RELEASE SAVEPOINT \"sp1\"");
            assert_eq!(q[4], "SAVEPOINT \"sp_block_ok\"");
            assert_eq!(q[5], "RELEASE SAVEPOINT \"sp_block_ok\"");
            assert_eq!(q[6], "SAVEPOINT \"sp_block_err\"");
            assert_eq!(q[7], "ROLLBACK TO SAVEPOINT \"sp_block_err\"");
            assert_eq!(q[8], "COMMIT");
        }
    }
}
