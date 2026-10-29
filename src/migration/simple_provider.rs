//! In-memory migration provider for unit testing and embedded applications.

use std::collections::BTreeMap;
use std::sync::Arc;
use async_trait::async_trait;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::migration::traits::{Migration, MigrationProvider};

/// In-memory migration provider storing registered migrations in a sorted BTreeMap.
// @spec REQ-MIG-002, REQ-MIG-004
#[derive(Default, Clone)]
pub struct SimpleMigrationProvider {
    migrations: BTreeMap<CompactString, Arc<dyn Migration>>,
}

impl SimpleMigrationProvider {
    /// Creates a new empty `SimpleMigrationProvider`.
    pub fn new() -> Self {
        Self {
            migrations: BTreeMap::new(),
        }
    }

    /// Builder method registering a migration by name.
    pub fn register(mut self, name: impl Into<CompactString>, migration: impl Migration + 'static) -> Self {
        self.migrations.insert(name.into(), Arc::new(migration));
        self
    }

    /// Mutating method registering a migration by name.
    pub fn add(&mut self, name: impl Into<CompactString>, migration: impl Migration + 'static) {
        self.migrations.insert(name.into(), Arc::new(migration));
    }

    /// Mutating method registering an `Arc<dyn Migration>` directly.
    pub fn add_arc(&mut self, name: impl Into<CompactString>, migration: Arc<dyn Migration>) {
        self.migrations.insert(name.into(), migration);
    }
}

#[async_trait]
impl MigrationProvider for SimpleMigrationProvider {
    async fn get_migrations(&self) -> Result<BTreeMap<CompactString, Arc<dyn Migration>>, DriverError> {
        Ok(self.migrations.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::dialect::CustomDialect;
    use crate::driver::{DatabaseConnection, Driver};
    use crate::kysely::Kysely;
    use crate::migration::traits::SqlMigration;
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use crate::query_executor::QueryResult;

    struct DummyConnection {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DatabaseConnection for DummyConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());
            Ok(QueryResult::default())
        }
    }

    struct DummyDriver {
        queries: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Driver for DummyDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(DummyConnection {
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
    async fn test_simple_migration_provider() {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let driver = Arc::new(DummyDriver {
            queries: Arc::clone(&recorded),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        let provider = SimpleMigrationProvider::new()
            .register(
                "002_add_index",
                SqlMigration::with_down(
                    "CREATE INDEX idx_user ON users(email)",
                    "DROP INDEX idx_user",
                ),
            )
            .register(
                "001_create_users",
                SqlMigration::with_down(
                    "CREATE TABLE users (id SERIAL PRIMARY KEY, email TEXT NOT NULL)",
                    "DROP TABLE users",
                ),
            );

        let migrations = provider.get_migrations().await.unwrap();
        assert_eq!(migrations.len(), 2);

        // Verify BTreeMap orders migrations lexicographically
        let names: Vec<CompactString> = migrations.keys().cloned().collect();
        assert_eq!(names, vec![
            CompactString::from("001_create_users"),
            CompactString::from("002_add_index")
        ]);

        // Execute 001 up in a transaction
        db.transaction()
            .execute(|trx| async move {
                migrations.get("001_create_users").unwrap().up(&trx).await?;
                migrations.get("001_create_users").unwrap().down(&trx).await?;
                Ok(())
            })
            .await
            .unwrap();

        let queries = recorded.lock().unwrap();
        assert_eq!(queries.len(), 4);
        assert_eq!(queries[0], "BEGIN");
        assert_eq!(queries[1], "CREATE TABLE users (id SERIAL PRIMARY KEY, email TEXT NOT NULL)");
        assert_eq!(queries[2], "DROP TABLE users");
        assert_eq!(queries[3], "COMMIT");
    }
}
