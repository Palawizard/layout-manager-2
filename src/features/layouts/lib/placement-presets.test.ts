import { describe, expect, it } from "vitest";

import type { WindowPlacement } from "../types/layout";
import {
  MIN_CENTER_SCALE,
  applyPlacementPreset,
  centerScaleToBounds,
  detectCenterScale,
  detectPreset,
} from "./placement-presets";

function placement(
  bounds: WindowPlacement["bounds"],
  patch: Partial<WindowPlacement> = {},
): WindowPlacement {
  return {
    monitorSelector: { preferredId: "\\\\.\\DISPLAY1", fallback: "primary" },
    bounds,
    state: "normal",
    ...patch,
  };
}

describe("placement presets", () => {
  it("locks full screen to maximized state", () => {
    const result = applyPlacementPreset(
      placement({ x: 0, y: 0, width: 0.5, height: 1 }),
      "fullScreen",
    );
    expect(result.state).toBe("maximized");
    expect(detectPreset(result)).toBe("fullScreen");
  });

  it("builds centered bounds from scale", () => {
    expect(centerScaleToBounds(1)).toEqual({ x: 0, y: 0, width: 1, height: 1 });
    expect(centerScaleToBounds(0.5)).toEqual({ x: 0.25, y: 0.25, width: 0.5, height: 0.5 });
    expect(centerScaleToBounds(MIN_CENTER_SCALE).width).toBe(MIN_CENTER_SCALE);
  });

  it("detects centered placements", () => {
    const centered = placement(centerScaleToBounds(0.4), { centerScale: 0.4 });
    expect(detectPreset(centered)).toBe("center");
    expect(detectCenterScale(centerScaleToBounds(0.4))).toBeCloseTo(0.4);
  });

  it("resets maximized state when leaving full screen", () => {
    const full = applyPlacementPreset(placement({ x: 0, y: 0, width: 1, height: 1 }), "fullScreen");
    const half = applyPlacementPreset(full, "leftHalf");
    expect(half.state).toBe("normal");
    expect(half.centerScale).toBeNull();
  });
});
