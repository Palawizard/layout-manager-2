export type LayoutRunStatus = "success" | "partial_failure" | "failed" | "cancelled";

export type ActionRunStatus =
  "pending" | "running" | "succeeded" | "failed" | "skipped" | "cancelled";

export interface RunWarning {
  code: string;
  message: string;
  actionId: string | null;
}

export interface ActionRunResult {
  actionId: string;
  label: string;
  status: ActionRunStatus;
  message: string | null;
  reusedExistingWindow: boolean;
  retryable: boolean;
}

export interface LayoutRunReport {
  runId: string;
  status: LayoutRunStatus;
  durationMs: number;
  completedActions: number;
  totalActions: number;
  warnings: RunWarning[];
  results: ActionRunResult[];
}

export interface RunStartedEvent {
  runId: string;
  layoutId: string;
  layoutName: string;
  totalActions: number;
}

export interface ActionStartedEvent {
  runId: string;
  actionId: string;
  label: string;
  phase: string;
}

export interface ActionCompletedEvent {
  runId: string;
  result: ActionRunResult;
}

export interface RunCompletedEvent {
  report: LayoutRunReport;
}

export interface InstalledBrowser {
  kind: "edge" | "chrome" | "firefox" | "system_default";
  executablePath: string;
  label: string;
}
