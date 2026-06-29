use tauri::State;

use crate::{
    domain::{
        monitor::Monitor,
        ports::{MonitorProvider, WindowInventory},
        window::DesktopWindow,
    },
    error::PublicError,
    infrastructure::windows::Win32WindowSystem,
};

#[tauri::command]
pub fn list_monitors(system: State<'_, Win32WindowSystem>) -> Result<Vec<Monitor>, PublicError> {
    system.list_monitors().map_err(PublicError::from)
}

#[tauri::command]
pub fn list_desktop_windows(
    system: State<'_, Win32WindowSystem>,
) -> Result<Vec<DesktopWindow>, PublicError> {
    system.list_windows().map_err(PublicError::from)
}
