//! Core plugin trait definition for AST query rewriting and result interception.

use async_trait::async_trait;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::operation_node::OperationNode;
use crate::query_executor::QueryResult;
use crate::util::QueryId;

/// Plugin trait for intercepting queries before compilation and results after execution.
// @spec REQ-PLUG-001
#[async_trait]
pub trait KyselyPlugin: Send + Sync {
    /// Intercepts and transforms the AST of a query before compilation.
    fn transform_query(&self, node: Arc<OperationNode>, _query_id: QueryId) -> Arc<OperationNode> {
        node
    }

    /// Intercepts and transforms the execution result before returning to the caller.
    async fn transform_result(
        &self,
        result: QueryResult,
        _query_id: QueryId,
    ) -> Result<QueryResult, DriverError> {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::kysely::Kysely;
    use crate::operation_node::{LimitNode, OperationNode, Value};
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use crate::query_executor::Row;
    use std::sync::Mutex;

    struct PipelineRecordingConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for PipelineRecordingConnection {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            let row = Row::new(["count"], [Value::I64(42)]);
            Ok(QueryResult::with_rows(vec![row]))
        }
    }

    struct PipelineRecordingDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for PipelineRecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(PipelineRecordingConnection {
                queries: Arc::clone(&self.queries),
            }))
        }

        async fn release_connection(
            &self,
            _connection: Box<dyn DatabaseConnection>,
        ) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    struct LimitAndResultMultiplierPlugin;

    #[async_trait]
    impl KyselyPlugin for LimitAndResultMultiplierPlugin {
        fn transform_query(
            &self,
            node: Arc<OperationNode>,
            _query_id: QueryId,
        ) -> Arc<OperationNode> {
            if let OperationNode::SelectQuery(ref select) = node.as_ref() {
                let mut modified = select.clone();
                modified.limit = Some(Arc::new(OperationNode::Limit(LimitNode::new(Arc::new(
                    OperationNode::Value(crate::operation_node::ValueNode::new(10i64)),
                )))));
                Arc::new(OperationNode::SelectQuery(modified))
            } else {
                node
            }
        }

        async fn transform_result(
            &self,
            mut result: QueryResult,
            _query_id: QueryId,
        ) -> Result<QueryResult, DriverError> {
            for row in &mut result.rows {
                if let Ok(v) = row.get::<i64>("count") {
                    let new_row =
                        Row::new(["count", "multiplied"], [Value::I64(v), Value::I64(v * 2)]);
                    *row = new_row;
                }
            }
            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_plugin_pipeline() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(PipelineRecordingDriver {
            queries: Arc::clone(&queries),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);

        let db = Kysely::new(dialect).with_plugin(LimitAndResultMultiplierPlugin);

        let result = db
            .select_from("users")
            .select(["id"])
            .execute()
            .await
            .unwrap();

        // 1. Verify AST transform injected LIMIT 10
        let recorded = queries.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert!(
            recorded[0].contains("LIMIT ?"),
            "SQL should have limit clause: {}",
            recorded[0]
        );

        // 2. Verify Result transform modified row
        assert_eq!(result.rows.len(), 1);
        let multiplied: i64 = result.rows[0].get("multiplied").unwrap();
        assert_eq!(multiplied, 84);
    }
}
