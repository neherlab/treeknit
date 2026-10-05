import { useMemo, useState } from "react";

import { createTreeViewStore, type TreeViewHandle, treeViewActions } from "../canvas/useTreeView";
import { drawingLeafAxis } from "./narrow";

export function useDrawingView(rows: number): TreeViewHandle {
  const [store] = useState(createTreeViewStore);
  const [width, setWidth] = useState<number | undefined>(undefined);
  const drawing = useMemo(() => ({ rows, leafAxis: drawingLeafAxis(width) }), [rows, width]);

  return useMemo(() => {
    const inner = treeViewActions(store, drawing);

    return {
      store,
      drawing,
      actions: {
        ...inner,
        resize(size) {
          setWidth(size.width);
          inner.resize(size);
        },
      },
    };
  }, [store, drawing]);
}
