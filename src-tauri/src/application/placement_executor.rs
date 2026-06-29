use crate::{
    application::window_service::apply_window_placement,
    application::execution_planner::PlannedLaunch,
    domain::{
        execution::{ActionRunResult, ActionRunStatus},
        ports::{NativeError, WindowController},
        window::NativeWindowHandle,
    },
};

pub fn apply_planned_placement(
    step: &PlannedLaunch,
    controller: &impl WindowController,
    handle: NativeWindowHandle,
    reused: bool,
) -> ActionRunResult {
    let placement = step.placement();
    let action_id = step.action_id().clone();
    let label = step.label().to_owned();
    if let Err(error) = apply_window_placement(
        controller,
        handle,
        placement.bounds,
        placement.state,
    ) {
        return ActionRunResult {
            action_id,
            label,
            status: ActionRunStatus::Failed,
            message: Some(native_error_message(error)),
            reused_existing_window: reused,
            retryable: true,
        };
    }
    ActionRunResult {
        action_id,
        label,
        status: ActionRunStatus::Succeeded,
        message: None,
        reused_existing_window: reused,
        retryable: false,
    }
}

fn native_error_message(error: NativeError) -> String {
    match error {
        NativeError::AccessDenied => "Cette fenêtre n’est pas accessible.".to_owned(),
        NativeError::InvalidHandle => "Fenêtre introuvable.".to_owned(),
        NativeError::OperationFailed(_) => "Placement impossible.".to_owned(),
    }
}
