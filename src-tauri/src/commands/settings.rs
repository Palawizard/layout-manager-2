use tauri::State;

use crate::{
    domain::settings::AppSettings,
    error::{AppError, PublicError},
    infrastructure::persistence::{Database, SettingsRepository},
};

#[tauri::command]
pub fn get_settings(database: State<'_, Database>) -> Result<AppSettings, PublicError> {
    SettingsRepository::new(&database)
        .load()
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn save_settings(
    database: State<'_, Database>,
    settings: AppSettings,
) -> Result<AppSettings, PublicError> {
    let repository = SettingsRepository::new(&database);
    repository.save(&settings)?;
    repository.load().map_err(PublicError::from)
}

#[tauri::command]
pub fn open_logs_directory(app: tauri::AppHandle) -> Result<(), PublicError> {
    use tauri_plugin_opener::OpenerExt;

    let directory = crate::logging::log_directory(&app).map_err(|error| {
        AppError::Storage(error.to_string())
    })?;
    std::fs::create_dir_all(&directory).map_err(|error| AppError::Storage(error.to_string()))?;
    app.opener()
        .open_path(directory.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Storage(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn open_data_directory(
    app: tauri::AppHandle,
    database: State<'_, Database>,
) -> Result<(), PublicError> {
    use tauri_plugin_opener::OpenerExt;

    let directory = database
        .path()
        .parent()
        .ok_or(AppError::Internal)?
        .to_path_buf();
    app.opener()
        .open_path(directory.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Storage(error.to_string()))?;
    Ok(())
}
