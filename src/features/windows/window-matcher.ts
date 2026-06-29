import type { DesktopWindow, WindowMatcher } from "../../lib/tauri/windows";

const MIN_CLIENT_WIDTH = 120;
const MIN_CLIENT_HEIGHT = 80;

const ANCILLARY_PANEL_KEYWORDS = [
  "contact list",
  "liste de contacts",
  "liste des contacts",
  "friends list",
  "liste d'amis",
  "liste des amis",
  "chat list",
  "liste de discussions",
];

function isAuxiliaryWindow(window: DesktopWindow): boolean {
  return window.bounds.width < MIN_CLIENT_WIDTH || window.bounds.height < MIN_CLIENT_HEIGHT;
}

function hasAncillaryPanelTitle(title: string): boolean {
  const normalized = title.trim().toLowerCase();
  return ANCILLARY_PANEL_KEYWORDS.some((keyword) => normalized.includes(keyword));
}

function isNonClientWindow(window: DesktopWindow): boolean {
  return isAuxiliaryWindow(window) || hasAncillaryPanelTitle(window.title);
}

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
    .filter(
      (window) =>
        matchesCriteria(window, criteria) && !isNonClientWindow(window),
    )
    .sort(
      (left, right) =>
        left.processId - right.processId ||
        left.title.localeCompare(right.title) ||
        left.bounds.x - right.bounds.x ||
        left.bounds.y - right.bounds.y ||
        left.bounds.width - right.bounds.width ||
        left.bounds.height - right.bounds.height,
    );
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
    executablePath: window.executablePath,
    ...criteria,
    instanceIndex: computeInstanceIndex(window, allWindows, criteria),
  };
}
