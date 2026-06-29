use std::{
    collections::HashSet,
    thread,
    time::{Duration, Instant},
};

use crate::{
    application::{
        client_window::{is_non_client_window, is_placeable_client_window},
        durable_window::{is_suspect_launch_candidate, window_area},
        post_launch::is_indirect_launch,
        window_matcher::{MatchContext, WindowMatchError, rank_window_matches, select_window},
    },
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
    InstanceNotFound { requested: usize, available: usize },
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
    launched_executable_path: Option<&str>,
    timeout_ms: u32,
    cancellation: &impl CancellationCheck,
) -> Result<DesktopWindow, WaitError> {
    let indirect_launch =
        launched_executable_path.is_some_and(|path| is_indirect_launch(path, matcher));
    let effective_timeout_ms = if indirect_launch {
        u64::from(timeout_ms).clamp(30_000, 120_000) as u32
    } else {
        timeout_ms
    };
    let deadline = Instant::now() + Duration::from_millis(u64::from(effective_timeout_ms));
    let launch_started = Instant::now();
    let suspect_only_grace = if indirect_launch {
        Duration::from_millis(
            u64::from(effective_timeout_ms)
                .saturating_sub(1_000)
                .clamp(10_000, 60_000),
        )
    } else {
        Duration::from_millis((u64::from(effective_timeout_ms) / 2).clamp(2_000, 8_000))
    };
    let stable_duration = if indirect_launch {
        Duration::from_millis(750)
    } else {
        Duration::from_millis(500)
    };
    let mut delay_ms = 50u64;
    let mut tracked_handle: Option<NativeWindowHandle> = None;
    let mut tracked_since: Option<Instant> = None;
    let post_launch = launched_process_id.is_some();

    loop {
        if cancellation.is_cancelled() {
            return Err(WaitError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(WaitError::Timeout);
        }

        let windows = if post_launch {
            inventory
                .list_windows_including_untitled()
                .map_err(|_| WaitError::InventoryFailed)?
        } else {
            inventory
                .list_windows()
                .map_err(|_| WaitError::InventoryFailed)?
        };
        let context = MatchContext {
            launched_process_id,
            previous_handles: previous_handles.clone(),
            process_hierarchy: Some(inventory),
        };
        let ranked = rank_window_matches(matcher, &windows, &context);
        let durable: Vec<_> = ranked
            .iter()
            .filter(|candidate| !is_non_client_window(candidate.window))
            .collect();
        let candidate_pool: Vec<_> = if post_launch && !previous_handles.is_empty() {
            durable
                .iter()
                .copied()
                .filter(|entry| !previous_handles.contains(&entry.window.handle))
                .collect()
        } else {
            durable.to_vec()
        };
        let Some(candidate) = candidate_pool
            .iter()
            .max_by_key(|entry| window_area(entry.window))
            .map(|entry| entry.window)
        else {
            tracked_handle = None;
            tracked_since = None;
            thread::sleep(Duration::from_millis(delay_ms));
            delay_ms = delay_ms.saturating_mul(2).min(500);
            continue;
        };
        if durable.len() > 1 && matcher.instance_index.is_none() {
            let top_score = ranked[0].score;
            let tied: Vec<_> = ranked
                .iter()
                .filter(|entry| entry.score == top_score)
                .collect();
            if tied.len() > 1 {
                let max_area = tied
                    .iter()
                    .map(|entry| window_area(entry.window))
                    .max()
                    .expect("non-empty tie group");
                if tied
                    .iter()
                    .filter(|entry| window_area(entry.window) == max_area)
                    .count()
                    > 1
                {
                    return Err(WaitError::Ambiguous);
                }
            }
        }

        let suspect = is_suspect_launch_candidate(candidate, matcher, &context);
        let sole_match = candidate_pool.len() == 1;

        if tracked_handle == Some(candidate.handle) {
            if tracked_since.is_some_and(|started| started.elapsed() >= stable_duration)
                && (!suspect || !sole_match || launch_started.elapsed() >= suspect_only_grace)
            {
                return Ok(candidate.clone());
            }
        } else {
            tracked_handle = Some(candidate.handle);
            tracked_since = Some(Instant::now());
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
        Err(WindowMatchError::InstanceNotFound {
            requested,
            available,
        }) => Err(WaitError::InstanceNotFound {
            requested,
            available,
        }),
    }
}

pub fn refresh_matched_handle(
    inventory: &impl WindowInventory,
    matcher: &WindowMatcher,
    fallback: NativeWindowHandle,
) -> NativeWindowHandle {
    resolve_client_window_handle(inventory, matcher, fallback)
}

pub fn resolve_client_window_handle(
    inventory: &impl WindowInventory,
    matcher: &WindowMatcher,
    fallback: NativeWindowHandle,
) -> NativeWindowHandle {
    let Ok(windows) = inventory.list_windows_including_untitled() else {
        return fallback;
    };
    if let Some(current) = windows.iter().find(|window| window.handle == fallback)
        && is_placeable_client_window(current, matcher)
    {
        return fallback;
    }
    let context = MatchContext {
        process_hierarchy: Some(inventory),
        ..Default::default()
    };
    select_window(matcher, &windows, &context)
        .map(|window| window.handle)
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{
        NeverCancelled, SharedCancellation, WaitError, find_existing_window, wait_for_window,
    };
    use crate::domain::{
        geometry::PixelBounds,
        ports::fakes::FakeWindowSystem,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };
    use std::collections::HashSet;

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
    fn accepts_a_legitimately_small_sole_window_after_launch_grace() {
        struct SmallInventory {
            window: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for SmallInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                Ok(vec![self.window.clone()])
            }
        }

        let window = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: Some("C:\\Apps\\VIVE Hub\\VHConsole.exe".to_owned()),
            process_name: Some("VHConsole.exe".to_owned()),
            title: "VIVE Hub 2.5.5".to_owned(),
            class_name: "Qt5QWindowIcon".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 320,
                height: 240,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = SmallInventory {
            window: window.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("VHConsole.exe".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            Some(10),
            None,
            6_000,
            &NeverCancelled,
        )
        .expect("small sole window is accepted");
        assert_eq!(found.title, "VIVE Hub 2.5.5");
    }

    #[test]
    fn waits_for_a_durable_window_after_a_transient_launch_window() {
        struct PhasedInventory {
            calls: std::sync::Mutex<usize>,
            transient: DesktopWindow,
            durable: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for PhasedInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                let mut calls = self.calls.lock().expect("calls");
                *calls += 1;
                if *calls < 4 {
                    Ok(vec![self.transient.clone()])
                } else {
                    Ok(vec![self.durable.clone()])
                }
            }
        }

        let transient = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: Some("C:\\Apps\\Discord.exe".to_owned()),
            process_name: Some("Discord.exe".to_owned()),
            title: "Discord Updater".to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 360,
                height: 240,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let durable = DesktopWindow {
            handle: NativeWindowHandle(2),
            process_id: 20,
            executable_path: Some("C:\\Apps\\Discord.exe".to_owned()),
            process_name: Some("Discord.exe".to_owned()),
            title: "Friends - Discord".to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = PhasedInventory {
            calls: std::sync::Mutex::new(0),
            transient,
            durable: durable.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("Discord.exe".to_owned()),
            class_name: Some("Chrome_WidgetWin_1".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            Some(20),
            None,
            5_000,
            &NeverCancelled,
        )
        .expect("durable window appears");
        assert_eq!(found.handle, durable.handle);
        assert_eq!(found.title, "Friends - Discord");
    }

    #[test]
    fn waits_for_a_durable_window_after_a_french_loading_splash() {
        struct PhasedInventory {
            calls: std::sync::Mutex<usize>,
            loading: DesktopWindow,
            durable: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for PhasedInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                self.list_windows_including_untitled()
            }

            fn list_windows_including_untitled(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                let mut calls = self.calls.lock().expect("calls");
                *calls += 1;
                if *calls < 4 {
                    Ok(vec![self.loading.clone()])
                } else {
                    Ok(vec![self.durable.clone()])
                }
            }
        }

        let loading = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: Some(
                "C:\\Apps\\Steam\\bin\\cef\\cef.win64\\steamwebhelper.exe".to_owned(),
            ),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Chargement de Steam...".to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 960,
                height: 640,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let durable = DesktopWindow {
            handle: NativeWindowHandle(2),
            process_id: 20,
            executable_path: Some(
                "C:\\Apps\\Steam\\bin\\cef\\cef.win64\\steamwebhelper.exe".to_owned(),
            ),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Steam".to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = PhasedInventory {
            calls: std::sync::Mutex::new(0),
            loading,
            durable: durable.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            Some(5),
            Some("C:\\Apps\\Steam\\steam.exe"),
            5_000,
            &NeverCancelled,
        )
        .expect("main steam window appears");
        assert_eq!(found.handle, durable.handle);
        assert_eq!(found.title, "Steam");
    }

    #[test]
    fn waits_for_the_largest_client_window_after_bootstrap_dialogs() {
        struct PhasedInventory {
            calls: std::sync::Mutex<usize>,
            login: DesktopWindow,
            main: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for PhasedInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                self.list_windows_including_untitled()
            }

            fn list_windows_including_untitled(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                let mut calls = self.calls.lock().expect("calls");
                *calls += 1;
                if *calls < 4 {
                    Ok(vec![self.login.clone()])
                } else {
                    Ok(vec![self.login.clone(), self.main.clone()])
                }
            }
        }

        let login = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 42,
            executable_path: Some("C:\\Apps\\Steam\\steamwebhelper.exe".to_owned()),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Se connecter à Steam".to_owned(),
            class_name: "SDL_app".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 705,
                height: 440,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let main = DesktopWindow {
            handle: NativeWindowHandle(2),
            process_id: 42,
            executable_path: Some("C:\\Apps\\Steam\\steamwebhelper.exe".to_owned()),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Steam".to_owned(),
            class_name: "SDL_app".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1280,
                height: 696,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = PhasedInventory {
            calls: std::sync::Mutex::new(0),
            login,
            main: main.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            Some(5),
            Some("C:\\Apps\\Steam\\steam.exe"),
            5_000,
            &NeverCancelled,
        )
        .expect("main client window appears");
        assert_eq!(found.handle, main.handle);
        assert_eq!(found.title, "Steam");
    }

    #[test]
    fn ignores_auxiliary_panels_while_waiting_for_the_main_client() {
        struct PhasedInventory {
            contacts: DesktopWindow,
            main: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for PhasedInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                self.list_windows_including_untitled()
            }

            fn list_windows_including_untitled(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                Ok(vec![self.contacts.clone(), self.main.clone()])
            }
        }

        let contacts = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 42,
            executable_path: Some("C:\\Apps\\Steam\\steamwebhelper.exe".to_owned()),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Liste de contacts".to_owned(),
            class_name: "SDL_app".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 160,
                height: 28,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let main = DesktopWindow {
            handle: NativeWindowHandle(2),
            process_id: 42,
            executable_path: Some("C:\\Apps\\Steam\\steamwebhelper.exe".to_owned()),
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: "Steam".to_owned(),
            class_name: "SDL_app".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1280,
                height: 696,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = PhasedInventory {
            contacts,
            main: main.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            None,
            None,
            2_000,
            &NeverCancelled,
        )
        .expect("main client window");
        assert_eq!(found.handle, main.handle);
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
            None,
            2_000,
            &NeverCancelled,
        )
        .expect("window appears");
        assert_eq!(found.handle, NativeWindowHandle(2));
    }

    #[test]
    fn matches_child_process_windows_after_launch() {
        struct TreeInventory {
            window: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for TreeInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                self.list_windows_including_untitled()
            }

            fn list_windows_including_untitled(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                Ok(vec![self.window.clone()])
            }

            fn is_process_in_tree(&self, process_id: u32, ancestor_id: u32) -> bool {
                process_id == 100 && ancestor_id == 50
            }
        }

        let javaw = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 100,
            executable_path: Some(
                "C:\\Users\\super\\AppData\\Roaming\\casterlabs-caffeinated\\app\\jre\\bin\\javaw.exe"
                    .to_owned(),
            ),
            process_name: Some("javaw.exe".to_owned()),
            title: "Caffeinated".to_owned(),
            class_name: "SunAwtFrame".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1280,
                height: 800,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = TreeInventory {
            window: javaw.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("Casterlabs-Caffeinated.exe".to_owned()),
            class_name: Some("SunAwtFrame".to_owned()),
            ..Default::default()
        };

        let found = wait_for_window(
            &inventory,
            &matcher,
            &HashSet::new(),
            Some(50),
            Some("C:\\Users\\super\\AppData\\Roaming\\casterlabs-caffeinated\\app\\Casterlabs-Caffeinated.exe"),
            2_000,
            &NeverCancelled,
        )
        .expect("javaw window");
        assert_eq!(found.handle, javaw.handle);
    }

    #[test]
    fn waits_for_a_new_browser_window_instead_of_reusing_an_existing_one() {
        struct PhasedInventory {
            calls: std::sync::Mutex<usize>,
            existing: DesktopWindow,
            new_window: DesktopWindow,
        }

        impl crate::domain::ports::WindowInventory for PhasedInventory {
            fn list_windows(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                self.list_windows_including_untitled()
            }

            fn list_windows_including_untitled(
                &self,
            ) -> Result<Vec<DesktopWindow>, crate::domain::ports::NativeError> {
                let mut calls = self.calls.lock().expect("calls");
                *calls += 1;
                if *calls < 3 {
                    Ok(vec![self.existing.clone()])
                } else {
                    Ok(vec![self.existing.clone(), self.new_window.clone()])
                }
            }
        }

        let existing = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 42,
            executable_path: Some("C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_owned()),
            process_name: Some("firefox.exe".to_owned()),
            title: "Old dashboard".to_owned(),
            class_name: "MozillaWindowClass".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let new_window = DesktopWindow {
            handle: NativeWindowHandle(2),
            process_id: 42,
            executable_path: Some("C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_owned()),
            process_name: Some("firefox.exe".to_owned()),
            title: "Stream manager".to_owned(),
            class_name: "MozillaWindowClass".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1600,
                height: 900,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let inventory = PhasedInventory {
            calls: std::sync::Mutex::new(0),
            existing: existing.clone(),
            new_window: new_window.clone(),
        };
        let matcher = WindowMatcher {
            process_name: Some("firefox.exe".to_owned()),
            class_name: Some("MozillaWindowClass".to_owned()),
            ..Default::default()
        };
        let previous = HashSet::from([existing.handle]);

        let found = wait_for_window(
            &inventory,
            &matcher,
            &previous,
            Some(42),
            Some("C:\\Program Files\\Mozilla Firefox\\firefox.exe"),
            5_000,
            &NeverCancelled,
        )
        .expect("new browser window");
        assert_eq!(found.handle, new_window.handle);
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
