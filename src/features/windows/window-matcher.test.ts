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

  it("orders same-process windows deterministically for instance selection", () => {
    const windows = [
      { ...window(42, "", "VHConsole.exe"), bounds: { x: 0, y: 0, width: 1920, height: 1080 } },
      { ...window(42, "VIVE Hub", "VHConsole.exe"), bounds: { x: 100, y: 100, width: 320, height: 240 } },
    ];

    expect(
      computeInstanceIndex(windows[1], windows, {
        processName: "VHConsole.exe",
        className: "Chrome_WidgetWin_1",
        titlePattern: null,
      }),
    ).toBe(1);
  });

  it("ignores auxiliary steam panels when computing the instance index", () => {
    const windows = [
      {
        processId: 42,
        executablePath: "C:\\Steam\\steamwebhelper.exe",
        processName: "steamwebhelper.exe",
        title: "Liste de contacts",
        className: "SDL_app",
        bounds: { x: 0, y: 0, width: 160, height: 28 },
        state: "normal" as const,
        monitorId: null,
      },
      {
        processId: 42,
        executablePath: "C:\\Steam\\steamwebhelper.exe",
        processName: "steamwebhelper.exe",
        title: "Steam",
        className: "SDL_app",
        bounds: { x: 0, y: 0, width: 1280, height: 696 },
        state: "normal" as const,
        monitorId: null,
      },
    ];

    expect(buildWindowMatcher(windows[1], windows).instanceIndex).toBe(0);
  });
});
