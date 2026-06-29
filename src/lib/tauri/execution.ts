import type { InstalledBrowser, LayoutRunReport } from "../../features/execution/types/execution";
import { invokeCommand } from "./client";

export function listInstalledBrowsers() {
  return invokeCommand<InstalledBrowser[]>("list_installed_browsers");
}

export function runLayout(layoutId: string, actionIds?: string[]) {
  return invokeCommand<string>("run_layout", actionIds ? { layoutId, actionIds } : { layoutId });
}

export function cancelLayoutRun() {
  return invokeCommand<void>("cancel_layout_run");
}

export type { LayoutRunReport };
