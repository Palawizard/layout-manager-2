use thiserror::Error;

use crate::domain::{
    execution::RunWarning,
    geometry::{PixelBounds, WorkArea},
    layout::{Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions},
    monitor::{Monitor, MonitorId, MonitorSelection},
    window::{WindowMatcher, WindowState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPhase {
    Launch,
    Placement,
    MinimizeUnmatched,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPlacement {
    pub monitor_id: MonitorId,
    pub work_area: WorkArea,
    pub bounds: PixelBounds,
    pub state: WindowState,
    pub used_monitor_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedLaunch {
    Application {
        action_id: LayoutActionId,
        label: String,
        executable_path: String,
        arguments: Vec<String>,
        working_directory: Option<String>,
        reuse_existing_window: bool,
        window_matcher: WindowMatcher,
        placement: ResolvedPlacement,
        startup_timeout_ms: u32,
    },
    ExistingWindow {
        action_id: LayoutActionId,
        label: String,
        window_matcher: WindowMatcher,
        executable_path: Option<String>,
        reopen_if_absent: bool,
        startup_timeout_ms: u32,
        placement: ResolvedPlacement,
    },
    Browser {
        action_id: LayoutActionId,
        label: String,
        browser_kind: crate::domain::layout::BrowserKind,
        executable_path: Option<String>,
        profile: Option<String>,
        urls: Vec<String>,
        placement: ResolvedPlacement,
        startup_timeout_ms: u32,
    },
}

impl PlannedLaunch {
    #[must_use]
    pub fn action_id(&self) -> &LayoutActionId {
        match self {
            Self::Application { action_id, .. }
            | Self::ExistingWindow { action_id, .. }
            | Self::Browser { action_id, .. } => action_id,
        }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::Application { label, .. }
            | Self::ExistingWindow { label, .. }
            | Self::Browser { label, .. } => label,
        }
    }

    #[must_use]
    pub fn placement(&self) -> &ResolvedPlacement {
        match self {
            Self::Application { placement, .. }
            | Self::ExistingWindow { placement, .. }
            | Self::Browser { placement, .. } => placement,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub layout_id: LayoutId,
    pub layout_name: String,
    pub options: LayoutOptions,
    pub launch_steps: Vec<PlannedLaunch>,
    pub warnings: Vec<RunWarning>,
    pub minimize_unmatched_windows: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
    #[error("layout validation failed: {0}")]
    Validation(String),
    #[error("no monitor available")]
    NoMonitor,
}

pub fn build_execution_plan(layout: &Layout, monitors: &[Monitor]) -> Result<ExecutionPlan, PlanError> {
    layout
        .validate(true)
        .map_err(|error| PlanError::Validation(error.to_string()))?;
    if monitors.is_empty() {
        return Err(PlanError::NoMonitor);
    }

    let mut warnings = Vec::new();
    let mut launch_steps = Vec::with_capacity(layout.actions.len());

    for action in &layout.actions {
        let (action_id, label, placement_selector, bounds, state) = match action {
            LayoutAction::LaunchApplication {
                id,
                executable_path,
                placement,
                ..
            } => (
                id.clone(),
                human_label_from_path(executable_path),
                placement.monitor_selector.clone(),
                placement.bounds,
                placement.state,
            ),
            LayoutAction::PlaceExistingWindow {
                id,
                window_matcher,
                placement,
                ..
            } => (
                id.clone(),
                human_label_from_matcher(window_matcher),
                placement.monitor_selector.clone(),
                placement.bounds,
                placement.state,
            ),
            LayoutAction::OpenBrowserWindow {
                id,
                browser_kind,
                placement,
                ..
            } => (
                id.clone(),
                human_label_from_browser(*browser_kind),
                placement.monitor_selector.clone(),
                placement.bounds,
                placement.state,
            ),
        };

        let resolved = resolve_placement(
            &placement_selector,
            monitors,
            bounds,
            state,
            &action_id,
            &mut warnings,
        )?;

        let step = match action {
            LayoutAction::LaunchApplication {
                executable_path,
                arguments,
                working_directory,
                reuse_existing_window,
                window_matcher,
                startup_timeout_ms,
                ..
            } => PlannedLaunch::Application {
                action_id,
                label,
                executable_path: executable_path.clone(),
                arguments: arguments.clone(),
                working_directory: working_directory.clone(),
                reuse_existing_window: *reuse_existing_window,
                window_matcher: window_matcher.clone(),
                placement: resolved,
                startup_timeout_ms: *startup_timeout_ms,
            },
            LayoutAction::PlaceExistingWindow {
                window_matcher,
                executable_path,
                reopen_if_absent,
                startup_timeout_ms,
                ..
            } => PlannedLaunch::ExistingWindow {
                action_id,
                label,
                window_matcher: window_matcher.clone(),
                executable_path: executable_path.clone(),
                reopen_if_absent: *reopen_if_absent,
                startup_timeout_ms: *startup_timeout_ms,
                placement: resolved,
            },
            LayoutAction::OpenBrowserWindow {
                browser_kind,
                executable_path,
                profile,
                urls,
                startup_timeout_ms,
                ..
            } => PlannedLaunch::Browser {
                action_id,
                label,
                browser_kind: *browser_kind,
                executable_path: executable_path.clone(),
                profile: profile.clone(),
                urls: urls.clone(),
                placement: resolved,
                startup_timeout_ms: *startup_timeout_ms,
            },
        };
        launch_steps.push(step);
    }

    Ok(ExecutionPlan {
        layout_id: layout.id.clone(),
        layout_name: layout.name.clone(),
        options: layout.options.clone(),
        launch_steps,
        warnings,
        minimize_unmatched_windows: layout.options.minimize_unmatched_windows,
    })
}

fn resolve_placement(
    selector: &crate::domain::monitor::MonitorSelector,
    monitors: &[Monitor],
    bounds: crate::domain::geometry::NormalizedBounds,
    state: WindowState,
    action_id: &LayoutActionId,
    warnings: &mut Vec<RunWarning>,
) -> Result<ResolvedPlacement, PlanError> {
    let MonitorSelection {
        monitor,
        used_fallback,
    } = selector
        .resolve(monitors)
        .ok_or(PlanError::NoMonitor)?;
    if used_fallback {
        warnings.push(RunWarning {
            code: "monitor_fallback".to_owned(),
            message: "L’écran prévu est absent. Un autre écran a été utilisé.".to_owned(),
            action_id: Some(action_id.clone()),
        });
    }
    Ok(ResolvedPlacement {
        monitor_id: monitor.id.clone(),
        work_area: monitor.work_area,
        bounds: bounds.to_pixels(monitor.work_area),
        state,
        used_monitor_fallback: used_fallback,
    })
}

fn human_label_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Application")
        .to_owned()
}

fn human_label_from_matcher(matcher: &WindowMatcher) -> String {
    matcher
        .process_name
        .as_deref()
        .or(matcher.executable_path.as_deref().and_then(|path| {
            std::path::Path::new(path).file_stem().and_then(|name| name.to_str())
        }))
        .unwrap_or("Fenêtre")
        .trim_end_matches(".exe")
        .to_owned()
}

fn human_label_from_browser(kind: crate::domain::layout::BrowserKind) -> String {
    match kind {
        crate::domain::layout::BrowserKind::Edge => "Microsoft Edge".to_owned(),
        crate::domain::layout::BrowserKind::Chrome => "Google Chrome".to_owned(),
        crate::domain::layout::BrowserKind::Firefox => "Mozilla Firefox".to_owned(),
        crate::domain::layout::BrowserKind::SystemDefault => "Navigateur".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionPhase, build_execution_plan};
    use crate::domain::{
        geometry::NormalizedBounds,
        layout::{
            BrowserKind, Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions,
            WindowPlacement,
        },
        monitor::{Monitor, MonitorFallback, MonitorId, MonitorSelector},
        window::{WindowMatcher, WindowState},
    };

    fn sample_monitor(id: &str) -> Monitor {
        Monitor {
            id: MonitorId(id.to_owned()),
            name: id.to_owned(),
            work_area: crate::domain::geometry::WorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040,
            },
            scale_factor: 1.0,
            is_primary: id == "primary",
        }
    }

    fn sample_placement() -> WindowPlacement {
        WindowPlacement {
            monitor_selector: MonitorSelector {
                preferred_id: MonitorId("primary".to_owned()),
                fallback: MonitorFallback::Primary,
            },
            bounds: NormalizedBounds::new(0.0, 0.0, 0.5, 1.0).expect("bounds"),
            state: WindowState::Normal,
            center_scale: None,
        }
    }

    #[test]
    fn builds_a_plan_for_a_valid_layout() {
        let layout = Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![LayoutAction::PlaceExistingWindow {
                id: LayoutActionId("action-1".to_owned()),
                window_matcher: WindowMatcher {
                    process_name: Some("notepad.exe".to_owned()),
                    ..Default::default()
                },
                placement: sample_placement(),
                executable_path: Some("C:\\Windows\\System32\\notepad.exe".to_owned()),
                reopen_if_absent: true,
                startup_timeout_ms: 15_000,
            }],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        };

        let plan = build_execution_plan(&layout, &[sample_monitor("primary")]).expect("plan");
        assert_eq!(plan.launch_steps.len(), 1);
        assert!(!plan.minimize_unmatched_windows);
    }

    #[test]
    fn warns_when_the_preferred_monitor_is_missing() {
        let layout = Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![LayoutAction::OpenBrowserWindow {
                id: LayoutActionId("action-1".to_owned()),
                browser_kind: BrowserKind::Edge,
                executable_path: None,
                profile: None,
                urls: vec!["https://example.com".to_owned()],
                placement: WindowPlacement {
                    monitor_selector: MonitorSelector {
                        preferred_id: MonitorId("missing".to_owned()),
                        fallback: MonitorFallback::Primary,
                    },
                    bounds: NormalizedBounds::new(0.0, 0.0, 1.0, 1.0).expect("bounds"),
                    state: WindowState::Normal,
                    center_scale: None,
                },
                startup_timeout_ms: 10_000,
            }],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        };

        let plan = build_execution_plan(&layout, &[sample_monitor("primary")]).expect("plan");
        assert_eq!(plan.warnings.len(), 1);
        assert!(plan.launch_steps[0].placement().used_monitor_fallback);
    }

    #[test]
    fn rejects_a_layout_without_actions() {
        let layout = Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        };

        assert!(build_execution_plan(&layout, &[sample_monitor("primary")]).is_err());
    }

    #[test]
    fn execution_phases_are_ordered() {
        assert!((ExecutionPhase::Launch as u8) < (ExecutionPhase::Placement as u8));
        assert!((ExecutionPhase::Placement as u8) < (ExecutionPhase::MinimizeUnmatched as u8));
    }
}
