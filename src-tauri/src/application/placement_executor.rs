use crate::{
    application::{
        client_window::is_placeable_client_window,
        window_service::apply_window_placement,
    },
    application::execution_planner::PlannedLaunch,
    domain::{
        execution::{ActionRunResult, ActionRunStatus},
        ports::{NativeError, WindowController, WindowInventory},
        window::NativeWindowHandle,
    },
};

pub fn apply_planned_placement(
    step: &PlannedLaunch,
    inventory: &impl WindowInventory,
    controller: &impl WindowController,
    handle: NativeWindowHandle,
    reused: bool,
) -> ActionRunResult {
    let placement = step.placement();
    let action_id = step.action_id().clone();
    let label = step.label().to_owned();
    if let Some(matcher) = step.window_matcher() {
        let Ok(windows) = inventory.list_windows_including_untitled() else {
            return failed_placement(action_id, label, reused, "Impossible d’inspecter les fenêtres.");
        };
        if let Some(window) = windows.iter().find(|window| window.handle == handle) {
            if !is_placeable_client_window(window, matcher) {
                return failed_placement(
                    action_id,
                    label,
                    reused,
                    "Fenêtre auxiliaire ignorée pour le placement.",
                );
            }
        }
    }
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
        NativeError::AccessDenied => crate::domain::ports::ACCESS_DENIED_USER_MESSAGE.to_owned(),
        NativeError::InvalidHandle => "Fenêtre introuvable.".to_owned(),
        NativeError::OperationFailed(_) => "Placement impossible.".to_owned(),
    }
}

fn failed_placement(
    action_id: crate::domain::layout::LayoutActionId,
    label: String,
    reused: bool,
    message: &str,
) -> ActionRunResult {
    ActionRunResult {
        action_id,
        label,
        status: ActionRunStatus::Failed,
        message: Some(message.to_owned()),
        reused_existing_window: reused,
        retryable: true,
    }
}
