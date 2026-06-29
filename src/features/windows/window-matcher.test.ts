import { describe, expect, it } from "vitest";

import type { DesktopWindow } from "../../lib/tauri/windows";
import { buildWindowMatcher, computeInstanceIndex } from "./window-matcher";

function window(
  processId: number,
  title: string,
  processName: string,
  x = 0,
): DesktopWindow {
  return {
    processId,
    executablePath: null,
    processName,
    title,
    className: "Chrome_WidgetWin_1",
    bounds: { x, y: 0, width: 800, height: 600 },
    state: "normal",
    monitorId: null,
  };
}

describe("buildWindowMatcher", () => {
  it("assigns the zero-based instance index of the selected window", () => {
    const windows = [
      window(10, "Discord A", "Discord.exe", 0),
      window(20, "Discord B", "Discord.exe", 100),
      window(30, "Vesktop", "vesktop.exe", 200),
    ];
    const selected = windows[1];

    expect(computeInstanceIndex(selected, windows, { processName: "Discord.exe", className: "Chrome_WidgetWin_1", titlePattern: null })).toBe(1);
    expect(buildWindowMatcher(windows[2], windows)).toEqual({
      executablePath: null,
      processName: "vesktop.exe",
      className: "Chrome_WidgetWin_1",
      titlePattern: null,
      instanceIndex: 0,
    });
  });
});
