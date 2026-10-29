//! SchemaModule coordinating DDL statement creation and execution.

use std::sync::Arc;
use compact_str::CompactString;

use crate::parser::IntoTableSource;
use crate::query_executor::QueryExecutor;
use crate::schema::alter_table_builder::{alter_table, AlterTableBuilder};
use crate::schema::create_table_builder::{create_table, CreateTableBuilder};
use crate::schema::drop_table_builder::{drop_table, DropTableBuilder};
use crate::schema::index_builder::{create_index, drop_index, CreateIndexBuilder, DropIndexBuilder};
use crate::schema::type_builder::{create_type, drop_type, CreateTypeBuilder, DropTypeBuilder};
use crate::schema::view_builder::{create_view, drop_view, CreateViewBuilder, DropViewBuilder};

/// Coordinator for creating and executing database DDL operations.
// @spec REQ-DDL-001
#[derive(Clone)]
pub struct SchemaModule {
    executor: Arc<dyn QueryExecutor>,
}

impl SchemaModule {
    /// Creates a new `SchemaModule` wrapping an active query executor.
    pub fn new(executor: Arc<dyn QueryExecutor>) -> Self {
        Self { executor }
    }

    /// Starts building a CREATE TABLE statement.
    // @spec REQ-DDL-002
    pub fn create_table(&self, table: impl IntoTableSource) -> CreateTableBuilder {
        create_table(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building an ALTER TABLE statement.
    // @spec REQ-DDL-005
    pub fn alter_table(&self, table: impl IntoTableSource) -> AlterTableBuilder {
        alter_table(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a DROP TABLE statement.
    // @spec REQ-DDL-006
    pub fn drop_table(&self, table: impl IntoTableSource) -> DropTableBuilder {
        drop_table(table).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a CREATE INDEX statement.
    // @spec REQ-DDL-007
    pub fn create_index(&self, name: impl Into<CompactString>) -> CreateIndexBuilder {
        create_index(name).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a DROP INDEX statement.
    // @spec REQ-DDL-007
    pub fn drop_index(&self, name: impl Into<CompactString>) -> DropIndexBuilder {
        drop_index(name).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a CREATE VIEW statement.
    // @spec REQ-DDL-009
    pub fn create_view(&self, name: impl Into<CompactString>) -> CreateViewBuilder {
        create_view(name).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a DROP VIEW statement.
    // @spec REQ-DDL-009
    pub fn drop_view(&self, name: impl Into<CompactString>) -> DropViewBuilder {
        drop_view(name).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a CREATE TYPE statement.
    // @spec REQ-DDL-008
    pub fn create_type(&self, name: impl Into<CompactString>) -> CreateTypeBuilder {
        create_type(name).with_executor(Arc::clone(&self.executor))
    }

    /// Starts building a DROP TYPE statement.
    // @spec REQ-DDL-008
    pub fn drop_type(&self, name: impl Into<CompactString>) -> DropTypeBuilder {
        drop_type(name).with_executor(Arc::clone(&self.executor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver, DriverError};
    use crate::kysely::Kysely;
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use crate::query_executor::QueryResult;

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
    async fn test_schema_module() {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(RecordingDriver {
            queries: Arc::clone(&recorded),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        // 1. db.schema().create_table(...).execute().await
        db.schema()
            .create_table("posts")
            .if_not_exists()
            .add_column("id", "serial", |c| c.primary_key())
            .add_column("title", "text", |c| c.not_null())
            .execute()
            .await
            .unwrap();

        // 2. db.schema().create_index(...).execute().await
        db.schema()
            .create_index("idx_posts_title")
            .on("posts")
            .column("title")
            .execute()
            .await
            .unwrap();

        // 3. db.schema().alter_table(...).execute().await
        db.schema()
            .alter_table("posts")
            .add_column("views", "int4", |c| c.default_to(0))
            .execute()
            .await
            .unwrap();

        // 4. db.schema().drop_table(...).execute().await
        db.schema()
            .drop_table("posts")
            .if_exists()
            .cascade()
            .execute()
            .await
            .unwrap();

        let q = recorded.lock().unwrap();
        assert_eq!(q.len(), 4);
        assert!(q[0].starts_with("CREATE TABLE IF NOT EXISTS \"posts\""));
        assert_eq!(q[1], "CREATE INDEX \"idx_posts_title\" ON \"posts\" (\"title\")");
        assert!(q[2].starts_with("ALTER TABLE \"posts\" ADD COLUMN \"views\""));
        assert_eq!(q[3], "DROP TABLE IF EXISTS \"posts\" CASCADE");
    }
}
