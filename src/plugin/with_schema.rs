//! Plugin that applies a default database schema qualification to unqualified tables.

use compact_str::CompactString;
use std::sync::Arc;

use crate::operation_node::transformer::OperationNodeTransformer;
use crate::operation_node::*;
use crate::plugin::traits::KyselyPlugin;
use crate::util::QueryId;

/// AST transformer that attaches a schema qualification to unqualified `TableNode`s.
struct WithSchemaTransformer {
    schema: CompactString,
}

impl OperationNodeTransformer for WithSchemaTransformer {
    fn transform_table(
        &mut self,
        node: &TableNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        if node.schema.is_none() {
            Arc::new(OperationNode::Table(TableNode {
                schema: Some(Arc::new(OperationNode::Identifier(IdentifierNode::new(
                    self.schema.clone(),
                )))),
                table: self.transform_node(Arc::clone(&node.table)),
            }))
        } else {
            Arc::clone(orig)
        }
    }
}

/// Plugin qualifying all unqualified table references with a default database schema.
// @spec REQ-PLUG-005
#[derive(Debug, Clone)]
pub struct WithSchemaPlugin {
    schema: CompactString,
}

impl WithSchemaPlugin {
    /// Creates a new `WithSchemaPlugin` qualifying tables with the given schema name.
    pub fn new(schema: impl Into<CompactString>) -> Self {
        Self {
            schema: schema.into(),
        }
    }

    /// Returns the schema name configured on this plugin.
    pub fn schema(&self) -> &str {
        &self.schema
    }
}

impl KyselyPlugin for WithSchemaPlugin {
    fn transform_query(&self, node: Arc<OperationNode>, _query_id: QueryId) -> Arc<OperationNode> {
        let mut transformer = WithSchemaTransformer {
            schema: self.schema.clone(),
        };
        transformer.transform_node(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver, DriverError};
    use crate::kysely::Kysely;
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use crate::query_executor::QueryResult;
    use std::sync::Mutex;

    struct SchemaRecordingConn {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl DatabaseConnection for SchemaRecordingConn {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct SchemaRecordingDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl Driver for SchemaRecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(SchemaRecordingConn {
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

    #[tokio::test]
    async fn test_with_schema_plugin() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(SchemaRecordingDriver {
            queries: Arc::clone(&queries),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);

        let db = Kysely::new(dialect).with_plugin(WithSchemaPlugin::new("custom_schema"));

        // 1. SELECT query
        let _ = db
            .select_from("users")
            .select(["id", "name"])
            .inner_join("posts", |join| join.on("posts.user_id", "=", "users.id"))
            .execute()
            .await;

        // 2. INSERT query
        let _ = db
            .insert_into("users")
            .values([("name", Value::Text("Alice".into()))])
            .execute()
            .await;

        // 3. UPDATE query
        let _ = db
            .update_table("users")
            .set("name", Value::Text("Bob".into()))
            .execute()
            .await;

        // 4. DELETE query
        let _ = db
            .delete_from("users")
            .where_("id", "=", Value::I64(1))
            .execute()
            .await;

        let recorded = queries.lock().unwrap();
        assert_eq!(recorded.len(), 4);

        // Verify all 4 statements qualified "users" and "posts" with "custom_schema"
        assert!(
            recorded[0].contains("\"custom_schema\".\"users\""),
            "SELECT from table: {}",
            recorded[0]
        );
        assert!(
            recorded[0].contains("\"custom_schema\".\"posts\""),
            "SELECT join table: {}",
            recorded[0]
        );
        assert!(
            recorded[1].contains("\"custom_schema\".\"users\""),
            "INSERT table: {}",
            recorded[1]
        );
        assert!(
            recorded[2].contains("\"custom_schema\".\"users\""),
            "UPDATE table: {}",
            recorded[2]
        );
        assert!(
            recorded[3].contains("\"custom_schema\".\"users\""),
            "DELETE table: {}",
            recorded[3]
        );
    }
}
