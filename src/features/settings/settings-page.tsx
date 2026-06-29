import { useEffect, useState } from "react";

import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader } from "../../components/ui/card";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "../../components/ui/dialog";
import { getAppInfo, type AppInfo } from "../../lib/tauri/app-info";

export function SettingsPage() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    let active = true;
    void getAppInfo()
      .then((info) => {
        if (active) setAppInfo(info);
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, []);

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
      <Card>
        <CardHeader>
          <h2 className="font-medium">À propos</h2>
          <p className="text-sm text-muted-foreground">
            Layout Manager 2{appInfo ? ` · Version ${appInfo.version}` : ""}
          </p>
        </CardHeader>
        <CardContent>
          <Dialog>
            <DialogTrigger asChild>
              <Button variant="secondary">Afficher les informations</Button>
            </DialogTrigger>
            <DialogContent>
              <DialogTitle>Layout Manager 2</DialogTitle>
              <DialogDescription>
                Organisez rapidement vos applications et vos fenêtres sur Windows.
              </DialogDescription>
              <div className="mt-6 flex justify-end">
                <DialogClose asChild>
                  <Button>Fermer</Button>
                </DialogClose>
              </div>
            </DialogContent>
          </Dialog>
        </CardContent>
      </Card>
    </section>
  );
}
