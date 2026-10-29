//! Migration traits and data structures.

use std::collections::BTreeMap;
use std::sync::Arc;
use async_trait::async_trait;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::query_compiler::CompiledQuery;
use crate::transaction::Transaction;
use crate::util::QueryId;

/// Execution status of a migration step.
// @spec REQ-MIG-008
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    /// The migration succeeded.
    Success,
    /// The migration failed.
    Error,
    /// The migration was not executed.
    NotExecuted,
}

/// Result of executing a single migration step.
// @spec REQ-MIG-008
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationResult {
    /// The name of the migration.
    pub migration_name: CompactString,
    /// Direction: "Up" or "Down".
    pub direction: CompactString,
    /// Execution status.
    pub status: MigrationStatus,
    /// Optional error message if the migration failed.
    pub error: Option<String>,
}

/// Information describing a migration's execution state.
// @spec REQ-MIG-001, REQ-MIG-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationInfo {
    /// Name/identifier of the migration (e.g. "001_create_users").
    pub name: CompactString,
    /// ISO 8601 timestamp string when the migration was executed, or None if pending.
    pub executed_at: Option<CompactString>,
}

/// An individual database migration containing up and down migration steps.
// @spec REQ-MIG-002
#[async_trait]
pub trait Migration: Send + Sync {
    /// Executes the migration forward.
    async fn up(&self, db: &Transaction) -> Result<(), DriverError>;

    /// Executes the migration rollback (optional, default no-op).
    async fn down(&self, _db: &Transaction) -> Result<(), DriverError> {
        Ok(())
    }
}

/// Provider responsible for discovering and loading migrations.
// @spec REQ-MIG-002
#[async_trait]
pub trait MigrationProvider: Send + Sync {
    /// Returns a map of migration name to `Arc<dyn Migration>`, sorted by migration name.
    async fn get_migrations(&self) -> Result<BTreeMap<CompactString, Arc<dyn Migration>>, DriverError>;
}

/// A migration executed from raw SQL string statements.
// @spec REQ-MIG-002, REQ-MIG-003
#[derive(Debug, Clone)]
pub struct SqlMigration {
    up_sql: String,
    down_sql: Option<String>,
}

impl SqlMigration {
    /// Creates a new `SqlMigration` with an `up` SQL string.
    pub fn new(up_sql: impl Into<String>) -> Self {
        Self {
            up_sql: up_sql.into(),
            down_sql: None,
        }
    }

    /// Creates a new `SqlMigration` with both `up` and `down` SQL strings.
    pub fn with_down(up_sql: impl Into<String>, down_sql: impl Into<String>) -> Self {
        Self {
            up_sql: up_sql.into(),
            down_sql: Some(down_sql.into()),
        }
    }
}

#[async_trait]
impl Migration for SqlMigration {
    async fn up(&self, db: &Transaction) -> Result<(), DriverError> {
        let trimmed = self.up_sql.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let query = CompiledQuery::new(trimmed, vec![], QueryId::generate());
        db.execute_query(&query).await.map(|_| ())
    }

    async fn down(&self, db: &Transaction) -> Result<(), DriverError> {
        if let Some(ref down_sql) = self.down_sql {
            let trimmed = down_sql.trim();
            if trimmed.is_empty() {
                return Ok(());
            }
            let query = CompiledQuery::new(trimmed, vec![], QueryId::generate());
            db.execute_query(&query).await.map(|_| ())
        } else {
            Ok(())
        }
    }
}
