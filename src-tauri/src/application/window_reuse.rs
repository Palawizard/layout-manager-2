use crate::domain::{
    layout::BrowserKind,
    ports::WindowInventory,
    window::{DesktopWindow, WindowMatcher},
};

use super::window_discovery_service::{WaitError, find_existing_window};

#[derive(Debug, Clone, PartialEq)]
pub struct ReuseDecision {
    pub reused: bool,
    pub window: Option<DesktopWindow>,
}

pub fn try_reuse_application_window(
    inventory: &impl WindowInventory,
    matcher: &WindowMatcher,
    reuse_existing_window: bool,
) -> Result<ReuseDecision, WaitError> {
    if !reuse_existing_window {
        return Ok(ReuseDecision {
            reused: false,
            window: None,
        });
    }
    let window = find_existing_window(inventory, matcher)?;
    Ok(ReuseDecision {
        reused: window.is_some(),
        window,
    })
}

#[must_use]
pub fn browser_window_matcher(
    kind: BrowserKind,
    executable_path: &str,
) -> WindowMatcher {
    let process_name = std::path::Path::new(executable_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned);
    let class_name = match kind {
        BrowserKind::Firefox => Some("MozillaWindowClass".to_owned()),
        BrowserKind::Edge | BrowserKind::Chrome | BrowserKind::SystemDefault => {
            Some("Chrome_WidgetWin_1".to_owned())
        }
    };
    WindowMatcher {
        executable_path: Some(executable_path.to_owned()),
        process_name,
        class_name,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{ReuseDecision, try_reuse_application_window};
    use crate::domain::{
        geometry::PixelBounds,
        ports::fakes::FakeWindowSystem,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };

    fn window() -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
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
    fn skips_reuse_when_disabled() {
        let system = FakeWindowSystem {
            windows: vec![window()],
            ..Default::default()
        };
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            try_reuse_application_window(&system, &matcher, false).expect("decision"),
            ReuseDecision {
                reused: false,
                window: None,
            }
        );
    }

    #[test]
    fn reuses_a_matching_window_when_enabled() {
        let existing = window();
        let system = FakeWindowSystem {
            windows: vec![existing.clone()],
            ..Default::default()
        };
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        let decision =
            try_reuse_application_window(&system, &matcher, true).expect("decision");
        assert!(decision.reused);
        assert_eq!(decision.window, Some(existing));
    }
}
