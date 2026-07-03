import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "../../components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "../../components/ui/dialog";

const UPDATE_CHECK_TIMEOUT_MS = 15_000;

interface DownloadProgress {
  downloaded: number;
  total: number | undefined;
}

export function UpdateManager() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [progress, setProgress] = useState<DownloadProgress>({ downloaded: 0, total: undefined });

  useEffect(() => {
    let active = true;

    void check({ timeout: UPDATE_CHECK_TIMEOUT_MS })
      .then((availableUpdate) => {
        if (active && availableUpdate) {
          setUpdate(availableUpdate);
        }
      })
      .catch(() => {
        // Update checks are best-effort and must not interrupt application startup.
      });

    return () => {
      active = false;
    };
  }, []);

  async function installUpdate() {
    if (!update || isInstalling) return;

    setIsInstalling(true);
    setProgress({ downloaded: 0, total: undefined });
    let downloaded = 0;
    let total: number | undefined;

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength;
          setProgress({ downloaded, total });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setProgress({ downloaded, total });
        }
      });
      await relaunch();
    } catch {
      setIsInstalling(false);
      toast.error("La mise à jour n’a pas pu être installée. Réessayez plus tard.");
    }
  }

  const percent = progress.total
    ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
    : undefined;

  return (
    <Dialog
      open={update !== null}
      onOpenChange={(open) => !open && !isInstalling && setUpdate(null)}
    >
      <DialogContent dismissOnEscape={!isInstalling} dismissOnOutsideClick={!isInstalling}>
        <DialogTitle>Mise à jour disponible</DialogTitle>
        <DialogDescription>
          La version {update?.version} est disponible. L’application redémarrera après
          l’installation.
        </DialogDescription>

        {isInstalling ? (
          <div className="mt-5 space-y-2" aria-live="polite">
            <div
              className="h-2 overflow-hidden rounded-full bg-muted"
              role="progressbar"
              aria-label="Téléchargement de la mise à jour"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={percent}
            >
              <div
                className="h-full bg-primary transition-[width]"
                style={{ width: percent === undefined ? "15%" : `${percent}%` }}
              />
            </div>
            <p className="text-sm text-muted-foreground">
              {percent === undefined ? "Téléchargement en cours…" : `Téléchargement : ${percent} %`}
            </p>
          </div>
        ) : null}

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="secondary" disabled={isInstalling} onClick={() => setUpdate(null)}>
            Plus tard
          </Button>
          <Button disabled={isInstalling} onClick={() => void installUpdate()}>
            {isInstalling ? "Installation…" : "Installer et redémarrer"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
