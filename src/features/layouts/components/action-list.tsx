import { ArrowDown, ArrowUp, Pencil, Trash2 } from "lucide-react";
import { useState } from "react";

import { Button } from "../../../components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "../../../components/ui/dialog";
import type { LayoutAction } from "../types/layout";
import { actionLabel } from "../lib/defaults";
import { ApplicationActionEditor } from "./application-action-editor";
import { BrowserActionEditor } from "./browser-action-editor";
import { ExistingWindowActionEditor } from "./existing-window-action-editor";

interface ActionListProps {
  actions: LayoutAction[];
  editingIndex: number | null;
  onChange: (actions: LayoutAction[]) => void;
  onCancelEdit: () => void;
  onConfirmEdit: () => void;
  onEditingIndexChange: (index: number | null) => void;
}

const actionTypeLabels: Record<LayoutAction["kind"], string> = {
  launch_application: "Application",
  place_existing_window: "Fenêtre existante",
  open_browser_window: "Navigateur",
};

export function ActionList({
  actions,
  editingIndex,
  onChange,
  onCancelEdit,
  onConfirmEdit,
  onEditingIndexChange,
}: ActionListProps) {
  const [pendingDeleteIndex, setPendingDeleteIndex] = useState<number | null>(null);

  function moveAction(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= actions.length) return;
    const next = [...actions];
    const [item] = next.splice(index, 1);
    if (!item) return;
    next.splice(target, 0, item);
    onChange(next);
  }

  function updateAction(index: number, action: LayoutAction) {
    const next = [...actions];
    next[index] = action;
    onChange(next);
  }

  return (
    <>
      <ul aria-label="Actions du layout" className="space-y-3" role="list">
        {actions.map((action, index) => (
          <li
            className="flex items-center justify-between gap-3 rounded-md border border-border p-3"
            key={action.id}
          >
            <div className="min-w-0">
              <p className="text-sm font-medium">{actionTypeLabels[action.kind]}</p>
              <p className="truncate text-sm text-muted-foreground">{actionLabel(action)}</p>
            </div>
            <div className="flex shrink-0 items-center gap-1">
              <Button
                aria-label="Monter l’action"
                disabled={index === 0}
                onClick={() => moveAction(index, -1)}
                size="icon"
                type="button"
                variant="ghost"
              >
                <ArrowUp aria-hidden="true" className="size-4" />
              </Button>
              <Button
                aria-label="Descendre l’action"
                disabled={index === actions.length - 1}
                onClick={() => moveAction(index, 1)}
                size="icon"
                type="button"
                variant="ghost"
              >
                <ArrowDown aria-hidden="true" className="size-4" />
              </Button>
              <Button
                aria-label="Modifier l’action"
                onClick={() => onEditingIndexChange(index)}
                size="icon"
                type="button"
                variant="ghost"
              >
                <Pencil aria-hidden="true" className="size-4" />
              </Button>
              <Button
                aria-label="Supprimer l’action"
                onClick={() => setPendingDeleteIndex(index)}
                size="icon"
                type="button"
                variant="ghost"
              >
                <Trash2 aria-hidden="true" className="size-4" />
              </Button>
            </div>
          </li>
        ))}
      </ul>

      <Dialog open={editingIndex !== null}>
        <DialogContent
          className="max-h-[85vh] max-w-2xl overflow-y-auto"
          dismissOnEscape={false}
          dismissOnOutsideClick={false}
        >
          <DialogTitle>Modifier l’action</DialogTitle>
          <DialogDescription>Ajustez les paramètres de cette action.</DialogDescription>
          {editingIndex !== null && actions[editingIndex]?.kind === "launch_application" && (
            <ApplicationActionEditor
              action={actions[editingIndex]}
              onChange={(action) => updateAction(editingIndex, action)}
            />
          )}
          {editingIndex !== null && actions[editingIndex]?.kind === "place_existing_window" && (
            <ExistingWindowActionEditor
              action={actions[editingIndex]}
              onChange={(action) => updateAction(editingIndex, action)}
            />
          )}
          {editingIndex !== null && actions[editingIndex]?.kind === "open_browser_window" && (
            <BrowserActionEditor
              action={actions[editingIndex]}
              onChange={(action) => updateAction(editingIndex, action)}
            />
          )}
          <div className="mt-6 flex justify-end gap-2">
            <Button onClick={onCancelEdit} variant="secondary">
              Annuler
            </Button>
            <Button onClick={onConfirmEdit}>Enregistrer</Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog
        onOpenChange={(open) => !open && setPendingDeleteIndex(null)}
        open={pendingDeleteIndex !== null}
      >
        <DialogContent>
          <DialogTitle>Supprimer cette action ?</DialogTitle>
          <DialogDescription>Cette action sera retirée du layout.</DialogDescription>
          <div className="mt-6 flex justify-end gap-2">
            <Button onClick={() => setPendingDeleteIndex(null)} variant="secondary">
              Annuler
            </Button>
            <Button
              onClick={() => {
                if (pendingDeleteIndex === null) return;
                onChange(actions.filter((_, index) => index !== pendingDeleteIndex));
                setPendingDeleteIndex(null);
              }}
              variant="danger"
            >
              Supprimer
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
