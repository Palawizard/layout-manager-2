use serde::{Deserialize, Serialize};

use super::layout::LayoutActionId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutRunStatus {
    Success,
    PartialFailure,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunWarning {
    pub code: String,
    pub message: String,
    pub action_id: Option<LayoutActionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunResult {
    pub action_id: LayoutActionId,
    pub label: String,
    pub status: ActionRunStatus,
    pub message: Option<String>,
    pub reused_existing_window: bool,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutRunReport {
    pub run_id: RunId,
    pub status: LayoutRunStatus,
    pub duration_ms: u64,
    pub completed_actions: usize,
    pub total_actions: usize,
    pub warnings: Vec<RunWarning>,
    pub results: Vec<ActionRunResult>,
}

#[must_use]
pub fn aggregate_run_status(results: &[ActionRunResult], cancelled: bool) -> LayoutRunStatus {
    if cancelled {
        return LayoutRunStatus::Cancelled;
    }
    if results.is_empty() {
        return LayoutRunStatus::Failed;
    }
    let succeeded = results
        .iter()
        .filter(|result| result.status == ActionRunStatus::Succeeded)
        .count();
    let failed = results
        .iter()
        .filter(|result| result.status == ActionRunStatus::Failed)
        .count();
    if failed == 0 {
        LayoutRunStatus::Success
    } else if succeeded > 0 {
        LayoutRunStatus::PartialFailure
    } else {
        LayoutRunStatus::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActionRunResult, ActionRunStatus, LayoutActionId, LayoutRunStatus, aggregate_run_status,
    };

    fn result(status: ActionRunStatus) -> ActionRunResult {
        ActionRunResult {
            action_id: LayoutActionId("action-1".to_owned()),
            label: "Test".to_owned(),
            status,
            message: None,
            reused_existing_window: false,
            retryable: false,
        }
    }

    #[test]
    fn reports_success_when_every_action_succeeds() {
        let results = vec![
            result(ActionRunStatus::Succeeded),
            result(ActionRunStatus::Succeeded),
        ];
        assert_eq!(
            aggregate_run_status(&results, false),
            LayoutRunStatus::Success
        );
    }

    #[test]
    fn reports_partial_failure_when_some_actions_fail() {
        let results = vec![
            result(ActionRunStatus::Succeeded),
            result(ActionRunStatus::Failed),
        ];
        assert_eq!(
            aggregate_run_status(&results, false),
            LayoutRunStatus::PartialFailure
        );
    }

    #[test]
    fn reports_failed_when_every_action_fails() {
        let results = vec![
            result(ActionRunStatus::Failed),
            result(ActionRunStatus::Failed),
        ];
        assert_eq!(
            aggregate_run_status(&results, false),
            LayoutRunStatus::Failed
        );
    }

    #[test]
    fn reports_cancelled_over_other_statuses() {
        let results = vec![result(ActionRunStatus::Succeeded)];
        assert_eq!(
            aggregate_run_status(&results, true),
            LayoutRunStatus::Cancelled
        );
    }

    #[test]
    fn reports_failed_for_an_empty_result_set() {
        assert_eq!(aggregate_run_status(&[], false), LayoutRunStatus::Failed);
    }
}
