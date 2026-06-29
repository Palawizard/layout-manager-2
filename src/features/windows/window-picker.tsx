import { Monitor, RefreshCw, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button } from "../../components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "../../components/ui/dialog";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  listDesktopWindows,
  type DesktopWindow,
  type WindowMatcher,
} from "../../lib/tauri/windows";
import { buildWindowMatcher } from "./window-matcher";

interface WindowPickerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (window: DesktopWindow, matcher: WindowMatcher) => void;
}

export function WindowPicker({ open, onOpenChange, onSelect }: WindowPickerProps) {
  const [windows, setWindows] = useState<DesktopWindow[]>([]);
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "error">("idle");

  const loadWindows = useCallback(async () => {
    setStatus("loading");
    try {
      setWindows(await listDesktopWindows());
      setStatus("idle");
    } catch {
      setStatus("error");
    }
  }, []);

  useEffect(() => {
    if (open) void Promise.resolve().then(loadWindows);
  }, [loadWindows, open]);

  const filteredWindows = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase("fr");
    if (!normalizedQuery) return windows;
    return windows.filter((window) =>
      `${window.processName ?? ""} ${window.title}`
        .toLocaleLowerCase("fr")
        .includes(normalizedQuery),
    );
  }, [query, windows]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <div className="flex items-start justify-between gap-4">
          <div>
            <DialogTitle>Choisir une fenêtre</DialogTitle>
            <DialogDescription>Sélectionnez une fenêtre actuellement ouverte.</DialogDescription>
          </div>
          <Button
            aria-label="Actualiser les fenêtres"
            onClick={() => void loadWindows()}
            size="icon"
            variant="ghost"
          >
            <RefreshCw aria-hidden="true" className="size-4" />
          </Button>
        </div>
        <div className="mt-5">
          <Label className="sr-only" htmlFor="window-search">
            Rechercher
          </Label>
          <div className="relative">
            <Search
              aria-hidden="true"
              className="absolute left-3 top-3 size-4 text-muted-foreground"
            />
            <Input
              id="window-search"
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Rechercher une application"
              value={query}
              className="pl-9"
            />
          </div>
        </div>
        <div className="mt-4 max-h-80 overflow-y-auto rounded-md border border-border">
          {status === "loading" && (
            <p className="p-6 text-center text-sm text-muted-foreground">Recherche des fenêtres…</p>
          )}
          {status === "error" && (
            <div className="p-6 text-center">
              <p className="text-sm">Impossible d’afficher les fenêtres.</p>
              <Button className="mt-4" onClick={() => void loadWindows()} variant="secondary">
                Réessayer
              </Button>
            </div>
          )}
          {status === "idle" && filteredWindows.length === 0 && (
            <p className="p-6 text-center text-sm text-muted-foreground">Aucune fenêtre trouvée.</p>
          )}
          {status === "idle" &&
            filteredWindows.map((window) => (
              <button
                className="flex w-full items-center gap-3 border-b border-border p-3 text-left last:border-0 hover:bg-muted focus-visible:bg-muted"
                key={`${window.processId}-${window.className}-${window.title}`}
                onClick={() => {
                  onSelect(window, buildWindowMatcher(window, windows));
                  onOpenChange(false);
                }}
                type="button"
              >
                <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted">
                  <Monitor aria-hidden="true" className="size-4" />
                </span>
                <span className="min-w-0">
                  <span className="block truncate text-sm font-medium">
                    {window.processName ?? "Application"}
                  </span>
                  <span className="block truncate text-xs text-muted-foreground">
                    {window.title}
                  </span>
                </span>
              </button>
            ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
