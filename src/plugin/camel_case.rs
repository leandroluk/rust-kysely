//! Plugin mapping between camelCase application identifiers and snake_case database columns.

use async_trait::async_trait;
use compact_str::CompactString;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::operation_node::transformer::OperationNodeTransformer;
use crate::operation_node::*;
use crate::plugin::traits::KyselyPlugin;
use crate::query_executor::{QueryResult, Row};
use crate::util::QueryId;

/// Converts a camelCase or PascalCase string to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_is_lower_or_digit = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if prev_is_lower_or_digit {
                result.push('_');
            } else if let Some(&next) = chars.peek() {
                if next.is_lowercase() && !result.is_empty() && !result.ends_with('_') {
                    result.push('_');
                }
            }
            result.extend(c.to_lowercase());
            prev_is_lower_or_digit = false;
        } else if c == '-' || c == '_' {
            result.push('_');
            prev_is_lower_or_digit = false;
        } else {
            result.push(c);
            prev_is_lower_or_digit = c.is_lowercase() || c.is_numeric();
        }
    }
    result
}

/// Converts a snake_case string to camelCase.
pub fn to_camel_case(s: &str, preserve_leading_underscore: bool) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    let mut is_first = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
            if is_first && preserve_leading_underscore {
                result.push(c);
            } else if !is_first {
                capitalize_next = true;
            }
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
            is_first = false;
        } else {
            result.push(c);
            is_first = false;
        }
    }
    result
}

/// Options configuring `CamelCasePlugin` behavior.
// @spec REQ-PLUG-003
#[derive(Debug, Clone)]
pub struct CamelCaseOptions {
    /// Preserves leading underscores in column names (e.g. `_id` -> `_id`). Defaults to `true`.
    pub preserve_leading_underscore: bool,
    /// Converts table names between camelCase and snake_case in addition to columns. Defaults to `false`.
    pub convert_tables: bool,
}

impl Default for CamelCaseOptions {
    fn default() -> Self {
        Self {
            preserve_leading_underscore: true,
            convert_tables: false,
        }
    }
}

/// AST transformer rewriting identifiers between camelCase and snake_case.
struct CamelCaseTransformer {
    options: CamelCaseOptions,
}

impl OperationNodeTransformer for CamelCaseTransformer {
    fn transform_column(
        &mut self,
        node: &ColumnNode,
        _orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let transformed_col = self.transform_node(Arc::clone(&node.column));
        if Arc::ptr_eq(&transformed_col, &node.column) {
            Arc::clone(_orig)
        } else {
            Arc::new(OperationNode::Column(ColumnNode {
                column: transformed_col,
            }))
        }
    }

    fn transform_table(
        &mut self,
        node: &TableNode,
        _orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        if self.options.convert_tables {
            let transformed_table = self.transform_node(Arc::clone(&node.table));
            let transformed_schema = node
                .schema
                .as_ref()
                .map(|s| self.transform_node(Arc::clone(s)));
            Arc::new(OperationNode::Table(TableNode {
                schema: transformed_schema,
                table: transformed_table,
            }))
        } else {
            Arc::clone(_orig)
        }
    }

    fn transform_reference(
        &mut self,
        node: &ReferenceNode,
        _orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let transformed_table = if self.options.convert_tables {
            node.table
                .as_ref()
                .map(|t| self.transform_node(Arc::clone(t)))
        } else {
            node.table.clone()
        };
        let transformed_col = self.transform_node(Arc::clone(&node.column));
        Arc::new(OperationNode::Reference(ReferenceNode {
            table: transformed_table,
            column: transformed_col,
        }))
    }

    fn transform_column_update(
        &mut self,
        node: &ColumnUpdateNode,
        _orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let transformed_col = self.transform_node(Arc::clone(&node.column));
        let transformed_val = self.transform_node(Arc::clone(&node.value));
        Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode::new(
            transformed_col,
            transformed_val,
        )))
    }

    fn transform_identifier(
        &mut self,
        node: &IdentifierNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let snake = to_snake_case(&node.name);
        if snake == node.name.as_str() {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Identifier(IdentifierNode::new(snake)))
        }
    }
}

/// Plugin converting camelCase identifiers in queries to snake_case, and snake_case row columns to camelCase.
// @spec REQ-PLUG-002, REQ-PLUG-003
#[derive(Debug, Clone, Default)]
pub struct CamelCasePlugin {
    options: CamelCaseOptions,
}

impl CamelCasePlugin {
    /// Creates a new `CamelCasePlugin` with default configuration.
    pub fn new() -> Self {
        Self {
            options: CamelCaseOptions::default(),
        }
    }

    /// Creates a `CamelCasePlugin` with custom options.
    pub fn with_options(options: CamelCaseOptions) -> Self {
        Self { options }
    }
}

#[async_trait]
impl KyselyPlugin for CamelCasePlugin {
    fn transform_query(&self, node: Arc<OperationNode>, _query_id: QueryId) -> Arc<OperationNode> {
        let mut transformer = CamelCaseTransformer {
            options: self.options.clone(),
        };
        transformer.transform_node(node)
    }

    async fn transform_result(
        &self,
        mut result: QueryResult,
        _query_id: QueryId,
    ) -> Result<QueryResult, DriverError> {
        for row in &mut result.rows {
            let new_columns: Vec<CompactString> = row
                .columns()
                .iter()
                .map(|col| {
                    CompactString::from(to_camel_case(
                        col.as_str(),
                        self.options.preserve_leading_underscore,
                    ))
                })
                .collect();

            let values = row.values().to_vec();
            *row = Row::new(new_columns, values);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::kysely::Kysely;
    use crate::operation_node::Value;
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use std::sync::Mutex;

    struct CamelCaseRecordingConn {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for CamelCaseRecordingConn {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());

            // Simulate database returning snake_case column names
            let row = Row::new(
                ["user_id", "first_name", "created_date_at"],
                [
                    Value::I64(100),
                    Value::Text("Alice".into()),
                    Value::Text("2026-10-29".into()),
                ],
            );
            Ok(QueryResult::with_rows(vec![row]))
        }
    }

    struct CamelCaseRecordingDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for CamelCaseRecordingDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(CamelCaseRecordingConn {
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
    async fn test_camel_case_plugin() {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(CamelCaseRecordingDriver {
            queries: Arc::clone(&queries),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);

        let db = Kysely::new(dialect).with_plugin(CamelCasePlugin::new());

        // Select query using camelCase columns
        let result = db
            .select_from("users")
            .select(["userId", "firstName", "createdDateAt"])
            .where_("firstName", "=", "Alice")
            .execute()
            .await
            .unwrap();

        // 1. Verify AST compiled to snake_case
        let recorded = queries.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        let sql = &recorded[0];
        assert!(
            sql.contains("\"user_id\""),
            "Should contain snake_case user_id: {sql}"
        );
        assert!(
            sql.contains("\"first_name\""),
            "Should contain snake_case first_name: {sql}"
        );
        assert!(
            sql.contains("\"created_date_at\""),
            "Should contain snake_case created_date_at: {sql}"
        );

        // 2. Verify QueryResult row keys mapped back to camelCase
        assert_eq!(result.rows.len(), 1);
        let row = &result.rows[0];
        let id: i64 = row.get("userId").unwrap();
        let name: String = row.get("firstName").unwrap();
        let created: String = row.get("createdDateAt").unwrap();

        assert_eq!(id, 100);
        assert_eq!(name, "Alice");
        assert_eq!(created, "2026-10-29");
    }
}
