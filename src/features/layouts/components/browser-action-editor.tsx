import { Plus, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "../../../components/ui/button";
import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { listInstalledBrowsers } from "../../../lib/tauri/execution";
import type { BrowserKind, LayoutAction } from "../types/layout";
import { createDefaultPlacement } from "../lib/defaults";
import { PlacementSelector } from "./placement-selector";

interface BrowserActionEditorProps {
  action: Extract<LayoutAction, { kind: "open_browser_window" }>;
  onChange: (action: Extract<LayoutAction, { kind: "open_browser_window" }>) => void;
}

const browserLabels: Record<BrowserKind, string> = {
  edge: "Microsoft Edge",
  chrome: "Google Chrome",
  firefox: "Mozilla Firefox",
  system_default: "Navigateur par défaut",
};

export function BrowserActionEditor({ action, onChange }: BrowserActionEditorProps) {
  const [availableBrowsers, setAvailableBrowsers] = useState<BrowserKind[]>([
    "edge",
    "chrome",
    "firefox",
    "system_default",
  ]);

  useEffect(() => {
    void listInstalledBrowsers()
      .then((browsers) => {
        if (browsers.length > 0) {
          setAvailableBrowsers(browsers.map((browser) => browser.kind));
        }
      })
      .catch(() => undefined);
  }, []);

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Cette action ouvre une nouvelle fenêtre avec plusieurs onglets.
      </p>
      <div>
        <Label htmlFor="browser-kind">Navigateur</Label>
        <select
          className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
          id="browser-kind"
          onChange={(event) =>
            onChange({ ...action, browserKind: event.target.value as BrowserKind })
          }
          value={action.browserKind}
        >
          {(availableBrowsers as BrowserKind[]).map((kind) => (
            <option key={kind} value={kind}>
              {browserLabels[kind]}
            </option>
          ))}
        </select>
      </div>
      <div>
        <Label htmlFor="browser-profile">Profil (facultatif)</Label>
        <Input
          id="browser-profile"
          onChange={(event) => onChange({ ...action, profile: event.target.value || null })}
          value={action.profile ?? ""}
        />
      </div>
      <div className="space-y-2">
        <Label>Adresses web</Label>
        {action.urls.map((url, index) => (
          <div className="flex gap-2" key={`${index}-${url}`}>
            <Input
              onChange={(event) => {
                const urls = [...action.urls];
                urls[index] = event.target.value;
                onChange({ ...action, urls });
              }}
              placeholder="https://"
              value={url}
            />
            <Button
              aria-label="Supprimer l’adresse"
              onClick={() =>
                onChange({
                  ...action,
                  urls: action.urls.filter((_, current) => current !== index),
                })
              }
              size="icon"
              type="button"
              variant="ghost"
            >
              <Trash2 aria-hidden="true" className="size-4" />
            </Button>
          </div>
        ))}
        <Button
          onClick={() => onChange({ ...action, urls: [...action.urls, ""] })}
          type="button"
          variant="secondary"
        >
          <Plus aria-hidden="true" className="size-4" />
          Ajouter une adresse
        </Button>
      </div>
      <div>
        <Label htmlFor="browser-timeout">Délai d’attente (secondes)</Label>
        <Input
          id="browser-timeout"
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
