import { useMemo, useState } from "react";

import { createTreeViewStore, type TreeViewHandle, treeViewActions } from "../canvas/useTreeView";

export function useDrawingView(rows: number): TreeViewHandle {
  const [store] = useState(createTreeViewStore);

  return useMemo(() => {
    const drawing = { rows };

    return { store, drawing, actions: treeViewActions(store, drawing) };
  }, [store, rows]);
}
