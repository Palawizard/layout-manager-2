use crate::{application::transient_title::has_transient_title, domain::window::DesktopWindow};

const TRANSIENT_WINDOW_CLASSES: &[&str] = &[
    "BootstrapUpdateUIClass",
    "SplashScreenClass",
    "MsoSplash",
    "NUIDialog",
];

/// Hard exclusion for splash, bootstrap and updater shells.
#[must_use]
pub fn is_transient_window(window: &DesktopWindow) -> bool {
    has_transient_title(&window.title) || has_transient_class(&window.class_name)
}

#[must_use]
pub fn has_transient_class(class_name: &str) -> bool {
    let normalized = class_name.trim().to_ascii_lowercase();
    TRANSIENT_WINDOW_CLASSES
        .iter()
        .any(|class| normalized == class.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{has_transient_class, is_transient_window};
    use crate::domain::{
        geometry::PixelBounds,
        window::{DesktopWindow, NativeWindowHandle, WindowState},
    };

    fn window(title: &str, class_name: &str) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 10,
            executable_path: None,
            process_name: Some("steam.exe".to_owned()),
            title: title.to_owned(),
            class_name: class_name.to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 400,
                height: 129,
            },
            state: WindowState::Normal,
            monitor_id: None,
        }
    }

    #[test]
    fn treats_bootstrap_classes_as_transient() {
        assert!(has_transient_class("BootstrapUpdateUIClass"));
        assert!(is_transient_window(&window(
            "Steam",
            "BootstrapUpdateUIClass",
        )));
    }

    #[test]
    fn keeps_regular_client_windows() {
        assert!(!is_transient_window(&window("Steam", "SDL_app")));
    }
}
