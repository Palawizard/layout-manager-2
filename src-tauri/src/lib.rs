pub mod application;
mod commands;
pub mod domain;
pub mod error;
mod infrastructure;
mod logging;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::initialize();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "application starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let database = infrastructure::persistence::Database::open(app.handle())?;
            app.manage(database);
            Ok(())
        })
        .manage(infrastructure::windows::Win32WindowSystem::new())
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::system::list_monitors,
            commands::system::list_desktop_windows,
            commands::layouts::list_layouts,
            commands::layouts::get_layout,
            commands::layouts::save_layout,
            commands::layouts::duplicate_layout,
            commands::layouts::delete_layout,
            commands::layouts::validate_executable
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Layout Manager 2");
}
