import { useMemo, useState } from "react";

import { type TreeView, type TreeViewActions, useTreeView } from "../canvas/useTreeView";
import { drawingLeafAxis } from "./narrow";

export function useDrawingView(rows: number): TreeView {
  const [width, setWidth] = useState<number | undefined>(undefined);
  const view = useTreeView(rows, drawingLeafAxis(width));
  const inner = view.actions;

  const actions = useMemo<TreeViewActions>(
    () => ({
      ...inner,
      resize(size) {
        setWidth(size.width);
        inner.resize(size);
      },
    }),
    [inner],
  );

  return { ...view, actions };
}
