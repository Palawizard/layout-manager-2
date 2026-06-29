import { describe, expect, it } from "vitest";

import type { Monitor } from "../../../lib/tauri/monitors";
import type { DesktopWindow } from "../../../lib/tauri/windows";
import { placementFromWindow } from "./placement-from-window";

const monitor: Monitor = {
  id: "\\\\.\\DISPLAY1",
  name: "Display 1",
  isPrimary: true,
  scaleFactor: 1,
  workArea: { x: 0, y: 0, width: 1920, height: 1040 },
};

const window: DesktopWindow = {
  processId: 1,
  executablePath: "C:\\Apps\\Discord.exe",
  processName: "Discord.exe",
  title: "Discord",
  className: "Chrome_WidgetWin_1",
  bounds: { x: 0, y: 0, width: 960, height: 1040 },
  state: "normal",
  monitorId: monitor.id,
};

describe("placementFromWindow", () => {
  it("converts pixel bounds to normalized placement on the selected monitor", () => {
    expect(placementFromWindow(window, [monitor])).toEqual({
      monitorSelector: { preferredId: monitor.id, fallback: "primary" },
      bounds: { x: 0, y: 0, width: 0.5, height: 1 },
      state: "normal",
      centerScale: null,
    });
  });
});
