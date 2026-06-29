use std::collections::HashSet;

use regex::Regex;
use thiserror::Error;

use crate::domain::window::{DesktopWindow, NativeWindowHandle, WindowMatcher};

#[derive(Debug, Default)]
pub struct MatchContext {
    pub launched_process_id: Option<u32>,
    pub previous_handles: HashSet<NativeWindowHandle>,
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
    context: &MatchContext,
) -> Result<&'a DesktopWindow, WindowMatchError> {
    let ranked = rank_window_matches(matcher, windows, context);
    if let Some(index) = matcher.instance_index {
        return ranked.get(index).map(|candidate| candidate.window).ok_or_else(|| {
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
    if ranked
        .get(1)
        .is_some_and(|second| second.score == best.score)
    {
        return Err(WindowMatchError::Ambiguous);
    }
    Ok(best.window)
}

#[must_use]
pub fn rank_window_matches<'a>(
    matcher: &WindowMatcher,
    windows: &'a [DesktopWindow],
    context: &MatchContext,
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
            .then_with(|| left.window.process_id.cmp(&right.window.process_id))
    });
    ranked
}

fn score_window<'a>(
    matcher: &WindowMatcher,
    window: &'a DesktopWindow,
    context: &MatchContext,
    title_regex: Option<&Regex>,
) -> Option<RankedWindow<'a>> {
    if matcher.process_name.as_ref().is_some_and(|expected| {
        window
            .process_name
            .as_ref()
            .is_some_and(|actual| !process_name_eq(expected, actual))
    }) || matcher
        .class_name
        .as_ref()
        .is_some_and(|expected| !window.class_name.eq_ignore_ascii_case(expected))
        || title_regex.is_some_and(|regex| !regex.is_match(&window.title))
    {
        return None;
    }
    let mut score = 0;
    if context.launched_process_id == Some(window.process_id) {
        score += 100;
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
    }
    score += u32::from(matcher.process_name.is_some()) * 20;
    score += u32::from(matcher.class_name.is_some()) * 15;
    score += u32::from(matcher.title_pattern.is_some()) * 10;
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
}
