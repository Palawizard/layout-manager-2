use std::collections::HashSet;

use regex::Regex;

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
    if matcher.executable_path.as_ref().is_some_and(|expected| {
        window
            .executable_path
            .as_ref()
            .is_none_or(|actual| !path_eq(expected, actual))
    }) || matcher.process_name.as_ref().is_some_and(|expected| {
        window
            .process_name
            .as_ref()
            .is_none_or(|actual| !actual.eq_ignore_ascii_case(expected))
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
    score += u32::from(matcher.executable_path.is_some()) * 30;
    score += u32::from(matcher.process_name.is_some()) * 20;
    score += u32::from(matcher.class_name.is_some()) * 15;
    score += u32::from(matcher.title_pattern.is_some()) * 10;
    Some(RankedWindow { window, score })
}

fn path_eq(left: &str, right: &str) -> bool {
    left.replace('/', "\\")
        .eq_ignore_ascii_case(&right.replace('/', "\\"))
}

#[cfg(test)]
mod tests {
    use super::{MatchContext, rank_window_matches};
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
}
