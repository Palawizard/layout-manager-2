use crate::{
    domain::settings::AppSettings, error::AppError, infrastructure::persistence::Database,
};

pub struct SettingsRepository<'a> {
    database: &'a Database,
}

impl<'a> SettingsRepository<'a> {
    #[must_use]
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        self.database.with_connection(|connection| {
            let mut settings = AppSettings::default();
            let mut statement = connection
                .prepare("SELECT key, value FROM settings")
                .map_err(|error| AppError::Storage(error.to_string()))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| AppError::Storage(error.to_string()))?;
            for row in rows.flatten() {
                apply_setting(&mut settings, &row.0, &row.1)?;
            }
            settings.validate()?;
            Ok(settings)
        })
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        settings.validate()?;
        self.database.with_connection(|connection| {
            let transaction = connection
                .unchecked_transaction()
                .map_err(|error| AppError::Storage(error.to_string()))?;
            for (key, value) in settings_entries(settings)? {
                transaction
                    .execute(
                        "INSERT INTO settings (key, value) VALUES (?1, ?2)
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        (key, value),
                    )
                    .map_err(|error| AppError::Storage(error.to_string()))?;
            }
            transaction
                .commit()
                .map_err(|error| AppError::Storage(error.to_string()))?;
            Ok(())
        })
    }
}

fn apply_setting(settings: &mut AppSettings, key: &str, value: &str) -> Result<(), AppError> {
    match key {
        "preferred_browser" => {
            settings.preferred_browser = serde_json::from_str(value)
                .map_err(|_| AppError::Storage("invalid preferred browser".to_owned()))?;
        }
        "default_startup_timeout_ms" => {
            settings.default_startup_timeout_ms = value
                .parse()
                .map_err(|_| AppError::Storage("invalid startup timeout".to_owned()))?;
        }
        "monitor_fallback" => {
            settings.monitor_fallback = serde_json::from_str(value)
                .map_err(|_| AppError::Storage("invalid monitor fallback".to_owned()))?;
        }
        _ => {}
    }
    Ok(())
}

fn settings_entries(settings: &AppSettings) -> Result<[(&'static str, String); 3], AppError> {
    Ok([
        (
            "preferred_browser",
            serde_json::to_string(&settings.preferred_browser)
                .map_err(|error| AppError::Storage(error.to_string()))?,
        ),
        (
            "default_startup_timeout_ms",
            settings.default_startup_timeout_ms.to_string(),
        ),
        (
            "monitor_fallback",
            serde_json::to_string(&settings.monitor_fallback)
                .map_err(|error| AppError::Storage(error.to_string()))?,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::SettingsRepository;
    use crate::{
        domain::settings::AppSettings, infrastructure::persistence::open_in_memory_for_tests,
    };

    #[test]
    fn round_trips_application_settings() {
        let database = open_in_memory_for_tests();
        let repository = SettingsRepository::new(&database);
        let settings = AppSettings {
            default_startup_timeout_ms: 20_000,
            ..AppSettings::default()
        };
        repository.save(&settings).expect("save");
        let loaded = repository.load().expect("load");
        assert_eq!(loaded.default_startup_timeout_ms, 20_000);
    }
}
