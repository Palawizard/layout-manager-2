import type { NormalizedBounds, PlacementPreset } from "../types/layout";

export const placementPresetLabels: Record<PlacementPreset, string> = {
  fullScreen: "Plein écran",
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
  "leftHalf",
  "rightHalf",
  "topHalf",
  "bottomHalf",
  "topLeftQuarter",
  "topRightQuarter",
  "bottomLeftQuarter",
  "bottomRightQuarter",
];

export function presetToBounds(preset: PlacementPreset): NormalizedBounds {
  switch (preset) {
    case "fullScreen":
      return { x: 0, y: 0, width: 1, height: 1 };
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

export function detectPreset(bounds: NormalizedBounds): PlacementPreset {
  for (const preset of standardPlacementPresets) {
    const candidate = presetToBounds(preset);
    if (
      candidate.x === bounds.x &&
      candidate.y === bounds.y &&
      candidate.width === bounds.width &&
      candidate.height === bounds.height
    ) {
      return preset;
    }
  }
  return "custom";
}
