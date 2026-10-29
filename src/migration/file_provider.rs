//! File-system based migration provider loading `.sql` migrations from disk.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use compact_str::CompactString;

use crate::driver::DriverError;
use crate::migration::traits::{Migration, MigrationProvider, SqlMigration};

/// Migration provider discovering and loading SQL migration files from a folder.
// @spec REQ-MIG-003
#[derive(Debug, Clone)]
pub struct FileMigrationProvider {
    folder: PathBuf,
}

impl FileMigrationProvider {
    /// Creates a new `FileMigrationProvider` pointing to the specified directory.
    pub fn from_folder(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into(),
        }
    }

    /// Returns the directory path this provider loads migrations from.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Internal synchronous loader discovering and parsing files in the directory.
    fn load_migrations_sync(&self) -> Result<BTreeMap<CompactString, Arc<dyn Migration>>, DriverError> {
        if !self.folder.exists() {
            return Err(DriverError::ExecutionFailed(format!(
                "Migration directory '{}' does not exist",
                self.folder.display()
            )));
        }

        let entries = std::fs::read_dir(&self.folder).map_err(|e| {
            DriverError::ExecutionFailed(format!(
                "Failed to read migration directory '{}': {}",
                self.folder.display(),
                e
            ))
        })?;

        let mut raw_migrations: BTreeMap<String, (Option<String>, Option<String>)> = BTreeMap::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                DriverError::ExecutionFailed(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|f| f.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            if file_name.ends_with(".up.sql") {
                let name = file_name[..file_name.len() - 7].to_string();
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    DriverError::ExecutionFailed(format!(
                        "Failed to read migration file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                raw_migrations.entry(name).or_insert((None, None)).0 = Some(content);
            } else if file_name.ends_with(".down.sql") {
                let name = file_name[..file_name.len() - 9].to_string();
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    DriverError::ExecutionFailed(format!(
                        "Failed to read migration file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                raw_migrations.entry(name).or_insert((None, None)).1 = Some(content);
            } else if file_name.ends_with(".sql") {
                let name = file_name[..file_name.len() - 4].to_string();
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    DriverError::ExecutionFailed(format!(
                        "Failed to read migration file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                raw_migrations.entry(name).or_insert((None, None)).0 = Some(content);
            }
        }

        let mut migrations: BTreeMap<CompactString, Arc<dyn Migration>> = BTreeMap::new();

        for (name, (up_opt, down_opt)) in raw_migrations {
            let up_sql = up_opt.unwrap_or_default();
            let migration: Arc<dyn Migration> = if let Some(down_sql) = down_opt {
                Arc::new(SqlMigration::with_down(up_sql, down_sql))
            } else {
                Arc::new(SqlMigration::new(up_sql))
            };

            migrations.insert(CompactString::from(name), migration);
        }

        Ok(migrations)
    }
}

#[async_trait]
impl MigrationProvider for FileMigrationProvider {
    async fn get_migrations(&self) -> Result<BTreeMap<CompactString, Arc<dyn Migration>>, DriverError> {
        self.load_migrations_sync()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_migration_provider() {
        let temp_dir = std::env::temp_dir().join(format!("kysely_test_migrations_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 1. Create migration files
        std::fs::write(
            temp_dir.join("001_users.up.sql"),
            "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);",
        )
        .unwrap();
        std::fs::write(
            temp_dir.join("001_users.down.sql"),
            "DROP TABLE users;",
        )
        .unwrap();

        std::fs::write(
            temp_dir.join("002_posts.sql"),
            "CREATE TABLE posts (id SERIAL PRIMARY KEY, title TEXT);",
        )
        .unwrap();
        std::fs::write(
            temp_dir.join("002_posts.down.sql"),
            "DROP TABLE posts;",
        )
        .unwrap();

        // Non-migration file that should be ignored
        std::fs::write(temp_dir.join("README.txt"), "This is a documentation file.").unwrap();

        let provider = FileMigrationProvider::from_folder(&temp_dir);
        let migrations = provider.get_migrations().await.unwrap();

        assert_eq!(migrations.len(), 2);

        let keys: Vec<CompactString> = migrations.keys().cloned().collect();
        assert_eq!(keys, vec![
            CompactString::from("001_users"),
            CompactString::from("002_posts"),
        ]);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
