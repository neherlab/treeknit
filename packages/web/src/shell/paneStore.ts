import { useCallback } from "react";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { INITIAL_PANE_STATE, type PaneId, paneIsOpen, type PaneOpenState, withPaneOpen } from "./panes";

const PANE_STORAGE_KEY = "treeknit-panes";

interface PaneStore extends PaneOpenState {
  setOpen(pane: PaneId, overlay: boolean, open: boolean): void;
}

const usePaneStore = create<PaneStore>()(
  persist(
    (set) => ({
      ...INITIAL_PANE_STATE,
      setOpen(pane, overlay, open) {
        set((state) => withPaneOpen(state, pane, overlay, open));
      },
    }),
    {
      name: PANE_STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
      partialize: ({ wide }) => ({ wide }),
    },
  ),
);

export function usePaneOpen(pane: PaneId, overlay: boolean): [boolean, (open: boolean) => void] {
  const open = usePaneStore((state) => paneIsOpen(state, pane, overlay));

  const setOpen = useCallback(
    (next: boolean) => {
      usePaneStore.getState().setOpen(pane, overlay, next);
    },
    [pane, overlay],
  );

  return [open, setOpen];
}

export function togglePane(pane: PaneId): void {
  const store = usePaneStore.getState();

  store.setOpen(pane, false, !paneIsOpen(store, pane, false));
}
