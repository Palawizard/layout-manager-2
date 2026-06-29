use std::mem::size_of;

use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
    UI::WindowsAndMessaging::{
        GetWindowRect, IsWindow, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SWP_NOACTIVATE,
        SWP_NOOWNERZORDER, SWP_NOZORDER, SetWindowPos, ShowWindow,
    },
};

use super::Win32WindowSystem;
use crate::domain::{
    geometry::PixelBounds,
    ports::{NativeError, WindowController},
    window::{NativeWindowHandle, WindowState},
};

const VISIBLE_ALIGNMENT_ITERATIONS: usize = 3;

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
            set_outer_bounds(window, bounds)?;
            align_visible_bounds(window, bounds)?;
        }
        Ok(())
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

fn set_outer_bounds(window: HWND, bounds: PixelBounds) -> Result<(), NativeError> {
    // SAFETY: The HWND was validated and the bounds are finite pixel coordinates.
    unsafe {
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
    .map_err(map_win32_error)
}

fn map_win32_error(error: windows::core::Error) -> NativeError {
    const E_ACCESSDENIED: i32 = 0x8007_0005;
    if error.code().0 == E_ACCESSDENIED {
        NativeError::AccessDenied
    } else {
        NativeError::OperationFailed(error.to_string())
    }
}

fn align_visible_bounds(window: HWND, target: PixelBounds) -> Result<(), NativeError> {
    for _ in 0..VISIBLE_ALIGNMENT_ITERATIONS {
        let Some(visible) = extended_frame_bounds(window) else {
            return Ok(());
        };
        let delta_x = target.x - visible.x;
        let delta_y = target.y - visible.y;
        let delta_w = target.width - visible.width;
        let delta_h = target.height - visible.height;
        if delta_x == 0 && delta_y == 0 && delta_w == 0 && delta_h == 0 {
            return Ok(());
        }
        let outer = outer_bounds(window)?;
        set_outer_bounds(
            window,
            PixelBounds {
                x: outer.x + delta_x,
                y: outer.y + delta_y,
                width: outer.width + delta_w,
                height: outer.height + delta_h,
            },
        )?;
    }
    Ok(())
}

fn outer_bounds(window: HWND) -> Result<PixelBounds, NativeError> {
    let mut rect = RECT::default();
    // SAFETY: `rect` is writable and the HWND was validated by the caller.
    unsafe { GetWindowRect(window, &mut rect) }
        .map_err(map_win32_error)?;
    Ok(rect_to_bounds(rect))
}

fn extended_frame_bounds(window: HWND) -> Option<PixelBounds> {
    let mut rect = RECT::default();
    // SAFETY: `rect` matches the requested DWM attribute size and the HWND is valid.
    let result = unsafe {
        DwmGetWindowAttribute(
            window,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&raw mut rect).cast(),
            size_of::<RECT>() as u32,
        )
    };
    result.is_ok().then(|| rect_to_bounds(rect))
}

fn rect_to_bounds(rect: RECT) -> PixelBounds {
    PixelBounds {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}
