import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";

import { getAppInfo } from "./app-info";

describe("getAppInfo", () => {
  it("uses the typed app information command", async () => {
    const info = await getAppInfo();

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_app_info", undefined);
    expect(info).toEqual({
      name: "Layout Manager 2",
      version: "0.1.0-beta.1",
      platform: "windows",
    });
  });
});
