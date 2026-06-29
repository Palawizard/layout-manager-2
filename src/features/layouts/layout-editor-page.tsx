import { useCallback, useEffect, useState } from "react";
import { useBlocker, useNavigate, useParams } from "react-router";
import { toast } from "sonner";

import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader } from "../../components/ui/card";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "../../components/ui/dialog";
import { getLayout, saveLayout } from "../../lib/tauri/layouts";
import { ActionList } from "./components/action-list";
import { LayoutDetailsForm } from "./components/layout-details-form";
import { createDefaultPlacement } from "./lib/defaults";
import { layoutDraftSchema } from "./schemas/layout-schema";
import { useEditorStore } from "./stores/editor-store";
import type { LayoutAction } from "./types/layout";

function createActionId() {
  return crypto.randomUUID();
}

function createAction(kind: LayoutAction["kind"]): LayoutAction {
  const placement = createDefaultPlacement();
  const id = createActionId();
  switch (kind) {
    case "launch_application":
      return {
        kind,
        id,
        executablePath: "",
        arguments: [],
        workingDirectory: null,
        reuseExistingWindow: true,
        windowMatcher: {
          executablePath: null,
          processName: null,
          className: null,
          titlePattern: null,
          instanceIndex: null,
        },
        placement,
        startupTimeoutMs: 15_000,
      };
    case "place_existing_window":
      return {
        kind,
        id,
        windowMatcher: {
          executablePath: null,
          processName: null,
          className: null,
          titlePattern: null,
          instanceIndex: null,
        },
        placement,
      };
    case "open_browser_window":
      return {
        kind,
        id,
        browserKind: "edge",
        executablePath: null,
        profile: null,
        urls: [""],
        placement,
        startupTimeoutMs: 15_000,
      };
  }
}

export function LayoutEditorPage() {
  const navigate = useNavigate();
  const { layoutId } = useParams();
  const isNew = layoutId === "new";
  const { draft, isDirty, resetDraft, setActions, setDraft, updateDetails } = useEditorStore();
  const [status, setStatus] = useState<"loading" | "ready">(() => (isNew ? "ready" : "loading"));
  const [isSaving, setIsSaving] = useState(false);
  const [editingActionIndex, setEditingActionIndex] = useState<number | null>(null);
  const blocker = useBlocker(isDirty && !isSaving);
  const leaveOpen = blocker.state === "blocked";

  useEffect(() => {
    if (isNew) {
      resetDraft();
      return;
    }
    if (!layoutId) return;
    let active = true;
    void getLayout(layoutId)
      .then((layout) => {
        if (!active) return;
        setDraft(layout, true);
        setStatus("ready");
      })
      .catch(() => {
        if (!active) return;
        toast.error("Impossible de charger ce layout.");
        navigate("/layouts");
      });
    return () => {
      active = false;
    };
  }, [isNew, layoutId, navigate, resetDraft, setDraft]);

  const addAction = useCallback(
    (kind: LayoutAction["kind"]) => {
      const newAction = createAction(kind);
      const nextActions = [...draft.actions, newAction];
      setActions(nextActions);
      setEditingActionIndex(nextActions.length - 1);
    },
    [draft.actions, setActions],
  );

  const save = useCallback(async () => {
    const parsed = layoutDraftSchema.safeParse({
      name: draft.name,
      description: draft.description ?? "",
      actions: draft.actions,
      options: draft.options,
    });
    if (!parsed.success) {
      toast.error(parsed.error.issues[0]?.message ?? "Le layout est incomplet.");
      return;
    }
    setIsSaving(true);
    try {
      const saved = await saveLayout({
        ...draft,
        name: parsed.data.name,
        description: parsed.data.description ? parsed.data.description : null,
      });
      setDraft(saved, true);
      toast.success("Layout enregistré.");
      if (isNew) {
        navigate(`/layouts/${saved.id}`, { replace: true });
      }
    } catch (error) {
      const message =
        error && typeof error === "object" && "message" in error
          ? String(error.message)
          : "Impossible d’enregistrer ce layout.";
      toast.error(message);
    } finally {
      setIsSaving(false);
    }
  }, [draft, isNew, navigate, setDraft]);

  if (status === "loading") {
    return <p className="text-sm text-muted-foreground">Chargement du layout…</p>;
  }

  return (
    <section aria-labelledby="editor-title">
      <div className="mb-8 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight" id="editor-title">
            {isNew ? "Nouveau layout" : "Modifier le layout"}
          </h1>
        </div>
        <div className="flex gap-2">
          <Button onClick={() => navigate("/layouts")} variant="secondary">
            Retour
          </Button>
          <Button onClick={() => void save()}>Enregistrer</Button>
        </div>
      </div>

      <div className="grid gap-6">
        <Card>
          <CardHeader>
            <h2 className="font-medium">Informations</h2>
          </CardHeader>
          <CardContent>
            <LayoutDetailsForm
              defaultValues={{
                name: draft.name,
                description: draft.description ?? "",
              }}
              onChange={(values) =>
                updateDetails(values.name, values.description ? values.description : null)
              }
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between gap-4">
            <h2 className="font-medium">Actions</h2>
            <div className="flex flex-wrap gap-2">
              <Button
                onClick={() => addAction("launch_application")}
                size="small"
                variant="secondary"
              >
                Application
              </Button>
              <Button onClick={() => addAction("place_existing_window")} size="small" variant="secondary">
                Fenêtre existante
              </Button>
              <Button onClick={() => addAction("open_browser_window")} size="small" variant="secondary">
                Navigateur
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {draft.actions.length === 0 ? (
              <p className="text-sm text-muted-foreground">Ajoutez au moins une action.</p>
            ) : (
              <ActionList
                actions={draft.actions}
                editingIndex={editingActionIndex}
                onChange={setActions}
                onEditingIndexChange={setEditingActionIndex}
              />
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <h2 className="font-medium">Options</h2>
          </CardHeader>
          <CardContent className="space-y-3">
            <label className="flex items-center gap-2 text-sm">
              <input
                checked={draft.options.minimizeUnmatchedWindows}
                onChange={(event) =>
                  useEditorStore.getState().setOptions({
                    ...draft.options,
                    minimizeUnmatchedWindows: event.target.checked,
                  })
                }
                type="checkbox"
              />
              Réduire les autres fenêtres
            </label>
          </CardContent>
        </Card>
      </div>

      <Dialog
        onOpenChange={(open) => {
          if (!open && blocker.state === "blocked") {
            blocker.reset();
          }
        }}
        open={leaveOpen}
      >
        <DialogContent>
          <DialogTitle>Quitter sans enregistrer ?</DialogTitle>
          <DialogDescription>Les modifications non enregistrées seront perdues.</DialogDescription>
          <div className="mt-6 flex justify-end gap-2">
            <DialogClose asChild>
              <Button
                onClick={() => blocker.state === "blocked" && blocker.reset()}
                variant="secondary"
              >
                Continuer l’édition
              </Button>
            </DialogClose>
            <Button
              onClick={() => {
                if (blocker.state === "blocked") {
                  blocker.proceed();
                }
              }}
              variant="danger"
            >
              Quitter
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
