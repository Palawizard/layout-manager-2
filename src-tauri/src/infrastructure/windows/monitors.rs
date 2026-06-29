use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST,
            MONITORINFO, MONITORINFOEXW, MonitorFromWindow,
        },
        UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
        UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
    },
    core::BOOL,
};

use super::Win32WindowSystem;
use crate::domain::{
    geometry::WorkArea,
    monitor::{Monitor, MonitorId},
    ports::{MonitorProvider, NativeError},
};

impl MonitorProvider for Win32WindowSystem {
    fn list_monitors(&self) -> Result<Vec<Monitor>, NativeError> {
        let mut handles = Vec::<HMONITOR>::new();
        // SAFETY: The callback only appends valid handles during this synchronous call. LPARAM
        // points to `handles`, which remains alive and exclusively borrowed until enumeration ends.
        let result = unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(collect_monitor),
                LPARAM((&raw mut handles).cast::<()>() as isize),
            )
        };
        if !result.as_bool() {
            return Err(NativeError::OperationFailed(
                "monitor enumeration".to_owned(),
            ));
        }
        handles.into_iter().map(inspect_monitor).collect()
    }
}

unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _device_context: HDC,
    _bounds: *mut RECT,
    data: LPARAM,
) -> BOOL {
    // SAFETY: `data` is created from a live `Vec<HMONITOR>` in `list_monitors`, and Windows calls
    // this callback synchronously before that vector is dropped.
    let monitors = unsafe { &mut *(data.0 as *mut Vec<HMONITOR>) };
    monitors.push(monitor);
    BOOL(1)
}

fn inspect_monitor(handle: HMONITOR) -> Result<Monitor, NativeError> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: `info` has the required size and remains writable for the duration of the call.
    if !unsafe { GetMonitorInfoW(handle, (&raw mut info).cast::<MONITORINFO>()) }.as_bool() {
        return Err(NativeError::OperationFailed(
            "monitor inspection".to_owned(),
        ));
    }
    let name = String::from_utf16_lossy(
        &info.szDevice[..info
            .szDevice
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(32)],
    );
    let work = info.monitorInfo.rcWork;
    let mut dpi_x = 96;
    let mut dpi_y = 96;
    // SAFETY: Output pointers reference initialized local integers and the monitor handle came from
    // `EnumDisplayMonitors`. A failure keeps the safe 96-DPI fallback.
    let _ = unsafe { GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &raw mut dpi_x, &raw mut dpi_y) };

    Ok(Monitor {
        id: MonitorId(name.clone()),
        name,
        work_area: WorkArea {
            x: work.left,
            y: work.top,
            width: work.right - work.left,
            height: work.bottom - work.top,
        },
        scale_factor: f64::from(dpi_x) / 96.0,
        is_primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
    })
}

pub(super) fn monitor_id_from_window(window: HWND) -> Option<MonitorId> {
    // SAFETY: The handle is inspected only to obtain its nearest monitor.
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    inspect_monitor(monitor).ok().map(|monitor| monitor.id)
}
