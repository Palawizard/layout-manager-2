use std::{
    collections::HashSet,
    thread,
    time::{Duration, Instant},
};

use crate::{
    application::window_matcher::{MatchContext, WindowMatchError, select_window},
    domain::{
        ports::WindowInventory,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitError {
    Timeout,
    Cancelled,
    NotFound,
    Ambiguous,
    InventoryFailed,
}

pub trait CancellationCheck: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct NeverCancelled;

impl CancellationCheck for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct SharedCancellation {
    cancelled: std::sync::atomic::AtomicBool,
}

impl SharedCancellation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl CancellationCheck for SharedCancellation {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl CancellationCheck for std::sync::Arc<SharedCancellation> {
    fn is_cancelled(&self) -> bool {
        self.as_ref().is_cancelled()
    }
}

pub fn snapshot_handles(inventory: &impl WindowInventory) -> HashSet<NativeWindowHandle> {
    inventory
        .list_windows()
        .unwrap_or_default()
        .into_iter()
        .map(|window| window.handle)
        .collect()
}

pub fn wait_for_window(
    inventory: &impl WindowInventory,
    matcher: &WindowMatcher,
    previous_handles: &HashSet<NativeWindowHandle>,
    launched_process_id: Option<u32>,
    timeout_ms: u32,
    cancellation: &impl CancellationCheck,
) -> Result<DesktopWindow, WaitError> {
    let deadline = Instant::now() + Duration::from_millis(u64::from(timeout_ms));
    let mut delay_ms = 50u64;

    loop {
        if cancellation.is_cancelled() {
            return Err(WaitError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(WaitError::Timeout);
        }

        let windows = inventory
            .list_windows()
            .map_err(|_| WaitError::InventoryFailed)?;
        let context = MatchContext {
            launched_process_id,
            previous_handles: previous_handles.clone(),
        };
        match select_window(matcher, &windows, &context) {
            Ok(window) => return Ok(window.clone()),
            Err(WindowMatchError::NotFound) => {}
            Err(WindowMatchError::Ambiguous) => return Err(WaitError::Ambiguous),
        }

        thread::sleep(Duration::from_millis(delay_ms));
        delay_ms = delay_ms.saturating_mul(2).min(500);
    }
}

pub fn find_existing_window(
    inventory: &impl WindowInventory,
    matcher: &WindowMatcher,
) -> Result<Option<DesktopWindow>, WaitError> {
    let windows = inventory
        .list_windows()
        .map_err(|_| WaitError::InventoryFailed)?;
    let context = MatchContext::default();
    match select_window(matcher, &windows, &context) {
        Ok(window) => Ok(Some(window.clone())),
        Err(WindowMatchError::NotFound) => Ok(None),
        Err(WindowMatchError::Ambiguous) => Err(WaitError::Ambiguous),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NeverCancelled, SharedCancellation, WaitError, find_existing_window, wait_for_window,
    };
    use std::{
        collections::HashSet,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };
    use crate::domain::{
        geometry::PixelBounds,
        ports::fakes::FakeWindowSystem,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };

    struct FlagCancellation(Arc<AtomicBool>);

    impl super::CancellationCheck for FlagCancellation {
        fn is_cancelled(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }
    }

    fn window(handle: isize, pid: u32) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(handle),
            process_id: pid,
            executable_path: Some("C:\\Apps\\Editor.exe".to_owned()),
            process_name: Some("Editor.exe".to_owned()),
            title: "Doc".to_owned(),
            class_name: "Editor".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            state: WindowState::Normal,
            monitor_id: None,
        }
    }

    #[test]
    fn waits_until_a_new_window_appears() {
        struct AppearingInventory {
            calls: std::sync::Mutex<usize>,
            window: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for AppearingInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                let mut calls = self.calls.lock().expect("calls");
                *calls += 1;
                if *calls >= 2 {
                    Ok(vec![self.window.clone()])
                } else {
                    Ok(vec![])
                }
            }
        }

        let inventory = AppearingInventory {
            calls: std::sync::Mutex::new(0),
            window: window(2, 20),
        };
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        let previous = HashSet::from([NativeWindowHandle(1)]);

        let found = wait_for_window(
            &inventory,
            &matcher,
            &previous,
            Some(20),
            2_000,
            &NeverCancelled,
        )
        .expect("window appears");
        assert_eq!(found.handle, NativeWindowHandle(2));
    }

    #[test]
    fn returns_timeout_when_no_window_appears() {
        let system = FakeWindowSystem::default();
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            wait_for_window(
                &system,
                &matcher,
                &HashSet::new(),
                None,
                100,
                &NeverCancelled,
            ),
            Err(WaitError::Timeout)
        );
    }

    #[test]
    fn stops_waiting_when_cancelled() {
        let cancellation = SharedCancellation::new();
        cancellation.cancel();
        let system = FakeWindowSystem::default();
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            wait_for_window(
                &system,
                &matcher,
                &HashSet::new(),
                None,
                5_000,
                &cancellation,
            ),
            Err(WaitError::Cancelled)
        );
    }

    #[test]
    fn finds_an_existing_window_without_waiting() {
        let system = FakeWindowSystem {
            windows: vec![window(1, 10)],
            ..Default::default()
        };
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        let found = find_existing_window(&system, &matcher)
            .expect("inventory")
            .expect("match");
        assert_eq!(found.process_id, 10);
    }
}
