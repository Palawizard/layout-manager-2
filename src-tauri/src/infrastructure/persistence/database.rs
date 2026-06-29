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
        fs::create_dir_all(&directory).map_err(|error| AppError::Storage(error.to_string()))?;
        let path = directory.join("layout-manager-2.sqlite");
        let connection = open_connection(&path)?;
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
        let connection = self.connection.lock().map_err(|_| AppError::Internal)?;
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
                backup_database_file(&self.path)?;
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

fn open_connection(path: &Path) -> Result<Connection, AppError> {
    match Connection::open(path) {
        Ok(connection) => {
            configure_connection(&connection)?;
            Ok(connection)
        }
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "database open failed, trying backup");
            restore_database_from_backup(path)?;
            let connection =
                Connection::open(path).map_err(|error| AppError::Storage(error.to_string()))?;
            configure_connection(&connection)?;
            Ok(connection)
        }
    }
}

fn configure_connection(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|error| AppError::Storage(error.to_string()))
}

fn backup_database_file(path: &Path) -> Result<(), AppError> {
    if !path.is_file() {
        return Ok(());
    }
    let backup_path = path.with_extension("sqlite.bak");
    fs::copy(path, &backup_path)
        .map(|_| ())
        .map_err(|error| AppError::Storage(error.to_string()))
}

fn restore_database_from_backup(path: &Path) -> Result<(), AppError> {
    let backup_path = path.with_extension("sqlite.bak");
    if !backup_path.is_file() {
        return Err(AppError::Storage(
            "La base de données est illisible et aucune sauvegarde n’est disponible.".to_owned(),
        ));
    }
    if path.is_file() {
        let corrupt_path = path.with_extension("sqlite.corrupt");
        let _ = fs::rename(path, &corrupt_path);
    }
    fs::copy(&backup_path, path)
        .map(|_| ())
        .map_err(|error| AppError::Storage(error.to_string()))
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}

#[cfg(test)]
pub(crate) fn open_in_memory_for_tests() -> Database {
    let connection = Connection::open_in_memory().expect("in-memory database");
    configure_connection(&connection).expect("pragmas");
    let database = Database {
        path: std::env::temp_dir().join("layout-manager-test.sqlite"),
        connection: Mutex::new(connection),
    };
    database.migrate().expect("migration");
    database
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        Database, backup_database_file, open_in_memory_for_tests, restore_database_from_backup,
    };
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
                    .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
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

    #[test]
    fn restores_a_database_from_backup_after_corruption() {
        let directory = std::env::temp_dir().join("layout-manager-db-test-restore");
        fs::create_dir_all(&directory).expect("directory");
        let path = directory.join("layout-manager-2.sqlite");
        fs::write(&path, b"not-a-database").expect("corrupt file");
        fs::write(path.with_extension("sqlite.bak"), b"not-a-database").expect("backup");

        restore_database_from_backup(&path).expect("restore");
        assert!(path.is_file());
        assert!(directory.join("layout-manager-2.sqlite.corrupt").is_file());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn creates_a_backup_before_the_first_migration() {
        let directory = std::env::temp_dir().join("layout-manager-db-backup-test");
        fs::create_dir_all(&directory).expect("directory");
        let path = directory.join("layout-manager-2.sqlite");
        fs::write(&path, b"seed").expect("seed");

        backup_database_file(&path).expect("backup");
        assert!(path.with_extension("sqlite.bak").is_file());

        let _ = fs::remove_dir_all(directory);
    }
}
