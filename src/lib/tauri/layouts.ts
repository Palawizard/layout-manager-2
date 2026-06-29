import type { AppSettings, Layout, LayoutSummary } from "../../features/layouts/types/layout";
import { invokeCommand } from "./client";

export function listLayouts() {
  return invokeCommand<LayoutSummary[]>("list_layouts");
}

export function getLayout(layoutId: string) {
  return invokeCommand<Layout>("get_layout", { layoutId });
}

export function saveLayout(layout: Layout) {
  return invokeCommand<Layout>("save_layout", { layout });
}

export function duplicateLayout(layoutId: string) {
  return invokeCommand<Layout>("duplicate_layout", { layoutId });
}

export function deleteLayout(layoutId: string) {
  return invokeCommand<void>("delete_layout", { layoutId });
}

export function validateExecutable(path: string) {
  return invokeCommand<string>("validate_executable", { path });
}

export function getSettings() {
  return invokeCommand<AppSettings>("get_settings");
}

export function saveSettings(settings: AppSettings) {
  return invokeCommand<AppSettings>("save_settings", { settings });
}

export function openDataDirectory() {
  return invokeCommand<void>("open_data_directory");
}
