import { LayoutDashboard, Plus, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import { listLayouts } from "../../lib/tauri/layouts";
import { LayoutList } from "./components/layout-list";
import type { LayoutSummary } from "./types/layout";

export function LayoutsPage() {
  const navigate = useNavigate();
  const [layouts, setLayouts] = useState<LayoutSummary[]>([]);
  const [status, setStatus] = useState<"loading" | "idle" | "error">("loading");

  const loadLayouts = useCallback(async () => {
    setStatus("loading");
    try {
      setLayouts(await listLayouts());
      setStatus("idle");
    } catch {
      setStatus("error");
    }
  }, []);

  useEffect(() => {
    void loadLayouts();
  }, [loadLayouts]);

  return (
    <section aria-labelledby="layouts-title">
      <div className="mb-8 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight" id="layouts-title">
            Layouts
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Organisez vos applications et vos fenêtres selon vos besoins.
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            aria-label="Actualiser la liste"
            onClick={() => void loadLayouts()}
            size="icon"
            variant="ghost"
          >
            <RefreshCw aria-hidden="true" className="size-4" />
          </Button>
          <Button onClick={() => navigate("/layouts/new")}>
            <Plus aria-hidden="true" className="size-4" />
            Nouveau layout
          </Button>
        </div>
      </div>

      {status === "loading" && (
        <p className="text-sm text-muted-foreground">Chargement des layouts…</p>
      )}
      {status === "error" && (
        <Card>
          <CardContent className="p-6 text-center">
            <p>Impossible d’afficher les layouts.</p>
            <Button className="mt-4" onClick={() => void loadLayouts()} variant="secondary">
              Réessayer
            </Button>
          </CardContent>
        </Card>
      )}
      {status === "idle" && layouts.length === 0 && (
        <Card>
          <CardContent className="flex min-h-64 flex-col items-center justify-center text-center">
            <span className="mb-4 flex size-12 items-center justify-center rounded-lg bg-muted">
              <LayoutDashboard aria-hidden="true" className="size-6 text-muted-foreground" />
            </span>
            <h2 className="font-medium">Aucun layout</h2>
            <p className="mt-2 max-w-sm text-sm text-muted-foreground">
              Créez un layout pour organiser votre espace de travail.
            </p>
            <Button className="mt-6" onClick={() => navigate("/layouts/new")} variant="secondary">
              Nouveau layout
            </Button>
          </CardContent>
        </Card>
      )}
      {status === "idle" && layouts.length > 0 && (
        <LayoutList layouts={layouts} onEdit={(layoutId) => navigate(`/layouts/${layoutId}`)} />
      )}
    </section>
  );
}
