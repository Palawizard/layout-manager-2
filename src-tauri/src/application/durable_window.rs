use crate::{
    application::{
        transient_window::is_transient_window,
        window_matcher::MatchContext,
    },
    domain::window::{DesktopWindow, WindowMatcher},
};

pub const SUSPECT_LAUNCH_WIDTH: i32 = 480;
pub const SUSPECT_LAUNCH_HEIGHT: i32 = 320;
pub const MIN_CLIENT_WIDTH: i32 = 120;
pub const MIN_CLIENT_HEIGHT: i32 = 80;

#[must_use]
pub fn window_area(window: &DesktopWindow) -> i64 {
    i64::from(window.bounds.width.max(0)) * i64::from(window.bounds.height.max(0))
}

/// Strip panels and tool surfaces too small to be a main application window.
#[must_use]
pub fn is_auxiliary_window(window: &DesktopWindow) -> bool {
    window.bounds.width < MIN_CLIENT_WIDTH || window.bounds.height < MIN_CLIENT_HEIGHT
}

/// Soft signal after a launch: a new, small window may be replaced by the main window.
#[must_use]
pub fn is_suspect_launch_candidate(
    window: &DesktopWindow,
    matcher: &WindowMatcher,
    context: &MatchContext<'_>,
) -> bool {
    if matcher.title_pattern.is_some() {
        return false;
    }
    if is_transient_window(window) {
        return true;
    }
    context.launched_process_id == Some(window.process_id)
        && !context.previous_handles.contains(&window.handle)
        && (window.bounds.width < SUSPECT_LAUNCH_WIDTH
            || window.bounds.height < SUSPECT_LAUNCH_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::{is_suspect_launch_candidate, window_area};
    use crate::{
        application::{
            transient_title::has_transient_title,
            transient_window::is_transient_window,
            window_matcher::MatchContext,
        },
        domain::{
            geometry::PixelBounds,
            window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
        },
    };

    fn window(title: &str, width: i32, height: i32) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: Some("C:\\Apps\\App.exe".to_owned()),
            process_name: Some("App.exe".to_owned()),
            title: title.to_owned(),
            class_name: "AppWindow".to_owned(),
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
    fn keeps_legitimately_small_windows() {
        let hub = window("VIVE Hub 2.5.5", 320, 240);
        assert!(!has_transient_title(&hub.title));
        assert!(!is_suspect_launch_candidate(
            &hub,
            &WindowMatcher::default(),
            &MatchContext::default(),
        ));
    }

    #[test]
    fn treats_updater_titles_as_transient() {
        let updater = window("Discord Updater", 960, 640);
        assert!(has_transient_title(&updater.title));
    }

    #[test]
    fn treats_french_loading_titles_as_transient() {
        let steam = window("Chargement de Steam...", 960, 640);
        assert!(has_transient_title(&steam.title));
    }

    #[test]
    fn marks_new_small_launch_windows_as_suspect_not_excluded() {
        let splash = window("Discord", 360, 240);
        let context = MatchContext {
            launched_process_id: Some(10),
            ..Default::default()
        };
        assert!(!has_transient_title(&splash.title));
        assert!(is_suspect_launch_candidate(
            &splash,
            &WindowMatcher::default(),
            &context,
        ));
    }

    #[test]
    fn does_not_mark_existing_small_windows_as_suspect() {
        let existing = window("VIVE Hub 2.5.5", 320, 240);
        let context = MatchContext {
            previous_handles: [NativeWindowHandle(1)].into_iter().collect(),
            ..Default::default()
        };
        assert!(!is_suspect_launch_candidate(
            &existing,
            &WindowMatcher::default(),
            &context,
        ));
    }

    #[test]
    fn honors_an_explicit_title_pattern() {
        let splash = window("Discord Updater", 360, 240);
        let matcher = WindowMatcher {
            title_pattern: Some("Discord Updater".to_owned()),
            ..Default::default()
        };
        assert!(!is_suspect_launch_candidate(
            &splash,
            &matcher,
            &MatchContext::default(),
        ));
    }

    #[test]
    fn treats_bootstrap_classes_as_suspect() {
        let bootstrap = DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: None,
            process_name: Some("steam.exe".to_owned()),
            title: "Steam".to_owned(),
            class_name: "BootstrapUpdateUIClass".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 400,
                height: 129,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        assert!(is_transient_window(&bootstrap));
    }

    #[test]
    fn keeps_main_windows_when_they_are_large_enough() {
        let main = window("Friends - Discord", 1280, 800);
        assert!(!has_transient_title(&main.title));
        assert_eq!(window_area(&main), 1_024_000);
    }

    #[test]
    fn treats_contact_list_strips_as_auxiliary() {
        let contacts = window("Liste de contacts", 160, 28);
        assert!(super::is_auxiliary_window(&contacts));
        let main = window("Steam", 1280, 696);
        assert!(!super::is_auxiliary_window(&main));
    }
}
