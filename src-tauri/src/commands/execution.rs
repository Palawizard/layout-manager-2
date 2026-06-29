use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::{
    application::{
        execution_planner::{ExecutionPlan, build_execution_plan},
        execution_service::{ExecutionService, RunProgressListener},
        layout_service::LayoutService,
        window_discovery_service::SharedCancellation,
    },
    domain::{
        execution::{ActionRunResult, LayoutRunReport, RunId},
        layout::{Layout, LayoutActionId, LayoutId},
        ports::MonitorProvider,
    },
    error::PublicError,
    infrastructure::{
        browser::{InstalledBrowser, WindowsBrowserLauncher, detect_installed_browsers},
        persistence::{Database, SqliteLayoutRepository},
        process::WindowsProcessLauncher,
        windows::Win32WindowSystem,
    },
};

const EVENT_STARTED: &str = "layout-run://started";
const EVENT_ACTION_STARTED: &str = "layout-run://action-started";
const EVENT_ACTION_COMPLETED: &str = "layout-run://action-completed";
const EVENT_COMPLETED: &str = "layout-run://completed";

#[derive(Debug, Default)]
pub struct ExecutionRuntime {
    pub(crate) inner: Mutex<ExecutionRuntimeState>,
}

#[derive(Debug, Default)]
pub(crate) struct ExecutionRuntimeState {
    active: bool,
    run_generation: u64,
    cancellation: Option<Arc<SharedCancellation>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunStartedEvent {
    run_id: String,
    layout_id: String,
    layout_name: String,
    total_actions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionStartedEvent {
    run_id: String,
    action_id: String,
    label: String,
    phase: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionCompletedEvent {
    run_id: String,
    result: ActionRunResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunCompletedEvent {
    report: LayoutRunReport,
}

struct TauriRunProgressListener {
    app: AppHandle,
    run_id: RunId,
    layout_id: LayoutId,
}

impl RunProgressListener for TauriRunProgressListener {
    fn on_started(&self, run_id: &RunId, layout_name: &str, total_actions: usize) {
        let _ = self.app.emit(
            EVENT_STARTED,
            RunStartedEvent {
                run_id: run_id.0.clone(),
                layout_id: self.layout_id.0.clone(),
                layout_name: layout_name.to_owned(),
                total_actions,
            },
        );
    }

    fn on_action_started(&self, action_id: &LayoutActionId, label: &str, phase: &str) {
        let _ = self.app.emit(
            EVENT_ACTION_STARTED,
            ActionStartedEvent {
                run_id: self.run_id.0.clone(),
                action_id: action_id.0.clone(),
                label: label.to_owned(),
                phase: phase.to_owned(),
            },
        );
    }

    fn on_action_completed(&self, result: &ActionRunResult) {
        let _ = self.app.emit(
            EVENT_ACTION_COMPLETED,
            ActionCompletedEvent {
                run_id: self.run_id.0.clone(),
                result: result.clone(),
            },
        );
    }

    fn on_completed(&self, report: &LayoutRunReport) {
        let _ = self
            .app
            .emit(EVENT_COMPLETED, RunCompletedEvent { report: report.clone() });
    }
}

#[tauri::command]
pub fn list_installed_browsers() -> Vec<InstalledBrowser> {
    detect_installed_browsers()
}

#[tauri::command]
pub fn run_layout(
    app: AppHandle,
    database: State<'_, Database>,
    windows: State<'_, Win32WindowSystem>,
    runtime: State<'_, ExecutionRuntime>,
    layout_id: String,
    action_ids: Option<Vec<String>>,
) -> Result<RunId, PublicError> {
    let run_generation = {
        let mut state = runtime.inner.lock().expect("execution runtime lock");
        if state.active {
            return Err(PublicError {
                code: "run_already_active",
                message: "Un layout est déjà en cours d’exécution.".to_owned(),
                field: None,
                retryable: true,
            });
        }
        state.run_generation += 1;
        state.active = true;
        state.cancellation = Some(Arc::new(SharedCancellation::new()));
        state.run_generation
    };

    let plan = match prepare_execution_plan(&database, &windows, &layout_id, action_ids) {
        Ok(plan) => plan,
        Err(error) => {
            tracing::warn!(?error, layout_id, "layout run preparation failed");
            release_run_slot(&runtime, run_generation);
            return Err(error);
        }
    };

    let cancellation = {
        let state = runtime.inner.lock().expect("execution runtime lock");
        state.cancellation.clone().expect("cancellation token")
    };
    let run_id = RunId(Uuid::new_v4().to_string());
    let listener = TauriRunProgressListener {
        app: app.clone(),
        run_id: run_id.clone(),
        layout_id: LayoutId(layout_id),
    };
    let thread_run_id = run_id.clone();

    std::thread::spawn(move || {
        let windows = Win32WindowSystem::new();
        let process_launcher = WindowsProcessLauncher;
        let browser_launcher = WindowsBrowserLauncher;
        let _report = ExecutionService::execute(
            plan,
            thread_run_id,
            &windows,
            &windows,
            &process_launcher,
            &browser_launcher,
            &cancellation,
            &listener,
        );
        if let Some(runtime) = app.try_state::<ExecutionRuntime>() {
            release_run_slot(&runtime, run_generation);
        }
    });

    Ok(run_id)
}

#[tauri::command]
pub fn cancel_layout_run(runtime: State<'_, ExecutionRuntime>) -> Result<(), PublicError> {
    let mut state = runtime.inner.lock().expect("execution runtime lock");
    if let Some(cancellation) = &state.cancellation {
        cancellation.cancel();
        state.active = false;
        state.cancellation = None;
        Ok(())
    } else {
        Err(PublicError {
            code: "no_active_run",
            message: "Aucune exécution en cours.".to_owned(),
            field: None,
            retryable: false,
        })
    }
}

fn prepare_execution_plan(
    database: &Database,
    windows: &Win32WindowSystem,
    layout_id: &str,
    action_ids: Option<Vec<String>>,
) -> Result<ExecutionPlan, PublicError> {
    let layout = LayoutService::new(SqliteLayoutRepository::new(database))
        .get(&LayoutId(layout_id.to_owned()))
        .map_err(PublicError::from)?;
    let layout = filter_layout_actions(layout, action_ids)?;
    let monitors = windows.list_monitors().map_err(PublicError::from)?;
    build_execution_plan(&layout, &monitors).map_err(plan_error)
}

fn release_run_slot(runtime: &ExecutionRuntime, run_generation: u64) {
    let mut state = runtime.inner.lock().expect("execution runtime lock");
    if state.run_generation == run_generation {
        state.active = false;
        state.cancellation = None;
    }
}

fn filter_layout_actions(
    mut layout: Layout,
    action_ids: Option<Vec<String>>,
) -> Result<Layout, PublicError> {
    let Some(ids) = action_ids else {
        return Ok(layout);
    };
    if ids.is_empty() {
        return Err(PublicError {
            code: "validation_failed",
            message: "Sélectionnez au moins une action à relancer.".to_owned(),
            field: None,
            retryable: false,
        });
    }
    layout.actions.retain(|action| ids.contains(&action.id().0));
    layout.validate(true).map_err(PublicError::from)?;
    Ok(layout)
}

fn plan_error(error: crate::application::execution_planner::PlanError) -> PublicError {
    use crate::application::execution_planner::PlanError;
    let message = match error {
        PlanError::Validation(message) => message,
        PlanError::NoMonitor => "Aucun écran n’est disponible.".to_owned(),
    };
    PublicError {
        code: "execution_plan_failed",
        message,
        field: None,
        retryable: false,
    }
}
