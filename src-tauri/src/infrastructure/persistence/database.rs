use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::AppError;

const MIGRATION_001: &str = include_str!("../../../migrations/001_initial.sql");

struct Migration {
    version: i64,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: MIGRATION_001,
}];

pub struct Database {
    path: PathBuf,
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(app: &AppHandle) -> Result<Self, AppError> {
        let directory = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::Storage(error.to_string()))?;
        fs::create_dir_all(&directory)
            .map_err(|error| AppError::Storage(error.to_string()))?;
        let path = directory.join("layout-manager-2.sqlite");
        let connection =
            Connection::open(&path).map_err(|error| AppError::Storage(error.to_string()))?;
        connection
            .execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|error| AppError::Storage(error.to_string()))?;
        let database = Self {
            path,
            connection: Mutex::new(connection),
        };
        database.migrate()?;
        Ok(database)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::Internal)?;
        operation(&connection)
    }

    fn migrate(&self) -> Result<(), AppError> {
        self.with_connection(|connection| {
            for migration in MIGRATIONS {
                let already_applied: bool = connection
                    .query_row(
                        "SELECT 1 FROM schema_migrations WHERE version = ?1",
                        [migration.version],
                        |_| Ok(()),
                    )
                    .is_ok();
                if already_applied {
                    continue;
                }
                connection
                    .execute_batch(migration.sql)
                    .map_err(|error| AppError::Storage(error.to_string()))?;
                connection
                    .execute(
                        "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                        (migration.version, current_timestamp()),
                    )
                    .map_err(|error| AppError::Storage(error.to_string()))?;
            }
            Ok(())
        })
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}

#[cfg(test)]
pub(crate) fn open_in_memory_for_tests() -> Database {
    let connection = Connection::open_in_memory().expect("in-memory database");
    connection
        .execute("PRAGMA foreign_keys = ON;", [])
        .expect("foreign keys");
    let database = Database {
        path: std::env::temp_dir().join("layout-manager-test.sqlite"),
        connection: Mutex::new(connection),
    };
    database.migrate().expect("migration");
    database
}

#[cfg(test)]
mod tests {
    use super::{open_in_memory_for_tests, Database};
    use crate::error::AppError;

    fn open_test_database() -> Database {
        open_in_memory_for_tests()
    }

    #[test]
    fn applies_the_initial_migration_from_an_empty_database() {
        let database = open_test_database();
        database
            .with_connection(|connection| {
                let tables: Vec<String> = connection
                    .prepare(
                        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
                    )
                    .expect("query")
                    .query_map([], |row| row.get(0))
                    .expect("rows")
                    .filter_map(Result::ok)
                    .collect();
                assert!(tables.contains(&"layouts".to_owned()));
                assert!(tables.contains(&"layout_actions".to_owned()));
                assert!(tables.contains(&"settings".to_owned()));
                Ok::<(), AppError>(())
            })
            .expect("tables");
    }
}
