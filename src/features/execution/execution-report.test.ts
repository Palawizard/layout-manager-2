import { describe, expect, it } from "vitest";

import type { LayoutRunReport } from "./types/execution";

describe("execution report", () => {
  it("distinguishes partial failure from total failure", () => {
    const partial: LayoutRunReport["status"] = "partial_failure";
    const failed: LayoutRunReport["status"] = "failed";
    expect(partial).not.toBe(failed);
  });
});
