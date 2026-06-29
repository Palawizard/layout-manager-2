import type { Layout, LayoutAction, LayoutOptions, WindowPlacement } from "../types/layout";

export const defaultLayoutOptions: LayoutOptions = {
  minimizeUnmatchedWindows: false,
  continueOnError: true,
  restorePreviousStateOnCancel: false,
};

export function createDefaultPlacement(monitorId = "primary"): WindowPlacement {
  return {
    monitorSelector: { preferredId: monitorId, fallback: "primary" },
    bounds: { x: 0, y: 0, width: 1, height: 1 },
    state: "normal",
  };
}

export function createEmptyLayout(): Layout {
  return {
    id: "",
    name: "",
    description: null,
    actions: [],
    options: defaultLayoutOptions,
    createdAt: 0,
    updatedAt: 0,
  };
}

export function actionLabel(action: LayoutAction): string {
  switch (action.kind) {
    case "launch_application":
      return action.executablePath.split("\\").pop() ?? "Application";
    case "place_existing_window":
      return action.windowMatcher.processName ?? "Fenêtre existante";
    case "open_browser_window":
      return action.browserKind === "system_default" ? "Navigateur" : action.browserKind;
  }
}
