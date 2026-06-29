import type { NormalizedBounds, PlacementPreset, WindowPlacement } from "../types/layout";

export const MIN_CENTER_SCALE = 0.1;

export const placementPresetLabels: Record<PlacementPreset, string> = {
  fullScreen: "Plein écran",
  center: "Centre",
  leftHalf: "Moitié gauche",
  rightHalf: "Moitié droite",
  topHalf: "Moitié haute",
  bottomHalf: "Moitié basse",
  topLeftQuarter: "Quart haut gauche",
  topRightQuarter: "Quart haut droit",
  bottomLeftQuarter: "Quart bas gauche",
  bottomRightQuarter: "Quart bas droit",
  custom: "Zone personnalisée",
};

export const standardPlacementPresets: PlacementPreset[] = [
  "fullScreen",
  "center",
  "leftHalf",
  "rightHalf",
  "topHalf",
  "bottomHalf",
  "topLeftQuarter",
  "topRightQuarter",
  "bottomLeftQuarter",
  "bottomRightQuarter",
];

export function centerScaleToBounds(scale: number): NormalizedBounds {
  const clamped = Math.min(1, Math.max(MIN_CENTER_SCALE, scale));
  const margin = (1 - clamped) / 2;
  return { x: margin, y: margin, width: clamped, height: clamped };
}

export function presetToBounds(preset: PlacementPreset): NormalizedBounds {
  switch (preset) {
    case "fullScreen":
      return { x: 0, y: 0, width: 1, height: 1 };
    case "center":
      return centerScaleToBounds(0.5);
    case "leftHalf":
      return { x: 0, y: 0, width: 0.5, height: 1 };
    case "rightHalf":
      return { x: 0.5, y: 0, width: 0.5, height: 1 };
    case "topHalf":
      return { x: 0, y: 0, width: 1, height: 0.5 };
    case "bottomHalf":
      return { x: 0, y: 0.5, width: 1, height: 0.5 };
    case "topLeftQuarter":
      return { x: 0, y: 0, width: 0.5, height: 0.5 };
    case "topRightQuarter":
      return { x: 0.5, y: 0, width: 0.5, height: 0.5 };
    case "bottomLeftQuarter":
      return { x: 0, y: 0.5, width: 0.5, height: 0.5 };
    case "bottomRightQuarter":
      return { x: 0.5, y: 0.5, width: 0.5, height: 0.5 };
    case "custom":
      return { x: 0, y: 0, width: 1, height: 1 };
  }
}

const BOUNDS_TOLERANCE = 0.002;

function boundsMatch(left: NormalizedBounds, right: NormalizedBounds): boolean {
  return (
    Math.abs(left.x - right.x) <= BOUNDS_TOLERANCE &&
    Math.abs(left.y - right.y) <= BOUNDS_TOLERANCE &&
    Math.abs(left.width - right.width) <= BOUNDS_TOLERANCE &&
    Math.abs(left.height - right.height) <= BOUNDS_TOLERANCE
  );
}

export function detectCenterScale(bounds: NormalizedBounds): number | null {
  const scale = bounds.width;
  if (Math.abs(bounds.width - bounds.height) > BOUNDS_TOLERANCE) {
    return null;
  }
  const margin = (1 - scale) / 2;
  if (
    Math.abs(bounds.x - margin) > BOUNDS_TOLERANCE ||
    Math.abs(bounds.y - margin) > BOUNDS_TOLERANCE
  ) {
    return null;
  }
  if (scale < MIN_CENTER_SCALE - BOUNDS_TOLERANCE || scale > 1 + BOUNDS_TOLERANCE) {
    return null;
  }
  return scale;
}

export function detectPreset(placement: WindowPlacement): PlacementPreset {
  if (placement.centerScale != null) {
    return "center";
  }
  if (
    boundsMatch(placement.bounds, presetToBounds("fullScreen")) &&
    placement.state === "maximized"
  ) {
    return "fullScreen";
  }
  for (const preset of standardPlacementPresets) {
    if (preset === "center" || preset === "fullScreen") {
      continue;
    }
    if (boundsMatch(placement.bounds, presetToBounds(preset))) {
      return preset;
    }
  }
  if (detectCenterScale(placement.bounds) != null) {
    return "center";
  }
  return "custom";
}

export function applyPlacementPreset(
  placement: WindowPlacement,
  preset: PlacementPreset,
): WindowPlacement {
  if (preset === "fullScreen") {
    return {
      ...placement,
      bounds: presetToBounds("fullScreen"),
      state: "maximized",
      centerScale: null,
    };
  }
  if (preset === "center") {
    const scale = placement.centerScale ?? detectCenterScale(placement.bounds) ?? 0.5;
    return {
      ...placement,
      bounds: centerScaleToBounds(scale),
      state: "normal",
      centerScale: scale,
    };
  }
  const wasFullScreen = detectPreset(placement) === "fullScreen";
  return {
    ...placement,
    bounds: presetToBounds(preset),
    centerScale: null,
    state: wasFullScreen ? "normal" : placement.state,
  };
}

export function applyCenterScale(placement: WindowPlacement, scale: number): WindowPlacement {
  const clamped = Math.min(1, Math.max(MIN_CENTER_SCALE, scale));
  return {
    ...placement,
    bounds: centerScaleToBounds(clamped),
    state: "normal",
    centerScale: clamped,
  };
}
