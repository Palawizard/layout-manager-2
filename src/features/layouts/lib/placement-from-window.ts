import type { Monitor } from "../../../lib/tauri/monitors";
import type { DesktopWindow } from "../../../lib/tauri/windows";
import type { WindowPlacement } from "../types/layout";

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function windowCenter(window: DesktopWindow): { x: number; y: number } {
  return {
    x: window.bounds.x + window.bounds.width / 2,
    y: window.bounds.y + window.bounds.height / 2,
  };
}

function containsPoint(workArea: Monitor["workArea"], x: number, y: number): boolean {
  return (
    x >= workArea.x &&
    x < workArea.x + workArea.width &&
    y >= workArea.y &&
    y < workArea.y + workArea.height
  );
}

const BOUNDS_TOLERANCE = 0.002;

export function clonePlacement(placement: WindowPlacement): WindowPlacement {
  return {
    monitorSelector: { ...placement.monitorSelector },
    bounds: { ...placement.bounds },
    state: placement.state,
    centerScale: placement.centerScale ?? null,
  };
}

export function placementsMatch(left: WindowPlacement, right: WindowPlacement): boolean {
  return (
    left.monitorSelector.preferredId === right.monitorSelector.preferredId &&
    left.state === right.state &&
    (left.centerScale ?? null) === (right.centerScale ?? null) &&
    Math.abs(left.bounds.x - right.bounds.x) <= BOUNDS_TOLERANCE &&
    Math.abs(left.bounds.y - right.bounds.y) <= BOUNDS_TOLERANCE &&
    Math.abs(left.bounds.width - right.bounds.width) <= BOUNDS_TOLERANCE &&
    Math.abs(left.bounds.height - right.bounds.height) <= BOUNDS_TOLERANCE
  );
}

export function placementFromWindow(
  window: DesktopWindow,
  monitors: Monitor[],
): WindowPlacement | null {
  if (monitors.length === 0) {
    return null;
  }

  const center = windowCenter(window);
  const monitor =
    (window.monitorId ? monitors.find((item) => item.id === window.monitorId) : undefined) ??
    monitors.find((item) => containsPoint(item.workArea, center.x, center.y)) ??
    monitors.find((item) => item.isPrimary) ??
    monitors[0];
  if (!monitor) {
    return null;
  }

  const { workArea } = monitor;
  if (workArea.width <= 0 || workArea.height <= 0) {
    return null;
  }

  const bounds = {
    x: clamp01((window.bounds.x - workArea.x) / workArea.width),
    y: clamp01((window.bounds.y - workArea.y) / workArea.height),
    width: clamp01(window.bounds.width / workArea.width),
    height: clamp01(window.bounds.height / workArea.height),
  };

  return {
    monitorSelector: { preferredId: monitor.id, fallback: "primary" },
    bounds: {
      x: bounds.x,
      y: bounds.y,
      width: Math.min(bounds.width, 1 - bounds.x),
      height: Math.min(bounds.height, 1 - bounds.y),
    },
    state: window.state,
    centerScale: null,
  };
}
