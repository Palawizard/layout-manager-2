use crate::{
    application::{
        ancillary_panel_title::has_ancillary_panel_title, durable_window::is_auxiliary_window,
        transient_window::is_transient_window,
    },
    domain::window::{DesktopWindow, WindowMatcher},
};

/// Windows that should never be auto-selected, placed, or minimized as the main client.
#[must_use]
pub fn is_non_client_window(window: &DesktopWindow) -> bool {
    is_transient_window(window)
        || is_auxiliary_window(window)
        || has_ancillary_panel_title(&window.title)
}

/// Whether a matched window is safe to place with the current matcher.
#[must_use]
pub fn is_placeable_client_window(window: &DesktopWindow, matcher: &WindowMatcher) -> bool {
    if matcher.title_pattern.is_some() {
        return true;
    }
    !is_non_client_window(window)
}

#[cfg(test)]
mod tests {
    use super::{is_non_client_window, is_placeable_client_window};
    use crate::domain::{
        geometry::PixelBounds,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };

    fn window(title: &str, width: i32, height: i32) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(1),
            process_id: 42,
            executable_path: None,
            process_name: Some("steamwebhelper.exe".to_owned()),
            title: title.to_owned(),
            class_name: "SDL_app".to_owned(),
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
    fn rejects_contact_list_even_after_a_previous_resize() {
        let mangled = window("Liste de contacts", 1920, 1080);
        assert!(is_non_client_window(&mangled));
        assert!(!is_placeable_client_window(
            &mangled,
            &WindowMatcher::default(),
        ));
    }

    #[test]
    fn honors_an_explicit_title_pattern() {
        let panel = window("Liste de contacts", 1920, 1080);
        let matcher = WindowMatcher {
            title_pattern: Some("^Liste de contacts$".to_owned()),
            ..Default::default()
        };
        assert!(is_placeable_client_window(&panel, &matcher));
    }
}
