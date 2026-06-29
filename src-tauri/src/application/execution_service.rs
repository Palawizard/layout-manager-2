use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    application::{
        concurrent_launch::{MAX_CONCURRENT_LAUNCHES, map_with_bounded_concurrency},
        execution_planner::{ExecutionPlan, PlannedLaunch},
        placement_executor::apply_planned_placement,
        unmatched_minimizer::minimize_unmatched_windows,
        window_discovery_service::{
            CancellationCheck, WaitError, find_existing_window, refresh_matched_handle,
            snapshot_handles, wait_for_window,
        },
        window_reuse::{browser_window_matcher, try_reuse_application_window},
    },
    domain::{
        execution::{
            ActionRunResult, ActionRunStatus, LayoutRunReport, RunId, RunWarning,
            aggregate_run_status,
        },
        layout::{BrowserKind, LayoutActionId},
        ports::{ProcessLaunchError, ProcessLaunchRequest, ProcessLauncher, WindowController, WindowInventory},
        window::{NativeWindowHandle},
    },
    infrastructure::browser::{WindowsBrowserLauncher, resolve_browser_executable},
    infrastructure::process::{
        is_windows_executable, launch_working_directory, recover_launch_executable,
        resolve_launch_executable,
    },
};

pub trait RunProgressListener: Send + Sync {
    fn on_started(&self, run_id: &RunId, layout_name: &str, total_actions: usize);
    fn on_action_started(&self, action_id: &LayoutActionId, label: &str, phase: &str);
    fn on_action_completed(&self, result: &ActionRunResult);
    fn on_completed(&self, report: &LayoutRunReport);
}

#[derive(Debug, Default)]
pub struct NoopRunProgressListener;

impl RunProgressListener for NoopRunProgressListener {
    fn on_started(&self, _: &RunId, _: &str, _: usize) {}
    fn on_action_started(&self, _: &LayoutActionId, _: &str, _: &str) {}
    fn on_action_completed(&self, _: &ActionRunResult) {}
    fn on_completed(&self, _: &LayoutRunReport) {}
}

pub struct ExecutionService;

struct ResolvedWindow {
    handle: NativeWindowHandle,
    reused: bool,
    relaunched: bool,
}

impl ExecutionService {
    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        plan: ExecutionPlan,
        run_id: RunId,
        inventory: &impl WindowInventory,
        controller: &impl WindowController,
        process_launcher: &impl ProcessLauncher,
        browser_launcher: &WindowsBrowserLauncher,
        cancellation: &impl CancellationCheck,
        listener: &impl RunProgressListener,
    ) -> LayoutRunReport {
        let started = Instant::now();
        let total_actions = plan.launch_steps.len();
        let warnings = plan.warnings;
        let mut results = Vec::with_capacity(total_actions);
        let mut matched_handles = HashSet::<NativeWindowHandle>::new();

        listener.on_started(&run_id, &plan.layout_name, total_actions);

        let warnings = Arc::new(Mutex::new(warnings));
        let launch_outcomes = map_with_bounded_concurrency(
            &plan.launch_steps,
            MAX_CONCURRENT_LAUNCHES,
            |step| {
                if cancellation.is_cancelled() {
                    return (
                        step.action_id().clone(),
                        step.label().to_owned(),
                        Err(skipped_result(step.action_id(), step.label())),
                    );
                }
                listener.on_action_started(step.action_id(), step.label(), "launch");
                let mut step_warnings = Vec::new();
                let outcome = Self::launch_step(
                    step,
                    inventory,
                    process_launcher,
                    browser_launcher,
                    cancellation,
                    &mut step_warnings,
                );
                if !step_warnings.is_empty() {
                    if let Ok(mut shared) = warnings.lock() {
                        shared.extend(step_warnings);
                    }
                }
                (
                    step.action_id().clone(),
                    step.label().to_owned(),
                    outcome,
                )
            },
        );

        for (step, (_, _, launch_result)) in plan.launch_steps.iter().zip(launch_outcomes) {
            let resolved = match launch_result {
                Ok(window) => window,
                Err(result) => {
                    listener.on_action_completed(&result);
                    results.push(result);
                    if !plan.options.continue_on_error {
                        break;
                    }
                    continue;
                }
            };

            listener.on_action_started(step.action_id(), step.label(), "placement");
            if resolved.relaunched {
                thread::sleep(Duration::from_millis(250));
            }
            let placement_handle = step
                .window_matcher()
                .map(|matcher| refresh_matched_handle(inventory, matcher, resolved.handle))
                .unwrap_or(resolved.handle);
            let mut placement_result = apply_planned_placement(
                step,
                inventory,
                controller,
                placement_handle,
                resolved.reused,
            );
            if resolved.relaunched && placement_result.status == ActionRunStatus::Succeeded {
                placement_result.message = Some("Application relancée.".to_owned());
            }
            listener.on_action_completed(&placement_result);
            let should_stop =
                placement_result_needs_stop(&placement_result, plan.options.continue_on_error);
            if placement_result.status == ActionRunStatus::Succeeded {
                matched_handles.insert(placement_handle);
            }
            results.push(placement_result);

            if should_stop {
                break;
            }
        }

        let warnings = Arc::try_unwrap(warnings)
            .ok()
            .and_then(|mutex| mutex.into_inner().ok())
            .unwrap_or_default();

        if !cancellation.is_cancelled()
            && plan.minimize_unmatched_windows
            && (plan.options.continue_on_error
                || results
                    .iter()
                    .all(|result| result.status != ActionRunStatus::Failed))
        {
            minimize_unmatched_windows(inventory, controller, &matched_handles);
        }

        let completed_actions = results
            .iter()
            .filter(|result| result.status == ActionRunStatus::Succeeded)
            .count();
        let report = LayoutRunReport {
            run_id,
            status: aggregate_run_status(&results, cancellation.is_cancelled()),
            duration_ms: started.elapsed().as_millis() as u64,
            completed_actions,
            total_actions,
            warnings,
            results,
        };
        listener.on_completed(&report);
        report
    }

    fn launch_step(
        step: &PlannedLaunch,
        inventory: &impl WindowInventory,
        process_launcher: &impl ProcessLauncher,
        browser_launcher: &WindowsBrowserLauncher,
        cancellation: &impl CancellationCheck,
        warnings: &mut Vec<RunWarning>,
    ) -> Result<ResolvedWindow, ActionRunResult> {
        match step {
            PlannedLaunch::ExistingWindow {
                action_id,
                label,
                window_matcher,
                executable_path,
                reopen_if_absent,
                startup_timeout_ms,
                ..
            } => match find_existing_window(inventory, window_matcher) {
                Ok(Some(window)) => Ok(ResolvedWindow {
                    handle: window.handle,
                    reused: true,
                    relaunched: false,
                }),
                Ok(None) if !reopen_if_absent => Err(failed_result(
                    action_id,
                    label,
                    "Fenêtre introuvable.".to_owned(),
                    true,
                )),
                Ok(None) => {
                    let Some(path) = executable_path else {
                        return Err(failed_result(
                            action_id,
                            label,
                            "Fenêtre introuvable et aucun exécutable enregistré pour la réouvrir."
                                .to_owned(),
                            true,
                        ));
                    };
                    let resolved_path = recover_launch_executable(
                        path,
                        window_matcher.process_name.as_deref(),
                    );
                    if !is_windows_executable(std::path::Path::new(&resolved_path)) {
                        return Err(failed_result(
                            action_id,
                            label,
                            "Impossible de relancer l’application : exécutable introuvable."
                                .to_owned(),
                            false,
                        ));
                    }
                    let previous_handles = snapshot_handles(inventory);
                    let launched = process_launcher
                        .launch(ProcessLaunchRequest {
                            executable_path: resolved_path.clone(),
                            arguments: Vec::new(),
                            working_directory: launch_working_directory(&resolved_path),
                        })
                        .map_err(|error| {
                            failed_result(action_id, label, launch_error_message(error), false)
                        })?;
                    let window = wait_for_window(
                        inventory,
                        window_matcher,
                        &previous_handles,
                        Some(launched.process_id),
                        Some(&resolved_path),
                        *startup_timeout_ms,
                        cancellation,
                    )
                    .map_err(|error| {
                        failed_result(action_id, label, wait_error_message(error), true)
                    })?;
                    Ok(ResolvedWindow {
                        handle: window.handle,
                        reused: false,
                        relaunched: true,
                    })
                }
                Err(error) => Err(failed_result(
                    action_id,
                    label,
                    wait_error_message(error),
                    true,
                )),
            },
            PlannedLaunch::Application {
                action_id,
                label,
                executable_path,
                arguments,
                working_directory,
                reuse_existing_window,
                window_matcher,
                startup_timeout_ms,
                ..
            } => {
                let reuse = try_reuse_application_window(
                    inventory,
                    window_matcher,
                    *reuse_existing_window,
                )
                .map_err(|error| failed_result(action_id, label, wait_error_message(error), true))?;
                if let Some(window) = reuse.window {
                    return Ok(ResolvedWindow {
                        handle: window.handle,
                        reused: true,
                        relaunched: false,
                    });
                }

                let resolved_path = resolve_launch_executable(executable_path);
                let previous_handles = snapshot_handles(inventory);
                let launched = process_launcher
                    .launch(ProcessLaunchRequest {
                        executable_path: resolved_path.clone(),
                        arguments: arguments.clone(),
                        working_directory: working_directory
                            .clone()
                            .or_else(|| launch_working_directory(executable_path)),
                    })
                    .map_err(|error| {
                        failed_result(action_id, label, launch_error_message(error), false)
                    })?;

                let window = wait_for_window(
                    inventory,
                    window_matcher,
                    &previous_handles,
                    Some(launched.process_id),
                    Some(&resolved_path),
                    *startup_timeout_ms,
                    cancellation,
                )
                .map_err(|error| failed_result(action_id, label, wait_error_message(error), true))?;

                Ok(ResolvedWindow {
                    handle: window.handle,
                    reused: false,
                    relaunched: false,
                })
            }
            PlannedLaunch::Browser {
                action_id,
                label,
                browser_kind,
                executable_path,
                profile,
                urls,
                startup_timeout_ms,
                ..
            } => {
                if *browser_kind == BrowserKind::SystemDefault {
                    warnings.push(RunWarning {
                        code: "default_browser_limit".to_owned(),
                        message: "Le navigateur par défaut peut ne pas ouvrir une fenêtre distincte."
                            .to_owned(),
                        action_id: Some(action_id.clone()),
                    });
                }
                let resolved_executable = resolve_browser_executable(*browser_kind, executable_path.as_deref())
                    .ok_or_else(|| {
                        failed_result(
                            action_id,
                            label,
                            "Navigateur introuvable.".to_owned(),
                            false,
                        )
                    })?;
                let matcher =
                    browser_window_matcher(*browser_kind, &resolved_executable);
                let previous_handles = snapshot_handles(inventory);
                let launched = browser_launcher
                    .launch_browser(
                        *browser_kind,
                        &resolved_executable,
                        urls,
                        profile.as_deref(),
                    )
                    .map_err(|error| {
                        failed_result(action_id, label, launch_error_message(error), false)
                    })?;
                let window = wait_for_window(
                    inventory,
                    &matcher,
                    &previous_handles,
                    Some(launched.process_id),
                    Some(&resolved_executable),
                    *startup_timeout_ms,
                    cancellation,
                )
                .map_err(|error| failed_result(action_id, label, wait_error_message(error), true))?;
                Ok(ResolvedWindow {
                    handle: window.handle,
                    reused: false,
                    relaunched: false,
                })
            }
        }
    }
}

fn placement_result_needs_stop(result: &ActionRunResult, continue_on_error: bool) -> bool {
    result.status == ActionRunStatus::Failed && !continue_on_error
}

fn skipped_result(action_id: &LayoutActionId, label: &str) -> ActionRunResult {
    ActionRunResult {
        action_id: action_id.clone(),
        label: label.to_owned(),
        status: ActionRunStatus::Skipped,
        message: Some("Action annulée.".to_owned()),
        reused_existing_window: false,
        retryable: true,
    }
}

fn failed_result(
    action_id: &LayoutActionId,
    label: &str,
    message: String,
    retryable: bool,
) -> ActionRunResult {
    ActionRunResult {
        action_id: action_id.clone(),
        label: label.to_owned(),
        status: ActionRunStatus::Failed,
        message: Some(message),
        reused_existing_window: false,
        retryable,
    }
}

fn wait_error_message(error: WaitError) -> String {
    match error {
        WaitError::Timeout => "Délai d’attente dépassé.".to_owned(),
        WaitError::Cancelled => "Action annulée.".to_owned(),
        WaitError::NotFound => "Fenêtre introuvable.".to_owned(),
        WaitError::Ambiguous => "Plusieurs fenêtres correspondent.".to_owned(),
        WaitError::InstanceNotFound { requested, available } => format!(
            "Occurrence {requested} introuvable ({available} fenêtre(s) correspondante(s))."
        ),
        WaitError::InventoryFailed => "Impossible d’inspecter les fenêtres.".to_owned(),
    }
}

fn launch_error_message(error: ProcessLaunchError) -> String {
    match error {
        ProcessLaunchError::ExecutableNotFound => "Application introuvable.".to_owned(),
        ProcessLaunchError::LaunchFailed(_) => "Lancement impossible.".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::execution::RunId;
    use super::{ExecutionService, NoopRunProgressListener};
    use crate::{
        application::{
            execution_planner::build_execution_plan,
            window_discovery_service::SharedCancellation,
        },
        domain::{
            execution::{ActionRunStatus, LayoutRunStatus},
            geometry::NormalizedBounds,
            layout::{
                Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions, WindowPlacement,
            },
            monitor::{Monitor, MonitorFallback, MonitorId, MonitorSelector},
            ports::fakes::FakeWindowSystem,
            window::{WindowMatcher, WindowState},
        },
        infrastructure::{
            browser::WindowsBrowserLauncher,
            process::WindowsProcessLauncher,
        },
    };

    fn monitor() -> Monitor {
        Monitor {
            id: MonitorId("primary".to_owned()),
            name: "primary".to_owned(),
            work_area: crate::domain::geometry::WorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040,
            },
            scale_factor: 1.0,
            is_primary: true,
        }
    }

    fn placement() -> WindowPlacement {
        WindowPlacement {
            monitor_selector: MonitorSelector {
                preferred_id: MonitorId("primary".to_owned()),
                fallback: MonitorFallback::Primary,
            },
            bounds: NormalizedBounds::new(0.0, 0.0, 1.0, 1.0).expect("bounds"),
            state: WindowState::Normal,
            center_scale: None,
        }
    }

    #[test]
    fn continues_after_a_failed_action_when_configured() {
        let layout = Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![
                LayoutAction::PlaceExistingWindow {
                    id: LayoutActionId("missing".to_owned()),
                    window_matcher: WindowMatcher {
                        process_name: Some("missing.exe".to_owned()),
                        ..Default::default()
                    },
                    placement: placement(),
                    captured_placement: None,
                    executable_path: None,
                    reopen_if_absent: false,
                    startup_timeout_ms: 15_000,
                },
                LayoutAction::PlaceExistingWindow {
                    id: LayoutActionId("present".to_owned()),
                    window_matcher: WindowMatcher {
                        process_name: Some("editor.exe".to_owned()),
                        ..Default::default()
                    },
                    placement: placement(),
                    captured_placement: None,
                    executable_path: None,
                    reopen_if_absent: false,
                    startup_timeout_ms: 15_000,
                },
            ],
            options: LayoutOptions {
                continue_on_error: true,
                ..Default::default()
            },
            created_at: 0,
            updated_at: 0,
        };
        let system = FakeWindowSystem {
            windows: vec![crate::domain::window::DesktopWindow {
                handle: crate::domain::window::NativeWindowHandle(1),
                process_id: 10,
                executable_path: Some("C:\\Apps\\Editor.exe".to_owned()),
                process_name: Some("Editor.exe".to_owned()),
                title: "Doc".to_owned(),
                class_name: "Editor".to_owned(),
                bounds: crate::domain::geometry::PixelBounds {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
                state: WindowState::Normal,
                monitor_id: None,
            }],
            ..Default::default()
        };
        let plan = build_execution_plan(&layout, &[monitor()]).expect("plan");
        let report = ExecutionService::execute(
            plan,
            RunId("run-1".to_owned()),
            &system,
            &system,
            &WindowsProcessLauncher,
            &WindowsBrowserLauncher,
            &SharedCancellation::new(),
            &NoopRunProgressListener,
        );
        assert_eq!(report.status, LayoutRunStatus::PartialFailure);
        assert_eq!(report.completed_actions, 1);
        assert_eq!(report.results[0].status, ActionRunStatus::Failed);
        assert_eq!(report.results[1].status, ActionRunStatus::Succeeded);
    }

    #[test]
    fn marks_cancelled_launch_steps_as_skipped() {
        let layout = Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![LayoutAction::PlaceExistingWindow {
                id: LayoutActionId("action-1".to_owned()),
                window_matcher: WindowMatcher {
                    process_name: Some("missing.exe".to_owned()),
                    ..Default::default()
                },
                placement: placement(),
                captured_placement: None,
                executable_path: None,
                reopen_if_absent: false,
                startup_timeout_ms: 15_000,
            }],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        };
        let cancellation = SharedCancellation::new();
        cancellation.cancel();
        let plan = build_execution_plan(&layout, &[monitor()]).expect("plan");
        let report = ExecutionService::execute(
            plan,
            RunId("run-1".to_owned()),
            &FakeWindowSystem::default(),
            &FakeWindowSystem::default(),
            &WindowsProcessLauncher,
            &WindowsBrowserLauncher,
            &cancellation,
            &NoopRunProgressListener,
        );
        assert_eq!(report.status, LayoutRunStatus::Cancelled);
        assert_eq!(report.results[0].status, ActionRunStatus::Skipped);
    }
}
