import type { MonitorFallback } from "../../../lib/tauri/monitors";
import type { WindowMatcher, WindowState } from "../../../lib/tauri/windows";

export type BrowserKind = "edge" | "chrome" | "firefox" | "system_default";

export type PlacementPreset =
  | "fullScreen"
  | "leftHalf"
  | "rightHalf"
  | "topHalf"
  | "bottomHalf"
  | "topLeftQuarter"
  | "topRightQuarter"
  | "bottomLeftQuarter"
  | "bottomRightQuarter"
  | "custom";

export interface NormalizedBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface MonitorSelector {
  preferredId: string;
  fallback: MonitorFallback;
}

export interface WindowPlacement {
  monitorSelector: MonitorSelector;
  bounds: NormalizedBounds;
  state: WindowState;
}

export interface LayoutOptions {
  minimizeUnmatchedWindows: boolean;
  continueOnError: boolean;
  restorePreviousStateOnCancel: boolean;
}

export type LayoutAction =
  | {
      kind: "launch_application";
      id: string;
      executablePath: string;
      arguments: string[];
      workingDirectory: string | null;
      reuseExistingWindow: boolean;
      windowMatcher: WindowMatcher;
      placement: WindowPlacement;
      startupTimeoutMs: number;
    }
  | {
      kind: "place_existing_window";
      id: string;
      windowMatcher: WindowMatcher;
      placement: WindowPlacement;
    }
  | {
      kind: "open_browser_window";
      id: string;
      browserKind: BrowserKind;
      executablePath: string | null;
      profile: string | null;
      urls: string[];
      placement: WindowPlacement;
      startupTimeoutMs: number;
    };

export interface Layout {
  id: string;
  name: string;
  description: string | null;
  actions: LayoutAction[];
  options: LayoutOptions;
  createdAt: number;
  updatedAt: number;
}

export interface LayoutSummary {
  id: string;
  name: string;
  description: string | null;
  actionCount: number;
  updatedAt: number;
}

export interface AppSettings {
  preferredBrowser: BrowserKind;
  defaultStartupTimeoutMs: number;
  monitorFallback: MonitorFallback;
}
