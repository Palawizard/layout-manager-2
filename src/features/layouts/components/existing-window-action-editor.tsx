import { useState } from "react";

import { Button } from "../../../components/ui/button";
import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { WindowPicker } from "../../windows/window-picker";
import type { LayoutAction } from "../types/layout";
import { createDefaultPlacement } from "../lib/defaults";
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
          placeholder="Expression exacte ou vide"
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
          placeholder="Automatique si vide — 0 = première, 1 = deuxième…"
          type="number"
          value={action.windowMatcher.instanceIndex ?? ""}
        />
      </div>
      <PlacementSelector
        onChange={(placement) => onChange({ ...action, placement })}
        value={action.placement ?? createDefaultPlacement()}
      />
      <WindowPicker
        onOpenChange={setPickerOpen}
        onSelect={(_window, matcher) => {
          onChange({ ...action, windowMatcher: matcher });
          setPickerOpen(false);
        }}
        open={pickerOpen}
      />
    </div>
  );
}
