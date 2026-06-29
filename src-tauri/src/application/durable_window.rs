use crate::domain::window::{DesktopWindow, WindowMatcher};

pub const MIN_DURABLE_WIDTH: i32 = 480;
pub const MIN_DURABLE_HEIGHT: i32 = 320;

const TRANSIENT_TITLE_KEYWORDS: &[&str] = &[
    "updater",
    "updating",
    "update available",
    "installing",
    "installation",
    "splash",
    "loading",
    "please wait",
    "setup",
    "patching",
    "maintenance",
    "checking for updates",
];

#[must_use]
pub fn window_area(window: &DesktopWindow) -> i64 {
    i64::from(window.bounds.width.max(0)) * i64::from(window.bounds.height.max(0))
}

#[must_use]
pub fn is_transient_launch_window(window: &DesktopWindow, matcher: &WindowMatcher) -> bool {
    if matcher.title_pattern.is_some() {
        return false;
    }
    if window.bounds.width < MIN_DURABLE_WIDTH || window.bounds.height < MIN_DURABLE_HEIGHT {
        return true;
    }
    transient_title(&window.title)
}

fn transient_title(title: &str) -> bool {
    let normalized = title.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }
    TRANSIENT_TITLE_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

#[cfg(test)]
mod tests {
    use super::{is_transient_launch_window, window_area};
    use crate::domain::{
        geometry::PixelBounds,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };

    fn window(title: &str, width: i32, height: i32) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: Some("C:\\Apps\\Discord.exe".to_owned()),
            process_name: Some("Discord.exe".to_owned()),
            title: title.to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width,
                height,
            },
            state: WindowState::Normal,
            monitor_id: None,
        }
    }

    #[test]
    fn treats_small_launch_windows_as_transient() {
        let updater = window("Discord", 360, 240);
        assert!(is_transient_launch_window(&updater, &WindowMatcher::default()));
    }

    #[test]
    fn treats_updater_titles_as_transient() {
        let updater = window("Discord Updater", 960, 640);
        assert!(is_transient_launch_window(&updater, &WindowMatcher::default()));
    }

    #[test]
    fn keeps_main_windows_when_they_are_large_enough() {
        let main = window("Friends - Discord", 1280, 800);
        assert!(!is_transient_launch_window(&main, &WindowMatcher::default()));
        assert_eq!(window_area(&main), 1_024_000);
    }

    #[test]
    fn honors_an_explicit_title_pattern() {
        let splash = window("Discord Updater", 360, 240);
        let matcher = WindowMatcher {
            title_pattern: Some("Discord Updater".to_owned()),
            ..Default::default()
        };
        assert!(!is_transient_launch_window(&splash, &matcher));
    }
}
