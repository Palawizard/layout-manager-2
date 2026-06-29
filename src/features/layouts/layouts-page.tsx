import { LayoutDashboard, Plus, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { toast } from "sonner";

import { Button } from "../../components/ui/button";
import { Card, CardContent } from "../../components/ui/card";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "../../components/ui/dialog";
import { deleteLayout, duplicateLayout, listLayouts } from "../../lib/tauri/layouts";
import { LayoutList } from "./components/layout-list";
import type { LayoutSummary } from "./types/layout";

export function LayoutsPage() {
  const navigate = useNavigate();
  const [layouts, setLayouts] = useState<LayoutSummary[]>([]);
  const [status, setStatus] = useState<"loading" | "idle" | "error">("loading");
  const [pendingDelete, setPendingDelete] = useState<LayoutSummary | null>(null);

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
    let active = true;
    void listLayouts()
      .then((data) => {
        if (!active) return;
        setLayouts(data);
        setStatus("idle");
      })
      .catch(() => {
        if (active) setStatus("error");
      });
    return () => {
      active = false;
    };
  }, []);

  async function handleDuplicate(layoutId: string) {
    try {
      const duplicate = await duplicateLayout(layoutId);
      toast.success(`Layout dupliqué : ${duplicate.name}`);
      await loadLayouts();
    } catch {
      toast.error("Impossible de dupliquer ce layout.");
    }
  }

  async function handleDelete() {
    if (!pendingDelete) return;
    try {
      await deleteLayout(pendingDelete.id);
      toast.success("Layout supprimé.");
      setPendingDelete(null);
      await loadLayouts();
    } catch {
      toast.error("Impossible de supprimer ce layout.");
    }
  }

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
        <LayoutList
          layouts={layouts}
          onDelete={(layoutId) =>
            setPendingDelete(layouts.find((layout) => layout.id === layoutId) ?? null)
          }
          onDuplicate={(layoutId) => void handleDuplicate(layoutId)}
          onEdit={(layoutId) => navigate(`/layouts/${layoutId}`)}
        />
      )}

      <Dialog
        onOpenChange={(open) => !open && setPendingDelete(null)}
        open={Boolean(pendingDelete)}
      >
        <DialogContent>
          <DialogTitle>Supprimer ce layout ?</DialogTitle>
          <DialogDescription>
            {pendingDelete
              ? `Le layout « ${pendingDelete.name} » sera définitivement supprimé.`
              : ""}
          </DialogDescription>
          <div className="mt-6 flex justify-end gap-2">
            <DialogClose asChild>
              <Button variant="secondary">Annuler</Button>
            </DialogClose>
            <Button onClick={() => void handleDelete()} variant="danger">
              Supprimer
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
