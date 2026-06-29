use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        IsWindow, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
        SWP_NOZORDER, SetWindowPos, ShowWindow,
    },
};

use super::Win32WindowSystem;
use crate::domain::{
    geometry::PixelBounds,
    ports::{NativeError, WindowController},
    window::{NativeWindowHandle, WindowState},
};

impl WindowController for Win32WindowSystem {
    fn place_window(
        &self,
        handle: NativeWindowHandle,
        bounds: PixelBounds,
    ) -> Result<(), NativeError> {
        let window = as_hwnd(handle)?;
        // SAFETY: The handle is validated immediately before these window-management operations.
        unsafe {
            let _ = ShowWindow(window, SW_RESTORE);
            SetWindowPos(
                window,
                None,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER,
            )
        }
        .map_err(|error| NativeError::OperationFailed(error.to_string()))
    }

    fn set_window_state(
        &self,
        handle: NativeWindowHandle,
        state: WindowState,
    ) -> Result<(), NativeError> {
        let window = as_hwnd(handle)?;
        let command = match state {
            WindowState::Normal => SW_RESTORE,
            WindowState::Maximized => SW_MAXIMIZE,
            WindowState::Minimized => SW_MINIMIZE,
        };
        // SAFETY: The handle is validated and the command is a predefined ShowWindow value.
        let _ = unsafe { ShowWindow(window, command) };
        Ok(())
    }
}

fn as_hwnd(handle: NativeWindowHandle) -> Result<HWND, NativeError> {
    let window = HWND(handle.0 as *mut _);
    // SAFETY: IsWindow accepts any HWND value and does not dereference application memory.
    if handle.0 == 0 || !unsafe { IsWindow(Some(window)) }.as_bool() {
        return Err(NativeError::InvalidHandle);
    }
    Ok(window)
}
