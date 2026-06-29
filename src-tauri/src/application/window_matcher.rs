use std::collections::HashSet;

use regex::Regex;
use thiserror::Error;

use crate::{
    application::{
        client_window::is_non_client_window,
        durable_window::window_area,
    },
    domain::{
        ports::WindowInventory,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    },
};

#[derive(Default)]
pub struct MatchContext<'a> {
    pub launched_process_id: Option<u32>,
    pub previous_handles: HashSet<NativeWindowHandle>,
    pub process_hierarchy: Option<&'a dyn WindowInventory>,
}

impl std::fmt::Debug for MatchContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MatchContext")
            .field("launched_process_id", &self.launched_process_id)
            .field("previous_handles", &self.previous_handles)
            .field(
                "process_hierarchy",
                &self.process_hierarchy.is_some(),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RankedWindow<'a> {
    pub window: &'a DesktopWindow,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WindowMatchError {
    #[error("no window matched")]
    NotFound,
    #[error("several windows matched with the same score")]
    Ambiguous,
    #[error("requested instance index {requested} is unavailable among {available} matching window(s)")]
    InstanceNotFound {
        requested: usize,
        available: usize,
    },
}

pub fn select_window<'a>(
    matcher: &WindowMatcher,
    windows: &'a [DesktopWindow],
    context: &MatchContext<'_>,
) -> Result<&'a DesktopWindow, WindowMatchError> {
    let ranked = rank_window_matches(matcher, windows, context);
    if let Some(index) = matcher.instance_index {
        let stable = stable_instance_order(&ranked);
        return stable.get(index).copied().ok_or_else(|| {
            if ranked.is_empty() {
                WindowMatchError::NotFound
            } else {
                WindowMatchError::InstanceNotFound {
                    requested: index,
                    available: ranked.len(),
                }
            }
        });
    }
    let best = ranked.first().ok_or(WindowMatchError::NotFound)?;
    let tied: Vec<_> = ranked
        .iter()
        .take_while(|candidate| candidate.score == best.score)
        .collect();
    if tied.len() > 1 {
        let max_area = tied
            .iter()
            .map(|candidate| window_area(candidate.window))
            .max()
            .expect("non-empty tie group");
        let largest: Vec<_> = tied
            .iter()
            .filter(|candidate| window_area(candidate.window) == max_area)
            .collect();
        if largest.len() > 1 {
            return Err(WindowMatchError::Ambiguous);
        }
        return Ok(largest[0].window);
    }
    Ok(best.window)
}

#[must_use]
pub fn rank_window_matches<'a>(
    matcher: &WindowMatcher,
    windows: &'a [DesktopWindow],
    context: &MatchContext<'_>,
) -> Vec<RankedWindow<'a>> {
    let title_regex = matcher
        .title_pattern
        .as_deref()
        .and_then(|pattern| Regex::new(pattern).ok());
    let mut ranked: Vec<_> = windows
        .iter()
        .filter_map(|window| score_window(matcher, window, context, title_regex.as_ref()))
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| {
                window_area(right.window)
                    .cmp(&window_area(left.window))
            })
            .then_with(|| left.window.process_id.cmp(&right.window.process_id))
            .then_with(|| left.window.handle.0.cmp(&right.window.handle.0))
    });
    ranked
}

fn stable_instance_order<'a>(ranked: &[RankedWindow<'a>]) -> Vec<&'a DesktopWindow> {
    let mut windows: Vec<&DesktopWindow> = ranked.iter().map(|entry| entry.window).collect();
    windows.sort_by(|left, right| {
        left.process_id
            .cmp(&right.process_id)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.bounds.x.cmp(&right.bounds.x))
            .then_with(|| left.bounds.y.cmp(&right.bounds.y))
            .then_with(|| left.bounds.width.cmp(&right.bounds.width))
            .then_with(|| left.bounds.height.cmp(&right.bounds.height))
    });
    windows
}

fn score_window<'a>(
    matcher: &WindowMatcher,
    window: &'a DesktopWindow,
    context: &MatchContext<'_>,
    title_regex: Option<&Regex>,
) -> Option<RankedWindow<'a>> {
    if is_non_client_window(window) && matcher.title_pattern.is_none() {
        return None;
    }
    if matcher.process_name.as_ref().is_some_and(|expected| {
        window
            .process_name
            .as_ref()
            .is_some_and(|actual| !process_name_eq(expected, actual))
    }) || title_regex.is_some_and(|regex| !regex.is_match(&window.title))
    {
        return None;
    }
    if window.bounds.width <= 0 || window.bounds.height <= 0 {
        return None;
    }
    let mut score = 0u32;
    if let Some(launched_id) = context.launched_process_id {
        if window.process_id == launched_id {
            score += 100;
        } else if context
            .process_hierarchy
            .is_some_and(|hierarchy| hierarchy.is_process_in_tree(window.process_id, launched_id))
        {
            score += 90;
        }
    }
    if !context.previous_handles.contains(&window.handle) {
        score += 50;
    }
    if matcher
        .executable_path
        .as_ref()
        .zip(window.executable_path.as_ref())
        .is_some_and(|(expected, actual)| path_eq(expected, actual))
    {
        score += 40;
    } else if matcher
        .executable_path
        .as_ref()
        .and_then(|expected| {
            std::path::Path::new(expected)
                .file_name()
                .and_then(|name| name.to_str())
        })
        .zip(window.executable_path.as_ref())
        .is_some_and(|(expected_name, actual)| {
            std::path::Path::new(actual)
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|actual_name| path_eq(expected_name, actual_name))
        })
    {
        score += 25;
    }
    if matcher.class_name.as_ref().is_some_and(|expected| {
        window.class_name.eq_ignore_ascii_case(expected)
    }) {
        score += 15;
    }
    if !window.title.trim().is_empty() {
        score += 5;
    } else {
        score = score.saturating_sub(20);
    }
    if window.state == WindowState::Normal {
        score += 5;
    }
    score += u32::from(matcher.process_name.is_some()) * 20;
    score += u32::from(matcher.title_pattern.is_some()) * 10;
    score += ((window_area(window) / 10_000).min(100)) as u32;
    Some(RankedWindow { window, score })
}

fn path_eq(left: &str, right: &str) -> bool {
    left.replace('/', "\\")
        .eq_ignore_ascii_case(&right.replace('/', "\\"))
}

fn process_name_eq(expected: &str, actual: &str) -> bool {
    fn stem(name: &str) -> &str {
        name.strip_suffix(".exe")
            .or_else(|| name.strip_suffix(".EXE"))
            .unwrap_or(name)
    }
    stem(expected).eq_ignore_ascii_case(stem(actual))
}

#[cfg(test)]
mod tests {
    use super::{MatchContext, WindowMatchError, rank_window_matches, select_window};
    use crate::domain::{
        geometry::PixelBounds,
        window::{DesktopWindow, NativeWindowHandle, WindowMatcher, WindowState},
    };

    fn window(handle: isize, pid: u32, title: &str) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(handle),
            process_id: pid,
            executable_path: Some("C:\\Apps\\Editor.exe".to_owned()),
            process_name: Some("Editor.exe".to_owned()),
            title: title.to_owned(),
            class_name: "EditorWindow".to_owned(),
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

    fn electron_window(handle: isize, pid: u32, title: &str, process_name: &str, path: &str) -> DesktopWindow {
        DesktopWindow {
            handle: NativeWindowHandle(handle),
            process_id: pid,
            executable_path: Some(path.to_owned()),
            process_name: Some(process_name.to_owned()),
            title: title.to_owned(),
            class_name: "Chrome_WidgetWin_1".to_owned(),
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
    fn prefers_the_launched_process() {
        let windows = [window(1, 10, "Old"), window(2, 20, "New")];
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };
        let context = MatchContext {
            launched_process_id: Some(20),
            ..Default::default()
        };

        assert_eq!(
            rank_window_matches(&matcher, &windows, &context)[0]
                .window
                .process_id,
            20
        );
    }

    #[test]
    fn excludes_windows_that_fail_a_required_criterion() {
        let windows = [window(1, 10, "Document")];
        let matcher = WindowMatcher {
            title_pattern: Some("Settings".to_owned()),
            ..Default::default()
        };

        assert!(rank_window_matches(&matcher, &windows, &MatchContext::default()).is_empty());
    }

    #[test]
    fn reports_an_ambiguous_best_match() {
        let windows = [window(1, 10, "One"), window(2, 20, "Two")];
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default()),
            Err(WindowMatchError::Ambiguous)
        );
    }

    #[test]
    fn matches_when_executable_path_changed_after_an_update() {
        let windows = [electron_window(
            1,
            10,
            "Discord",
            "Discord.exe",
            "C:\\Apps\\Discord\\app-2.0.0\\Discord.exe",
        )];
        let matcher = WindowMatcher {
            executable_path: Some("C:\\Apps\\Discord\\app-1.0.0\\Discord.exe".to_owned()),
            process_name: Some("Discord.exe".to_owned()),
            class_name: Some("Chrome_WidgetWin_1".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default())
                .expect("process name still matches")
                .process_id,
            10
        );
    }

    #[test]
    fn compares_process_names_without_the_exe_suffix() {
        let windows = [electron_window(
            1,
            10,
            "Vesktop",
            "Vesktop.exe",
            "C:\\Apps\\Vesktop\\Vesktop.exe",
        )];
        let matcher = WindowMatcher {
            process_name: Some("vesktop".to_owned()),
            class_name: Some("Chrome_WidgetWin_1".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default())
                .expect("stem comparison")
                .process_id,
            10
        );
    }

    #[test]
    fn reports_when_the_requested_instance_is_unavailable() {
        let windows = [window(1, 10, "Only")];
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            instance_index: Some(1),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default()),
            Err(WindowMatchError::InstanceNotFound {
                requested: 1,
                available: 1,
            })
        );
    }

    #[test]
    fn prefers_a_larger_durable_window_over_a_transient_launch_window() {
        let windows = [
            DesktopWindow {
                bounds: PixelBounds {
                    x: 0,
                    y: 0,
                    width: 360,
                    height: 240,
                },
                ..electron_window(
                    1,
                    10,
                    "Discord Updater",
                    "Discord.exe",
                    "C:\\Apps\\Discord\\Discord.exe",
                )
            },
            electron_window(
                2,
                20,
                "Friends - Discord",
                "Discord.exe",
                "C:\\Apps\\Discord\\Discord.exe",
            ),
        ];
        let matcher = WindowMatcher {
            process_name: Some("Discord.exe".to_owned()),
            class_name: Some("Chrome_WidgetWin_1".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default())
                .expect("main window")
                .title,
            "Friends - Discord"
        );
    }

    #[test]
    fn ignores_mangled_contact_list_panels() {
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
                width: 1920,
                height: 1080,
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
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &[contacts, main], &MatchContext::default())
                .expect("main window")
                .title,
            "Steam"
        );
    }

    #[test]
    fn ignores_auxiliary_panels_when_matching_steam() {
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
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &[contacts, main], &MatchContext::default())
                .expect("main window")
                .title,
            "Steam"
        );
    }

    #[test]
    fn prefers_the_largest_matching_window_when_scores_are_tied() {
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
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &[login, main], &MatchContext::default())
                .expect("main window")
                .handle,
            NativeWindowHandle(2)
        );
    }

    #[test]
    fn uses_the_requested_instance_index() {
        let windows = [window(1, 10, "One"), window(2, 20, "Two")];
        let matcher = WindowMatcher {
            process_name: Some("editor.exe".to_owned()),
            instance_index: Some(1),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &windows, &MatchContext::default())
                .expect("second instance")
                .process_id,
            20
        );
    }

    #[test]
    fn selects_the_expected_instance_among_same_process_windows() {
        let helper = DesktopWindow {
            handle: NativeWindowHandle(100),
            process_id: 42,
            executable_path: Some("C:\\Apps\\VIVE Hub\\VHConsole\\VHConsole.exe".to_owned()),
            process_name: Some("VHConsole.exe".to_owned()),
            title: String::new(),
            class_name: "Qt5QWindowIcon".to_owned(),
            bounds: PixelBounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            state: WindowState::Normal,
            monitor_id: None,
        };
        let main = DesktopWindow {
            handle: NativeWindowHandle(200),
            process_id: 42,
            executable_path: Some("C:\\Apps\\VIVE Hub\\VHConsole\\VHConsole.exe".to_owned()),
            process_name: Some("VHConsole.exe".to_owned()),
            title: "VIVE Hub".to_owned(),
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
        let matcher = WindowMatcher {
            process_name: Some("VHConsole.exe".to_owned()),
            class_name: Some("Qt5QWindowIcon".to_owned()),
            instance_index: Some(1),
            ..Default::default()
        };

        assert_eq!(
            select_window(&matcher, &[helper, main], &MatchContext::default())
                .expect("second stable instance")
                .title,
            "VIVE Hub"
        );
    }
}
