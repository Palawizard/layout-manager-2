import type { Layout, LayoutAction } from "../types/layout";

export function normalizeLayoutAction(action: LayoutAction): LayoutAction {
  if (action.kind !== "place_existing_window") {
    return action;
  }
  return {
    ...action,
    executablePath: action.executablePath ?? null,
    reopenIfAbsent: action.reopenIfAbsent ?? true,
    startupTimeoutMs: action.startupTimeoutMs ?? 15_000,
  };
}

export function normalizeLayout(layout: Layout): Layout {
  return {
    ...layout,
    actions: layout.actions.map(normalizeLayoutAction),
  };
}
