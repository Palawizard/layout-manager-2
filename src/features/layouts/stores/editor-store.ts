import { create } from "zustand";

import type { Layout, LayoutAction } from "../types/layout";
import { createEmptyLayout } from "../lib/defaults";

interface EditorState {
  draft: Layout;
  baseline: string;
  isDirty: boolean;
  setDraft: (layout: Layout, markClean?: boolean) => void;
  resetDraft: () => void;
  updateDetails: (name: string, description: string | null) => void;
  setActions: (actions: LayoutAction[]) => void;
  updateAction: (index: number, action: LayoutAction) => void;
  removeAction: (index: number) => void;
  moveAction: (from: number, to: number) => void;
  setOptions: (options: Layout["options"]) => void;
}

function serializeDraft(layout: Layout) {
  return JSON.stringify(layout);
}

export const useEditorStore = create<EditorState>((set, get) => ({
  draft: createEmptyLayout(),
  baseline: serializeDraft(createEmptyLayout()),
  isDirty: false,
  setDraft: (layout, markClean = true) =>
    set({
      draft: layout,
      baseline: markClean ? serializeDraft(layout) : get().baseline,
      isDirty: markClean ? false : serializeDraft(layout) !== get().baseline,
    }),
  resetDraft: () => {
    const empty = createEmptyLayout();
    set({ draft: empty, baseline: serializeDraft(empty), isDirty: false });
  },
  updateDetails: (name, description) => {
    const draft = { ...get().draft, name, description };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
  setActions: (actions) => {
    const draft = { ...get().draft, actions };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
  updateAction: (index, action) => {
    const actions = [...get().draft.actions];
    actions[index] = action;
    const draft = { ...get().draft, actions };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
  removeAction: (index) => {
    const actions = get().draft.actions.filter((_, current) => current !== index);
    const draft = { ...get().draft, actions };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
  moveAction: (from, to) => {
    const actions = [...get().draft.actions];
    const [item] = actions.splice(from, 1);
    if (!item) return;
    actions.splice(to, 0, item);
    const draft = { ...get().draft, actions };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
  setOptions: (options) => {
    const draft = { ...get().draft, options };
    set({ draft, isDirty: serializeDraft(draft) !== get().baseline });
  },
}));
