import { useEffect, useState } from "react";
import { UNSTABLE_ToastQueue as ToastQueue } from "react-aria-components";

import type { ActionToastContent } from "../ui/Toast";
import { useWorkspaceStore } from "./context";
import type { UndoEntry } from "./store";
import { undoMessage } from "./undo";

export const UNDO_TIMEOUT_MS = 10_000;

export function useUndoToast(): ToastQueue<ActionToastContent> {
  const store = useWorkspaceStore();
  const [queue] = useState(() => new ToastQueue<ActionToastContent>({ maxVisibleToasts: 1 }));

  useEffect(() => {
    let shown: { entry: UndoEntry; key: string } | null = null;
    let active = true;

    const show = (entry: UndoEntry | null): void => {
      if (shown !== null && shown.entry !== entry) {
        queue.close(shown.key);
        shown = null;
      }

      if (entry === null || shown !== null) {
        return;
      }

      const key = queue.add(
        {
          message: undoMessage(entry),
          actionLabel: "Undo",
          onAction: () => {
            store.getState().restoreUndo();
          },
          timeoutMs: UNDO_TIMEOUT_MS,
        },
        {
          timeout: UNDO_TIMEOUT_MS,
          onClose: () => {
            if (active) {
              store.getState().dismissUndo(entry);
            }
          },
        },
      );

      shown = { entry, key };
    };

    show(store.getState().undo);

    const unsubscribe = store.subscribe((state, previous) => {
      if (state.undo !== previous.undo) {
        show(state.undo);
      }
    });

    return () => {
      active = false;
      unsubscribe();

      if (shown !== null) {
        queue.close(shown.key);
      }
    };
  }, [queue, store]);

  return queue;
}
