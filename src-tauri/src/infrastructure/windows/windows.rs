use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute},
        System::Threading::GetCurrentProcessId,
        UI::WindowsAndMessaging::{
            EnumWindows, GW_OWNER, GetClassNameW, GetWindow, GetWindowRect, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
        },
    },
    core::BOOL,
};

use super::{Win32WindowSystem, monitors::monitor_id_from_window, process::process_metadata};
use crate::domain::{
    geometry::PixelBounds,
    ports::{NativeError, WindowInventory},
    window::{DesktopWindow, NativeWindowHandle, WindowState},
};

impl WindowInventory for Win32WindowSystem {
    fn list_windows(&self) -> Result<Vec<DesktopWindow>, NativeError> {
        let mut handles = Vec::<HWND>::new();
        // SAFETY: The pointer in LPARAM refers to `handles` for the entire synchronous enumeration.
        if unsafe {
            EnumWindows(
                Some(collect_window),
                LPARAM((&raw mut handles).cast::<()>() as isize),
            )
        }
        .is_err()
        {
            return Err(NativeError::OperationFailed(
                "window enumeration".to_owned(),
            ));
        }
        Ok(handles.into_iter().filter_map(inspect_window).collect())
    }
}

unsafe extern "system" fn collect_window(window: HWND, data: LPARAM) -> BOOL {
    // SAFETY: `data` originates from a live, exclusively borrowed vector in `list_windows`.
    unsafe { &mut *(data.0 as *mut Vec<HWND>) }.push(window);
    BOOL(1)
}

fn inspect_window(window: HWND) -> Option<DesktopWindow> {
    // SAFETY: The handle came from EnumWindows and each query is read-only.
    if !unsafe { IsWindowVisible(window) }.as_bool()
        || unsafe { GetWindow(window, GW_OWNER) }.ok().is_some()
    {
        return None;
    }
    let mut cloaked = 0u32;
    // SAFETY: The output buffer matches the requested u32 DWMWA_CLOAKED attribute.
    if unsafe {
        DwmGetWindowAttribute(
            window,
            DWMWA_CLOAKED,
            (&raw mut cloaked).cast(),
            size_of::<u32>() as u32,
        )
    }
    .is_ok_and(|()| cloaked != 0)
    {
        return None;
    }
    let mut process_id = 0u32;
    // SAFETY: The output pointer is valid and the handle came from EnumWindows.
    unsafe { GetWindowThreadProcessId(window, Some(&raw mut process_id)) };
    // SAFETY: This function has no preconditions.
    if process_id == unsafe { GetCurrentProcessId() } {
        return None;
    }
    let title = window_text(window);
    let class_name = window_class(window);
    if title.trim().is_empty()
        || matches!(class_name.as_str(), "Shell_TrayWnd" | "Progman" | "WorkerW")
    {
        return None;
    }
    let mut rect = RECT::default();
    // SAFETY: `rect` is writable and the handle came from EnumWindows.
    if unsafe { GetWindowRect(window, &raw mut rect) }.is_err() {
        return None;
    }
    let state = if unsafe { IsIconic(window) }.as_bool() {
        WindowState::Minimized
    } else if unsafe { IsZoomed(window) }.as_bool() {
        WindowState::Maximized
    } else {
        WindowState::Normal
    };
    let (executable_path, process_name) = process_metadata(process_id);
    Some(DesktopWindow {
        handle: NativeWindowHandle(window.0 as isize),
        process_id,
        executable_path,
        process_name,
        title,
        class_name,
        bounds: PixelBounds {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        },
        state,
        monitor_id: monitor_id_from_window(window),
    })
}

fn window_text(window: HWND) -> String {
    // SAFETY: The handle came from EnumWindows.
    let length = unsafe { GetWindowTextLengthW(window) };
    if length <= 0 {
        return String::new();
    }
    let mut buffer = vec![0u16; length as usize + 1];
    // SAFETY: The buffer has room for the reported text and its null terminator.
    let copied = unsafe { GetWindowTextW(window, &mut buffer) };
    String::from_utf16_lossy(&buffer[..copied.max(0) as usize])
}

fn window_class(window: HWND) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: The buffer is writable and its length is supplied to Windows.
    let copied = unsafe { GetClassNameW(window, &mut buffer) };
    String::from_utf16_lossy(&buffer[..copied.max(0) as usize])
}

use std::mem::size_of;
