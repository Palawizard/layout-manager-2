import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { getAppInfo, type AppInfo } from "../../lib/tauri/app-info";
import { getSettings, openDataDirectory, openLogsDirectory, saveSettings } from "../../lib/tauri/layouts";
import type { AppSettings, BrowserKind } from "../layouts/types/layout";
import type { MonitorFallback } from "../../lib/tauri/monitors";

const browserLabels: Record<BrowserKind, string> = {
  edge: "Microsoft Edge",
  chrome: "Google Chrome",
  firefox: "Mozilla Firefox",
  system_default: "Navigateur par défaut",
};

export function SettingsPage() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");

  useEffect(() => {
    let active = true;
    void Promise.all([getAppInfo(), getSettings()])
      .then(([info, loadedSettings]) => {
        if (!active) return;
        setAppInfo(info);
        setSettings(loadedSettings);
        setStatus("ready");
      })
      .catch(() => {
        if (active) setStatus("error");
      });
    return () => {
      active = false;
    };
  }, []);

  async function handleSave() {
    if (!settings) return;
    try {
      const saved = await saveSettings(settings);
      setSettings(saved);
      toast.success("Réglages enregistrés.");
    } catch {
      toast.error("Impossible d’enregistrer les réglages.");
    }
  }

  return (
    <section aria-labelledby="settings-title">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold tracking-tight" id="settings-title">
          Réglages
        </h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Personnalisez le fonctionnement de l’application.
        </p>
      </div>

      {status === "loading" && (
        <p className="text-sm text-muted-foreground">Chargement des réglages…</p>
      )}
      {status === "error" && (
        <p className="text-sm text-danger">Impossible d’afficher les réglages.</p>
      )}

      {status === "ready" && settings ? (
        <div className="grid gap-6">
          <Card>
            <CardHeader>
              <h2 className="font-medium">Préférences</h2>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div>
                <Label htmlFor="preferred-browser">Navigateur préféré</Label>
                <select
                  className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
                  id="preferred-browser"
                  onChange={(event) =>
                    setSettings({
                      ...settings,
                      preferredBrowser: event.target.value as BrowserKind,
                    })
                  }
                  value={settings.preferredBrowser}
                >
                  {(Object.keys(browserLabels) as BrowserKind[]).map((kind) => (
                    <option key={kind} value={kind}>
                      {browserLabels[kind]}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <Label htmlFor="default-timeout">Délai d’attente par défaut (secondes)</Label>
                <Input
                  id="default-timeout"
                  min={1}
                  onChange={(event) =>
                    setSettings({
                      ...settings,
                      defaultStartupTimeoutMs: Number(event.target.value) * 1000,
                    })
                  }
                  type="number"
                  value={settings.defaultStartupTimeoutMs / 1000}
                />
              </div>
              <div>
                <Label htmlFor="monitor-fallback">Écran de secours</Label>
                <select
                  className="mt-2 h-10 w-full rounded-md border border-border bg-surface px-3 text-sm"
                  id="monitor-fallback"
                  onChange={(event) =>
                    setSettings({
                      ...settings,
                      monitorFallback: event.target.value as MonitorFallback,
                    })
                  }
                  value={settings.monitorFallback}
                >
                  <option value="primary">Écran principal</option>
                  <option value="first_available">Premier écran disponible</option>
                </select>
              </div>
              <Button onClick={() => void handleSave()}>Enregistrer</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <h2 className="font-medium">Diagnostic</h2>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Consultez les journaux locaux de l’application. Les informations sensibles y sont
                limitées.
              </p>
              <Button onClick={() => void openLogsDirectory()} variant="secondary">
                Ouvrir le dossier des journaux
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <h2 className="font-medium">Données</h2>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Ouvrez le dossier où sont enregistrés vos layouts et vos réglages.
              </p>
              <Button onClick={() => void openDataDirectory()} variant="secondary">
                Ouvrir le dossier des données
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <h2 className="font-medium">À propos</h2>
              <p className="text-sm text-muted-foreground">
                Layout Manager 2{appInfo ? ` · Version ${appInfo.version}` : ""}
              </p>
            </CardHeader>
          </Card>
        </div>
      ) : null}
    </section>
  );
}
