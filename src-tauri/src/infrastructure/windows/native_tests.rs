use std::{thread, time::Duration};

use windows::{
    Win32::{
        Foundation::{HWND, RECT},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetWindowRect, IsIconic, WINDOW_EX_STYLE,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
    core::w,
};

use super::Win32WindowSystem;
use crate::domain::{
    geometry::PixelBounds,
    ports::{MonitorProvider, WindowController, WindowInventory},
    window::{NativeWindowHandle, WindowState},
};

struct TestWindow(HWND);

impl TestWindow {
    fn create() -> Self {
        // SAFETY: STATIC is a predefined Windows class; all optional ownership parameters are null.
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Layout Manager Native Test"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100,
                100,
                640,
                480,
                None,
                None,
                None,
                None,
            )
        }
        .expect("test window should be created");
        Self(window)
    }

    fn handle(&self) -> NativeWindowHandle {
        NativeWindowHandle(self.0.0 as isize)
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        // SAFETY: The guard owns this dedicated test window and destroys it exactly once.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}

#[test]
#[ignore = "inspects the interactive Windows desktop"]
fn inspect_steam_windows() {
    let system = Win32WindowSystem::new();
    let windows = system
        .list_windows_including_untitled()
        .expect("windows should be enumerable");
    let steam: Vec<_> = windows
        .into_iter()
        .filter(|window| {
            window.process_name.as_deref().is_some_and(|name| {
                name.eq_ignore_ascii_case("steam.exe")
                    || name.eq_ignore_ascii_case("steamwebhelper.exe")
            })
        })
        .collect();
    eprintln!(
        "Steam-related windows visible to the app ({}):",
        steam.len()
    );
    for window in &steam {
        eprintln!(
            "  handle={} pid={} class={} title={:?} {}x{} @({},{}) state={:?}",
            window.handle.0,
            window.process_id,
            window.class_name,
            window.title,
            window.bounds.width,
            window.bounds.height,
            window.bounds.x,
            window.bounds.y,
            window.state,
        );
    }
    assert!(system.list_windows().is_ok());
}

#[test]
#[ignore = "inspects the interactive Windows desktop"]
fn enumerates_native_monitors_and_windows() {
    let system = Win32WindowSystem::new();
    let monitors = system
        .list_monitors()
        .expect("monitors should be available");
    assert!(!monitors.is_empty());
    assert!(monitors.iter().all(|monitor| monitor.work_area.width > 0));
    assert!(system.list_windows().is_ok());
}

#[test]
#[ignore = "creates and moves a visible native test window"]
fn places_and_minimizes_a_dedicated_test_window() {
    let system = Win32WindowSystem::new();
    let window = TestWindow::create();
    let destination = PixelBounds {
        x: 120,
        y: 140,
        width: 700,
        height: 500,
    };

    system
        .place_window(window.handle(), destination)
        .expect("window should move");
    let mut actual = RECT::default();
    // SAFETY: The window guard keeps the handle alive and `actual` is writable.
    unsafe { GetWindowRect(window.0, &raw mut actual) }.expect("bounds should be readable");
    assert_eq!((actual.left, actual.top), (destination.x, destination.y));

    system
        .set_window_state(window.handle(), WindowState::Minimized)
        .expect("window should minimize");
    thread::sleep(Duration::from_millis(100));
    // SAFETY: The window remains alive for this state query.
    assert!(unsafe { IsIconic(window.0) }.as_bool());
}
