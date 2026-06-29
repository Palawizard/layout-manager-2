import { invokeCommand } from "./client";

export type WindowState = "normal" | "maximized" | "minimized";

export interface DesktopWindow {
  processId: number;
  executablePath: string | null;
  processName: string | null;
  title: string;
  className: string;
  bounds: { x: number; y: number; width: number; height: number };
  state: WindowState;
  monitorId: string | null;
}

export function listDesktopWindows() {
  return invokeCommand<DesktopWindow[]>("list_desktop_windows");
}
