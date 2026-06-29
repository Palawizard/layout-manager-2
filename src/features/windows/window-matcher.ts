import type { DesktopWindow, WindowMatcher } from "../../lib/tauri/windows";

function processNameStem(name: string): string {
  return name.replace(/\.exe$/i, "");
}

function processNameEq(expected: string, actual: string): boolean {
  return processNameStem(expected).toLowerCase() === processNameStem(actual).toLowerCase();
}

function sameWindow(left: DesktopWindow, right: DesktopWindow): boolean {
  return (
    left.processId === right.processId &&
    left.title === right.title &&
    left.className === right.className &&
    left.bounds.x === right.bounds.x &&
    left.bounds.y === right.bounds.y &&
    left.bounds.width === right.bounds.width &&
    left.bounds.height === right.bounds.height
  );
}

function matchesCriteria(
  window: DesktopWindow,
  criteria: Pick<WindowMatcher, "processName" | "className" | "titlePattern">,
): boolean {
  if (
    criteria.processName &&
    window.processName &&
    !processNameEq(criteria.processName, window.processName)
  ) {
    return false;
  }
  if (
    criteria.className &&
    window.className.localeCompare(criteria.className, undefined, { sensitivity: "accent" }) !== 0
  ) {
    return false;
  }
  if (criteria.titlePattern) {
    try {
      if (!new RegExp(criteria.titlePattern).test(window.title)) {
        return false;
      }
    } catch {
      return false;
    }
  }
  return true;
}

export function computeInstanceIndex(
  selected: DesktopWindow,
  allWindows: DesktopWindow[],
  criteria: Pick<WindowMatcher, "processName" | "className" | "titlePattern">,
): number | null {
  const ranked = allWindows
    .filter((window) => matchesCriteria(window, criteria))
    .sort((left, right) => left.processId - right.processId);
  const index = ranked.findIndex((window) => sameWindow(window, selected));
  return index >= 0 ? index : null;
}

export function buildWindowMatcher(
  window: DesktopWindow,
  allWindows: DesktopWindow[],
): WindowMatcher {
  const criteria = {
    processName: window.processName,
    className: window.className || null,
    titlePattern: null,
  };

  return {
    executablePath: null,
    ...criteria,
    instanceIndex: computeInstanceIndex(window, allWindows, criteria),
  };
}
