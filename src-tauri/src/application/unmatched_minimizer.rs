use std::collections::HashSet;

use crate::{
    application::client_window::is_non_client_window,
    domain::{
        ports::WindowController,
        window::{NativeWindowHandle, WindowState},
    },
};

pub fn minimize_unmatched_windows(
    inventory: &impl crate::domain::ports::WindowInventory,
    controller: &impl WindowController,
    matched_handles: &HashSet<NativeWindowHandle>,
) {
    let Ok(windows) = inventory.list_windows() else {
        return;
    };
    for window in windows {
        if matched_handles.contains(&window.handle)
            || window.state == WindowState::Minimized
            || is_non_client_window(&window)
        {
            continue;
        }
        let _ = controller.set_window_state(window.handle, WindowState::Minimized);
    }
}

#[cfg(test)]
mod tests {
    use super::minimize_unmatched_windows;
    use crate::domain::{
        geometry::PixelBounds,
        ports::fakes::FakeWindowSystem,
        window::{DesktopWindow, NativeWindowHandle, WindowState},
    };
    use std::collections::HashSet;

    #[test]
    fn minimizes_only_unmatched_visible_windows() {
        let system = FakeWindowSystem {
            windows: vec![
                DesktopWindow {
                    handle: NativeWindowHandle(1),
                    process_id: 1,
                    executable_path: None,
                    process_name: Some("one.exe".to_owned()),
                    title: "One".to_owned(),
                    class_name: "One".to_owned(),
                    bounds: PixelBounds {
                        x: 0,
                        y: 0,
                        width: 200,
                        height: 200,
                    },
                    state: WindowState::Normal,
                    monitor_id: None,
                },
                DesktopWindow {
                    handle: NativeWindowHandle(2),
                    process_id: 2,
                    executable_path: None,
                    process_name: Some("two.exe".to_owned()),
                    title: "Two".to_owned(),
                    class_name: "Two".to_owned(),
                    bounds: PixelBounds {
                        x: 0,
                        y: 0,
                        width: 200,
                        height: 200,
                    },
                    state: WindowState::Normal,
                    monitor_id: None,
                },
            ],
            ..Default::default()
        };
        minimize_unmatched_windows(&system, &system, &HashSet::from([NativeWindowHandle(1)]));
        let states = system.states.lock().expect("states");
        assert_eq!(states.len(), 1);
        assert_eq!(states[0], (NativeWindowHandle(2), WindowState::Minimized));
    }

    #[test]
    fn skips_ancillary_panels_when_minimizing_unmatched_windows() {
        let system = FakeWindowSystem {
            windows: vec![
                DesktopWindow {
                    handle: NativeWindowHandle(1),
                    process_id: 1,
                    executable_path: None,
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
                },
                DesktopWindow {
                    handle: NativeWindowHandle(2),
                    process_id: 1,
                    executable_path: None,
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
                },
            ],
            ..Default::default()
        };
        minimize_unmatched_windows(&system, &system, &HashSet::from([NativeWindowHandle(1)]));
        let states = system.states.lock().expect("states");
        assert!(states.is_empty());
    }
}
