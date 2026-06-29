pub mod application;
mod commands;
pub mod domain;
pub mod error;
mod infrastructure;
mod logging;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::initialize();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "application starting");

    tauri::Builder::default()
        .manage(infrastructure::windows::Win32WindowSystem::new())
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::system::list_monitors,
            commands::system::list_desktop_windows
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Layout Manager 2");
}
