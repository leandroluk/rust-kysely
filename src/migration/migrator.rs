//! Schema migrator engine coordinating locks, transactions, and migration state.

use compact_str::CompactString;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::driver::DriverError;
use crate::kysely::Kysely;
use crate::migration::traits::{
    Migration, MigrationInfo, MigrationProvider, MigrationResult, MigrationStatus,
};
use crate::operation_node::{OrderDirection, Value};

/// Upstream Kysely sentinel name indicating rollback of all migrations.
// @spec REQ-MIG-008
pub const NO_MIGRATIONS: &str = "__NO_MIGRATIONS__";

/// Formats a `SystemTime` as an ISO 8601 UTC timestamp string without external dependencies.
pub fn format_iso8601(time: std::time::SystemTime) -> String {
    let dur = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();

    let days = total_secs / 86400;
    let day_secs = total_secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z")
}

/// Configuration options for the migration runner.
// @spec REQ-MIG-001, REQ-MIG-007
#[derive(Debug, Clone)]
pub struct MigratorConfig {
    /// Name of the migration tracking table. Defaults to "kysely_migration".
    pub migration_table_name: CompactString,
    /// Schema where the migration tracking table lives (optional).
    pub migration_table_schema: Option<CompactString>,
    /// Optional lock table name (if lock table is used).
    pub migration_lock_table_name: Option<CompactString>,
    /// Optional lock table schema.
    pub migration_lock_table_schema: Option<CompactString>,
}

impl Default for MigratorConfig {
    fn default() -> Self {
        Self {
            migration_table_name: CompactString::from("kysely_migration"),
            migration_table_schema: None,
            migration_lock_table_name: None,
            migration_lock_table_schema: None,
        }
    }
}

/// Result set containing all executed migration steps and optional overall error.
// @spec REQ-MIG-008
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationResultSet {
    /// List of migration execution results in chronological execution order.
    pub results: Vec<MigrationResult>,
    /// Error message if migration stopped due to failure.
    pub error: Option<String>,
}

impl MigrationResultSet {
    /// Returns true if all attempted migrations succeeded and no error occurred.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
            && self
                .results
                .iter()
                .all(|r| r.status == MigrationStatus::Success)
    }
}

/// Database schema migration runner.
// @spec REQ-MIG-001, REQ-MIG-007, REQ-MIG-008
pub struct Migrator {
    db: Arc<Kysely>,
    provider: Arc<dyn MigrationProvider>,
    config: MigratorConfig,
}

impl Migrator {
    /// Creates a new `Migrator` with default configuration.
    pub fn new(db: Kysely, provider: impl MigrationProvider + 'static) -> Self {
        Self {
            db: Arc::new(db),
            provider: Arc::new(provider),
            config: MigratorConfig::default(),
        }
    }

    /// Creates a new `Migrator` with custom configuration.
    pub fn with_config(
        db: Kysely,
        provider: impl MigrationProvider + 'static,
        config: MigratorConfig,
    ) -> Self {
        Self {
            db: Arc::new(db),
            provider: Arc::new(provider),
            config,
        }
    }

    /// Returns a reference to the active `MigratorConfig`.
    pub fn config(&self) -> &MigratorConfig {
        &self.config
    }

    /// Ensures the migration tracking table exists.
    // @spec REQ-MIG-007
    pub async fn ensure_migration_table(&self) -> Result<(), DriverError> {
        let table_name = self.config.migration_table_name.clone();
        self.db
            .schema()
            .create_table(table_name)
            .if_not_exists()
            .add_column("name", "varchar(255)", |c| c.primary_key())
            .add_column("timestamp", "varchar(255)", |c| c.not_null())
            .execute()
            .await?;
        Ok(())
    }

    /// Returns the complete list of known migrations with their execution status.
    // @spec REQ-MIG-001
    pub async fn get_migrations(&self) -> Result<Vec<MigrationInfo>, DriverError> {
        self.ensure_migration_table().await?;

        let registered = self.provider.get_migrations().await?;
        let executed = self.get_executed_migrations().await?;

        let mut list = Vec::with_capacity(registered.len());
        for name in registered.keys() {
            let executed_at = executed.get(name).cloned();
            list.push(MigrationInfo {
                name: name.clone(),
                executed_at,
            });
        }

        Ok(list)
    }

    /// Queries the migration tracking table for already executed migrations.
    async fn get_executed_migrations(
        &self,
    ) -> Result<BTreeMap<CompactString, CompactString>, DriverError> {
        let rows = self
            .db
            .select_from(self.config.migration_table_name.clone())
            .select(["name", "timestamp"])
            .order_by("name", OrderDirection::Asc)
            .execute()
            .await?;

        let mut map = BTreeMap::new();
        for row in rows.into_iter() {
            let name: CompactString = row.get("name")?;
            let timestamp: CompactString = row.get("timestamp")?;
            map.insert(name, timestamp);
        }

        Ok(map)
    }

    /// Runs all pending migrations forward in ascending order.
    // @spec REQ-MIG-008
    pub async fn migrate_to_latest(&self) -> Result<MigrationResultSet, DriverError> {
        let adapter = Arc::clone(self.db.adapter());
        let executor = Arc::clone(self.db.executor());

        adapter.acquire_migration_lock(executor.as_ref()).await?;

        let result = self.execute_migrate_to_latest().await;

        let release_res = adapter.release_migration_lock(executor.as_ref()).await;
        if let Err(err) = release_res {
            if result.is_ok() {
                return Err(err);
            }
        }

        result
    }

    async fn execute_migrate_to_latest(&self) -> Result<MigrationResultSet, DriverError> {
        self.ensure_migration_table().await?;

        let registered = self.provider.get_migrations().await?;
        let executed = self.get_executed_migrations().await?;

        let mut results = Vec::new();

        for (name, migration) in registered {
            if !executed.contains_key(&name) {
                let res = self.run_migration_step(&name, &migration, true).await;
                match res {
                    Ok(step_res) => {
                        let is_err = step_res.status == MigrationStatus::Error;
                        let err_msg = step_res.error.clone();
                        results.push(step_res);
                        if is_err {
                            return Ok(MigrationResultSet {
                                results,
                                error: err_msg,
                            });
                        }
                    }
                    Err(err) => {
                        results.push(MigrationResult {
                            migration_name: name,
                            direction: "Up".into(),
                            status: MigrationStatus::Error,
                            error: Some(err.to_string()),
                        });
                        return Ok(MigrationResultSet {
                            results,
                            error: Some(err.to_string()),
                        });
                    }
                }
            }
        }

        Ok(MigrationResultSet {
            results,
            error: None,
        })
    }

    /// Runs the next pending migration forward.
    // @spec REQ-MIG-008
    pub async fn migrate_up(&self) -> Result<MigrationResultSet, DriverError> {
        let adapter = Arc::clone(self.db.adapter());
        let executor = Arc::clone(self.db.executor());

        adapter.acquire_migration_lock(executor.as_ref()).await?;

        let result = self.execute_migrate_up().await;

        let release_res = adapter.release_migration_lock(executor.as_ref()).await;
        if let Err(err) = release_res {
            if result.is_ok() {
                return Err(err);
            }
        }

        result
    }

    async fn execute_migrate_up(&self) -> Result<MigrationResultSet, DriverError> {
        self.ensure_migration_table().await?;

        let registered = self.provider.get_migrations().await?;
        let executed = self.get_executed_migrations().await?;

        for (name, migration) in registered {
            if !executed.contains_key(&name) {
                let step_res = self.run_migration_step(&name, &migration, true).await?;
                let is_err = step_res.status == MigrationStatus::Error;
                let err_msg = step_res.error.clone();
                return Ok(MigrationResultSet {
                    results: vec![step_res],
                    error: if is_err { err_msg } else { None },
                });
            }
        }

        Ok(MigrationResultSet::default())
    }

    /// Rolls back the most recently executed migration.
    // @spec REQ-MIG-008
    pub async fn migrate_down(&self) -> Result<MigrationResultSet, DriverError> {
        let adapter = Arc::clone(self.db.adapter());
        let executor = Arc::clone(self.db.executor());

        adapter.acquire_migration_lock(executor.as_ref()).await?;

        let result = self.execute_migrate_down().await;

        let release_res = adapter.release_migration_lock(executor.as_ref()).await;
        if let Err(err) = release_res {
            if result.is_ok() {
                return Err(err);
            }
        }

        result
    }

    async fn execute_migrate_down(&self) -> Result<MigrationResultSet, DriverError> {
        self.ensure_migration_table().await?;

        let registered = self.provider.get_migrations().await?;
        let executed = self.get_executed_migrations().await?;

        // Find the last executed migration by name
        if let Some((last_name, _)) = executed.into_iter().next_back() {
            if let Some(migration) = registered.get(&last_name) {
                let step_res = self
                    .run_migration_step(&last_name, migration, false)
                    .await?;
                let is_err = step_res.status == MigrationStatus::Error;
                let err_msg = step_res.error.clone();
                return Ok(MigrationResultSet {
                    results: vec![step_res],
                    error: if is_err { err_msg } else { None },
                });
            } else {
                return Err(DriverError::ExecutionFailed(format!(
                    "Executed migration '{last_name}' not found in registered migrations provider"
                )));
            }
        }

        Ok(MigrationResultSet::default())
    }

    /// Migrates up or down to reach the specified target migration.
    // @spec REQ-MIG-008
    pub async fn migrate_to(&self, target_name: &str) -> Result<MigrationResultSet, DriverError> {
        let adapter = Arc::clone(self.db.adapter());
        let executor = Arc::clone(self.db.executor());

        adapter.acquire_migration_lock(executor.as_ref()).await?;

        let result = self.execute_migrate_to(target_name).await;

        let release_res = adapter.release_migration_lock(executor.as_ref()).await;
        if let Err(err) = release_res {
            if result.is_ok() {
                return Err(err);
            }
        }

        result
    }

    async fn execute_migrate_to(
        &self,
        target_name: &str,
    ) -> Result<MigrationResultSet, DriverError> {
        self.ensure_migration_table().await?;

        let registered = self.provider.get_migrations().await?;
        let executed = self.get_executed_migrations().await?;

        if target_name != NO_MIGRATIONS && !registered.contains_key(target_name) {
            return Err(DriverError::ExecutionFailed(format!(
                "Migration '{target_name}' not found in registered migrations provider"
            )));
        }

        let mut results = Vec::new();

        if target_name == NO_MIGRATIONS {
            // Roll back all executed migrations in reverse order
            let executed_rev: Vec<CompactString> = executed.into_keys().rev().collect();
            for name in executed_rev {
                if let Some(migration) = registered.get(&name) {
                    let step_res = self.run_migration_step(&name, migration, false).await?;
                    let is_err = step_res.status == MigrationStatus::Error;
                    let err_msg = step_res.error.clone();
                    results.push(step_res);
                    if is_err {
                        return Ok(MigrationResultSet {
                            results,
                            error: err_msg,
                        });
                    }
                }
            }
            return Ok(MigrationResultSet {
                results,
                error: None,
            });
        }

        // Check if target is currently executed
        if executed.contains_key(target_name) {
            // Target is executed. Roll back all migrations executed AFTER target in reverse order.
            let executed_rev: Vec<CompactString> = executed.into_keys().rev().collect();
            for name in executed_rev {
                if name.as_str() == target_name {
                    break;
                }
                if let Some(migration) = registered.get(&name) {
                    let step_res = self.run_migration_step(&name, migration, false).await?;
                    let is_err = step_res.status == MigrationStatus::Error;
                    let err_msg = step_res.error.clone();
                    results.push(step_res);
                    if is_err {
                        return Ok(MigrationResultSet {
                            results,
                            error: err_msg,
                        });
                    }
                }
            }
        } else {
            // Target is not executed. Run pending migrations up to and including target.
            for (name, migration) in registered {
                if !executed.contains_key(&name) {
                    let step_res = self.run_migration_step(&name, &migration, true).await?;
                    let is_err = step_res.status == MigrationStatus::Error;
                    let err_msg = step_res.error.clone();
                    let matches_target = name.as_str() == target_name;
                    results.push(step_res);
                    if is_err {
                        return Ok(MigrationResultSet {
                            results,
                            error: err_msg,
                        });
                    }
                    if matches_target {
                        break;
                    }
                }
            }
        }

        Ok(MigrationResultSet {
            results,
            error: None,
        })
    }

    /// Executes a single migration step (up or down) inside an atomic transaction.
    async fn run_migration_step(
        &self,
        name: &str,
        migration: &Arc<dyn Migration>,
        is_up: bool,
    ) -> Result<MigrationResult, DriverError> {
        let migration = Arc::clone(migration);
        let name_owned = name.to_string();
        let table_name = self.config.migration_table_name.clone();

        let direction_str = if is_up { "Up" } else { "Down" };

        let result = self
            .db
            .transaction()
            .execute(|trx| async move {
                if is_up {
                    migration.up(&trx).await?;
                    let now = format_iso8601(std::time::SystemTime::now());
                    trx.insert_into(table_name)
                        .values([
                            ("name", Value::Text(name_owned.into())),
                            ("timestamp", Value::Text(now.into())),
                        ])
                        .execute()
                        .await?;
                } else {
                    migration.down(&trx).await?;
                    trx.delete_from(table_name)
                        .where_("name", "=", Value::Text(name_owned.into()))
                        .execute()
                        .await?;
                }
                Ok(())
            })
            .await;

        match result {
            Ok(()) => Ok(MigrationResult {
                migration_name: CompactString::from(name),
                direction: CompactString::from(direction_str),
                status: MigrationStatus::Success,
                error: None,
            }),
            Err(err) => Ok(MigrationResult {
                migration_name: CompactString::from(name),
                direction: CompactString::from(direction_str),
                status: MigrationStatus::Error,
                error: Some(err.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::CustomDialect;
    use crate::driver::DatabaseConnection;
    use crate::driver::Driver;
    use crate::migration::simple_provider::SimpleMigrationProvider;
    use crate::migration::traits::SqlMigration;
    use crate::query_compiler::{CompiledQuery, DefaultQueryCompiler};
    use crate::query_executor::QueryResult;
    use std::sync::Mutex;

    struct MockMigratorConnection {
        queries: Arc<Mutex<Vec<String>>>,
        migrated: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl DatabaseConnection for MockMigratorConnection {
        async fn execute_query(
            &mut self,
            query: &CompiledQuery,
        ) -> Result<QueryResult, DriverError> {
            self.queries.lock().unwrap().push(query.sql.clone());

            // If it's a SELECT on kysely_migration, return mock rows of already migrated
            if query.sql.contains("SELECT") && query.sql.contains("kysely_migration") {
                let list = self.migrated.lock().unwrap().clone();
                let rows: Vec<crate::query_executor::Row> = list
                    .into_iter()
                    .map(|name| {
                        crate::query_executor::Row::new(
                            ["name", "timestamp"],
                            [
                                Value::Text(name.into()),
                                Value::Text("2026-10-29T12:00:00.000Z".into()),
                            ],
                        )
                    })
                    .collect();
                return Ok(QueryResult::with_rows(rows));
            }

            // If it's an INSERT into kysely_migration
            if query.sql.contains("INSERT INTO") && query.sql.contains("kysely_migration") {
                if let Some(Value::Text(ref s)) = query.parameters.first() {
                    self.migrated.lock().unwrap().push(s.to_string());
                }
            }

            // If it's a DELETE from kysely_migration
            if query.sql.contains("DELETE FROM") && query.sql.contains("kysely_migration") {
                if let Some(Value::Text(ref s)) = query.parameters.first() {
                    let mut list = self.migrated.lock().unwrap();
                    if let Some(pos) = list.iter().position(|x| x == s.as_str()) {
                        list.remove(pos);
                    }
                }
            }

            Ok(QueryResult::default())
        }
    }

    struct MockMigratorDriver {
        queries: Arc<Mutex<Vec<String>>>,
        migrated: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl Driver for MockMigratorDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockMigratorConnection {
                queries: Arc::clone(&self.queries),
                migrated: Arc::clone(&self.migrated),
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
    async fn test_migrator_execution() {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let migrated = Arc::new(Mutex::new(Vec::new()));

        let driver = Arc::new(MockMigratorDriver {
            queries: Arc::clone(&recorded),
            migrated: Arc::clone(&migrated),
        });
        let compiler = Arc::new(DefaultQueryCompiler::new());
        let dialect = CustomDialect::new(driver, compiler);
        let db = Kysely::new(dialect);

        let provider = SimpleMigrationProvider::new()
            .register(
                "001_users",
                SqlMigration::with_down(
                    "CREATE TABLE users (id SERIAL PRIMARY KEY)",
                    "DROP TABLE users",
                ),
            )
            .register(
                "002_posts",
                SqlMigration::with_down(
                    "CREATE TABLE posts (id SERIAL PRIMARY KEY)",
                    "DROP TABLE posts",
                ),
            );

        let migrator = Migrator::new(db, provider);

        // 1. Initial migrations state
        let infos = migrator.get_migrations().await.unwrap();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].executed_at, None);
        assert_eq!(infos[1].executed_at, None);

        // 2. Migrate to latest
        let res = migrator.migrate_to_latest().await.unwrap();
        assert!(res.is_success());
        assert_eq!(res.results.len(), 2);
        assert_eq!(res.results[0].migration_name, "001_users");
        assert_eq!(res.results[1].migration_name, "002_posts");

        // 3. Status now shows both executed
        let infos = migrator.get_migrations().await.unwrap();
        assert!(infos[0].executed_at.is_some());
        assert!(infos[1].executed_at.is_some());

        // 4. Migrate down one
        let res_down = migrator.migrate_down().await.unwrap();
        assert!(res_down.is_success());
        assert_eq!(res_down.results.len(), 1);
        assert_eq!(res_down.results[0].migration_name, "002_posts");
        assert_eq!(res_down.results[0].direction, "Down");

        // 5. Migrate up again
        let res_up = migrator.migrate_up().await.unwrap();
        assert!(res_up.is_success());
        assert_eq!(res_up.results.len(), 1);
        assert_eq!(res_up.results[0].migration_name, "002_posts");
        assert_eq!(res_up.results[0].direction, "Up");

        // 6. Migrate to NO_MIGRATIONS
        let res_all_down = migrator.migrate_to(NO_MIGRATIONS).await.unwrap();
        assert!(res_all_down.is_success());
        assert_eq!(res_all_down.results.len(), 2);
        assert_eq!(res_all_down.results[0].migration_name, "002_posts");
        assert_eq!(res_all_down.results[1].migration_name, "001_users");
    }
}
