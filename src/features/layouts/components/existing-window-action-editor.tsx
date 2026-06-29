import { useState } from "react";

import { Button } from "../../../components/ui/button";
import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { listMonitors } from "../../../lib/tauri/monitors";
import { resolveLaunchExecutable } from "../../../lib/tauri/layouts";
import { WindowPicker } from "../../windows/window-picker";
import type { LayoutAction } from "../types/layout";
import { createDefaultPlacement } from "../lib/defaults";
import { clonePlacement, placementFromWindow } from "../lib/placement-from-window";
import { PlacementSelector } from "./placement-selector";

interface ExistingWindowActionEditorProps {
  action: Extract<LayoutAction, { kind: "place_existing_window" }>;
  onChange: (action: Extract<LayoutAction, { kind: "place_existing_window" }>) => void;
}

export function ExistingWindowActionEditor({ action, onChange }: ExistingWindowActionEditorProps) {
  const [pickerOpen, setPickerOpen] = useState(false);

  return (
    <div className="space-y-4">
      <div>
        <Label>Fenêtre à retrouver</Label>
        <div className="mt-2 flex gap-2">
          <Input
            readOnly
            value={action.windowMatcher.processName ?? "Aucune fenêtre sélectionnée"}
          />
          <Button onClick={() => setPickerOpen(true)} type="button" variant="secondary">
            Choisir
          </Button>
        </div>
        {action.executablePath ? (
          <p className="mt-2 truncate text-xs text-muted-foreground">
            Relance : {action.executablePath}
          </p>
        ) : null}
      </div>
      <div>
        <Label htmlFor="title-pattern">Titre à retrouver (facultatif)</Label>
        <Input
          id="title-pattern"
          onChange={(event) =>
            onChange({
              ...action,
              windowMatcher: {
                ...action.windowMatcher,
                titlePattern: event.target.value || null,
              },
            })
          }
          placeholder="Partie du titre"
          value={action.windowMatcher.titlePattern ?? ""}
        />
      </div>
      <div>
        <Label htmlFor="instance-index">Occurrence (facultatif)</Label>
        <Input
          id="instance-index"
          min={0}
          onChange={(event) =>
            onChange({
              ...action,
              windowMatcher: {
                ...action.windowMatcher,
                instanceIndex: event.target.value ? Number(event.target.value) : null,
              },
            })
          }
          placeholder="Laisser vide pour la première fenêtre"
          type="number"
          value={action.windowMatcher.instanceIndex ?? ""}
        />
      </div>
      <div className="flex items-start gap-3 rounded-md border border-border p-4">
        <input
          checked={!action.reopenIfAbsent}
          className="mt-1 size-4 rounded border-border"
          id="skip-reopen-if-absent"
          onChange={(event) =>
            onChange({
              ...action,
              reopenIfAbsent: !event.target.checked,
            })
          }
          type="checkbox"
        />
        <div>
          <Label htmlFor="skip-reopen-if-absent">Ne pas réouvrir si absente</Label>
          <p className="mt-1 text-sm text-muted-foreground">
            Si l’application est fermée, elle sera relancée automatiquement avant le placement, sauf
            si cette option est cochée.
          </p>
        </div>
      </div>
      <PlacementSelector
        capturedPlacement={action.capturedPlacement}
        onChange={(placement) => onChange({ ...action, placement })}
        value={action.placement ?? createDefaultPlacement()}
      />
      <WindowPicker
        onOpenChange={setPickerOpen}
        onSelect={(window, matcher) => {
          void listMonitors()
            .then(async (monitors) => {
              const placement = placementFromWindow(window, monitors) ?? action.placement;
              const capturedPlacement = clonePlacement(placement);
              const executablePath = window.executablePath
                ? await resolveLaunchExecutable(window.executablePath)
                : null;
              onChange({
                ...action,
                windowMatcher: matcher,
                executablePath,
                placement,
                capturedPlacement,
              });
            })
            .catch(() => {
              onChange({
                ...action,
                windowMatcher: matcher,
                executablePath: window.executablePath,
              });
            })
            .finally(() => {
              setPickerOpen(false);
            });
        }}
        open={pickerOpen}
      />
    </div>
  );
}
