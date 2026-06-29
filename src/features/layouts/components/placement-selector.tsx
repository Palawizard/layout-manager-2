import { useEffect, useMemo, useState } from "react";

import { Label } from "../../../components/ui/label";
import type { Monitor } from "../../../lib/tauri/monitors";
import { listMonitors } from "../../../lib/tauri/monitors";
import type { WindowState } from "../../../lib/tauri/windows";
import type { WindowPlacement, PlacementPreset } from "../types/layout";
import { clonePlacement, placementsMatch } from "../lib/placement-from-window";
import {
  MIN_CENTER_SCALE,
  applyCenterScale,
  applyPlacementPreset,
  detectPreset,
  placementPresetLabels,
  standardPlacementPresets,
} from "../lib/placement-presets";

interface PlacementSelectorProps {
  value: WindowPlacement;
  onChange: (placement: WindowPlacement) => void;
  capturedPlacement?: WindowPlacement;
}

const windowStateLabels: Record<WindowState, string> = {
  normal: "Normal",
  maximized: "Agrandi",
  minimized: "Réduit",
};

const CAPTURED_PLACEMENT_LABEL = "Position actuelle";

export function PlacementSelector({
  capturedPlacement,
  onChange,
  value,
}: PlacementSelectorProps) {
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const isCapturedSelected = useMemo(
    () => Boolean(capturedPlacement && placementsMatch(value, capturedPlacement)),
    [capturedPlacement, value],
  );
  const preset = useMemo(
    () => (isCapturedSelected ? null : detectPreset(value)),
    [isCapturedSelected, value],
  );
  const centerScalePercent = Math.round((value.centerScale ?? MIN_CENTER_SCALE) * 100);
  const zoneLabel = isCapturedSelected
    ? CAPTURED_PLACEMENT_LABEL
    : preset
      ? placementPresetLabels[preset]
      : "Zone personnalisée";

  const preferredMonitorId = value.monitorSelector.preferredId;

  useEffect(() => {
    void listMonitors()
      .then(setMonitors)
      .catch(() => setMonitors([]));
  }, []);

  useEffect(() => {
    if (monitors.length === 0) {
      return;
    }
    const isPlaceholder = preferredMonitorId === "primary";
    const isKnown = monitors.some((monitor) => monitor.id === preferredMonitorId);
    if (!isPlaceholder && isKnown) {
      return;
    }
    const preferred = monitors.find((monitor) => monitor.isPrimary) ?? monitors[0];
    if (preferred.id === preferredMonitorId) {
      return;
    }
    onChange({
      ...value,
      monitorSelector: {
        ...value.monitorSelector,
        preferredId: preferred.id,
      },
    });
  }, [monitors, onChange, preferredMonitorId, value]);

  function updatePlacement(patch: Partial<WindowPlacement>) {
    onChange({ ...value, ...patch });
  }

  function selectPreset(item: PlacementPreset) {
    onChange(applyPlacementPreset(value, item));
  }

  function selectCapturedPlacement() {
    if (!capturedPlacement) {
      return;
    }
    onChange(clonePlacement(capturedPlacement));
  }

  return (
    <div className="space-y-4">
      <div>
        <Label htmlFor="placement-monitor">Écran</Label>
        <select
          className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
          id="placement-monitor"
          onChange={(event) =>
            updatePlacement({
              monitorSelector: {
                ...value.monitorSelector,
                preferredId: event.target.value,
              },
            })
          }
          value={value.monitorSelector.preferredId}
        >
          {monitors.map((monitor) => (
            <option key={monitor.id} value={monitor.id}>
              {monitor.name}
              {monitor.isPrimary ? " · Principal" : ""}
            </option>
          ))}
        </select>
      </div>

      <div>
        <Label>Zone</Label>
        <div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3">
          {capturedPlacement ? (
            <button
              className={`rounded-md border px-3 py-2 text-left text-sm ${
                isCapturedSelected ? "border-primary bg-muted" : "border-border"
              }`}
              onClick={selectCapturedPlacement}
              type="button"
            >
              {CAPTURED_PLACEMENT_LABEL}
            </button>
          ) : null}
          {standardPlacementPresets.map((item) => (
            <button
              className={`rounded-md border px-3 py-2 text-left text-sm ${
                preset === item ? "border-primary bg-muted" : "border-border"
              }`}
              key={item}
              onClick={() => selectPreset(item)}
              type="button"
            >
              {placementPresetLabels[item]}
            </button>
          ))}
        </div>
      </div>

      {preset === "center" && (
        <div>
          <Label htmlFor="center-scale">Taille de la zone ({centerScalePercent} %)</Label>
          <input
            className="mt-2 w-full accent-primary"
            id="center-scale"
            max={100}
            min={MIN_CENTER_SCALE * 100}
            onChange={(event) =>
              onChange(applyCenterScale(value, Number(event.target.value) / 100))
            }
            step={1}
            type="range"
            value={centerScalePercent}
          />
          <p className="mt-2 text-xs text-muted-foreground">
            Fenêtre centrée en taille normale, de 10 % à 100 % de l’écran.
          </p>
        </div>
      )}

      <div className="rounded-md border border-border p-4">
        <p className="mb-3 text-sm font-medium">Aperçu</p>
        <MonitorPreview monitors={monitors} placement={value} zoneLabel={zoneLabel} />
      </div>

      {preset === "custom" && (
        <div className="grid grid-cols-2 gap-3">
          {(["x", "y", "width", "height"] as const).map((field) => (
            <div key={field}>
              <Label htmlFor={`bounds-${field}`}>{field.toUpperCase()}</Label>
              <input
                className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
                id={`bounds-${field}`}
                max={1}
                min={0}
                onChange={(event) =>
                  updatePlacement({
                    bounds: {
                      ...value.bounds,
                      [field]: Number(event.target.value),
                    },
                    centerScale: null,
                  })
                }
                step={0.01}
                type="number"
                value={value.bounds[field]}
              />
            </div>
          ))}
        </div>
      )}

      <div>
        <Label htmlFor="placement-state">État final</Label>
        {preset === "fullScreen" ? (
          <p className="mt-2 text-sm text-muted-foreground" id="placement-state">
            Plein écran impose l’état « Agrandi ».
          </p>
        ) : (
          <select
            className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
            id="placement-state"
            onChange={(event) => updatePlacement({ state: event.target.value as WindowState })}
            value={value.state}
          >
            {(Object.keys(windowStateLabels) as WindowState[]).map((state) => (
              <option key={state} value={state}>
                {windowStateLabels[state]}
              </option>
            ))}
          </select>
        )}
      </div>
    </div>
  );
}

function MonitorPreview({
  monitors,
  placement,
  zoneLabel,
}: {
  monitors: Monitor[];
  placement: WindowPlacement;
  zoneLabel: string;
}) {
  if (monitors.length === 0) {
    return <p className="text-sm text-muted-foreground">Aucun écran détecté.</p>;
  }

  const minX = Math.min(...monitors.map((monitor) => monitor.workArea.x));
  const minY = Math.min(...monitors.map((monitor) => monitor.workArea.y));
  const maxX = Math.max(...monitors.map((monitor) => monitor.workArea.x + monitor.workArea.width));
  const maxY = Math.max(...monitors.map((monitor) => monitor.workArea.y + monitor.workArea.height));
  const width = maxX - minX;
  const height = maxY - minY;
  const selected =
    monitors.find((monitor) => monitor.id === placement.monitorSelector.preferredId) ??
    monitors.at(0);
  if (!selected) {
    return null;
  }

  const windowStyle = {
    left: `${((selected.workArea.x - minX + placement.bounds.x * selected.workArea.width) / width) * 100}%`,
    top: `${((selected.workArea.y - minY + placement.bounds.y * selected.workArea.height) / height) * 100}%`,
    width: `${((placement.bounds.width * selected.workArea.width) / width) * 100}%`,
    height: `${((placement.bounds.height * selected.workArea.height) / height) * 100}%`,
  };

  return (
    <div className="relative h-36 rounded-md bg-muted">
      {monitors.map((monitor) => (
        <div
          className="absolute rounded-sm border border-border bg-surface/80"
          key={monitor.id}
          style={{
            left: `${((monitor.workArea.x - minX) / width) * 100}%`,
            top: `${((monitor.workArea.y - minY) / height) * 100}%`,
            width: `${(monitor.workArea.width / width) * 100}%`,
            height: `${(monitor.workArea.height / height) * 100}%`,
          }}
        />
      ))}
      <div
        aria-label={`Zone sélectionnée : ${zoneLabel}`}
        className="absolute rounded-sm border-2 border-primary bg-primary/20"
        style={windowStyle}
      />
    </div>
  );
}
