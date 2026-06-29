use tauri::State;

use crate::{
    domain::{monitor::Monitor, ports::MonitorProvider},
    error::PublicError,
    infrastructure::windows::Win32WindowSystem,
};

#[tauri::command]
pub fn list_monitors(system: State<'_, Win32WindowSystem>) -> Result<Vec<Monitor>, PublicError> {
    system.list_monitors().map_err(PublicError::from)
}
