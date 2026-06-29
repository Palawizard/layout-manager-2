import { describe, expect, it } from "vitest";

import { describeRunStatus, getRetryableActionIds } from "./lib/retry";
import type { LayoutRunReport } from "./types/execution";

describe("execution report", () => {
  it("distinguishes partial failure from total failure", () => {
    const partial: LayoutRunReport["status"] = "partial_failure";
    const failed: LayoutRunReport["status"] = "failed";
    expect(partial).not.toBe(failed);
  });

  it("collects retryable failed actions", () => {
    const ids = getRetryableActionIds({
      runId: "run-1",
      status: "partial_failure",
      durationMs: 1,
      completedActions: 1,
      totalActions: 2,
      warnings: [],
      results: [
        {
          actionId: "a1",
          label: "One",
          status: "succeeded",
          message: null,
          reusedExistingWindow: false,
          retryable: false,
        },
        {
          actionId: "a2",
          label: "Two",
          status: "failed",
          message: "Fenêtre introuvable.",
          reusedExistingWindow: false,
          retryable: true,
        },
      ],
    });
    expect(ids).toEqual(["a2"]);
    expect(describeRunStatus("partial_failure")).toContain("Certaines actions");
  });
});
