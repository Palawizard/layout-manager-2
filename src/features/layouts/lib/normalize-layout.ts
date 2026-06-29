import type { Layout, LayoutAction, WindowPlacement } from "../types/layout";
import { clonePlacement } from "./placement-from-window";

export function normalizeLayoutAction(action: LayoutAction): LayoutAction {
  if (action.kind !== "place_existing_window") {
    return action;
  }
  const capturedPlacement =
    action.capturedPlacement ?? clonePlacement(action.placement as WindowPlacement);
  return {
    ...action,
    executablePath: action.executablePath ?? null,
    reopenIfAbsent: action.reopenIfAbsent ?? true,
    startupTimeoutMs: action.startupTimeoutMs ?? 15_000,
    capturedPlacement,
  };
}

export function normalizeLayout(layout: Layout): Layout {
  return {
    ...layout,
    actions: layout.actions.map(normalizeLayoutAction),
  };
}
