import type { DesktopWindow, WindowMatcher } from "../../lib/tauri/windows";

export function buildWindowMatcher(window: DesktopWindow): WindowMatcher {
  return {
    executablePath: window.executablePath,
    processName: window.processName,
    className: window.className || null,
    titlePattern: null,
    instanceIndex: null,
  };
}
