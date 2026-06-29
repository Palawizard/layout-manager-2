import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

import { Button } from "../../../components/ui/button";
import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { validateExecutable } from "../../../lib/tauri/layouts";
import type { LayoutAction } from "../types/layout";
import { createDefaultPlacement } from "../lib/defaults";
import { PlacementSelector } from "./placement-selector";

interface ApplicationActionEditorProps {
  action: Extract<LayoutAction, { kind: "launch_application" }>;
  onChange: (action: Extract<LayoutAction, { kind: "launch_application" }>) => void;
}

export function ApplicationActionEditor({ action, onChange }: ApplicationActionEditorProps) {
  const [displayName, setDisplayName] = useState(
    action.executablePath.split("\\").pop() ?? "Application",
  );
  const [error, setError] = useState<string | null>(null);

  async function chooseExecutable() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Application", extensions: ["exe"] }],
    });
    if (!selected || Array.isArray(selected)) return;
    try {
      const name = await validateExecutable(selected);
      setDisplayName(name);
      setError(null);
      onChange({
        ...action,
        executablePath: selected,
        windowMatcher: {
          ...action.windowMatcher,
          executablePath: selected,
          processName: name,
        },
      });
    } catch {
      setError("Choisissez un fichier d’application valide.");
    }
  }

  return (
    <div className="space-y-4">
      <div>
        <Label>Application</Label>
        <div className="mt-2 flex gap-2">
          <Input readOnly value={displayName} />
          <Button onClick={() => void chooseExecutable()} type="button" variant="secondary">
            Choisir
          </Button>
        </div>
        {error ? <p className="mt-2 text-sm text-danger">{error}</p> : null}
      </div>
      <div>
        <Label htmlFor="app-arguments">Arguments</Label>
        <Input
          id="app-arguments"
          onChange={(event) =>
            onChange({
              ...action,
              arguments: event.target.value
                .split(/\s+/)
                .map((item) => item.trim())
                .filter(Boolean),
            })
          }
          placeholder="Un argument par espace"
          value={action.arguments.join(" ")}
        />
      </div>
      <div>
        <Label htmlFor="app-working-directory">Répertoire de travail</Label>
        <Input
          id="app-working-directory"
          onChange={(event) =>
            onChange({
              ...action,
              workingDirectory: event.target.value || null,
            })
          }
          value={action.workingDirectory ?? ""}
        />
      </div>
      <label className="flex items-center gap-2 text-sm">
        <input
          checked={action.reuseExistingWindow}
          onChange={(event) => onChange({ ...action, reuseExistingWindow: event.target.checked })}
          type="checkbox"
        />
        Réutiliser une fenêtre déjà ouverte
      </label>
      <div>
        <Label htmlFor="app-timeout">Délai d’attente (secondes)</Label>
        <Input
          id="app-timeout"
          min={1}
          onChange={(event) =>
            onChange({
              ...action,
              startupTimeoutMs: Number(event.target.value) * 1000,
            })
          }
          type="number"
          value={action.startupTimeoutMs / 1000}
        />
      </div>
      <PlacementSelector
        onChange={(placement) => onChange({ ...action, placement })}
        value={action.placement ?? createDefaultPlacement()}
      />
    </div>
  );
}
