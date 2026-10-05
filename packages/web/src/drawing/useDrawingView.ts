import { useMemo, useState } from "react";

import { type TreeView, type TreeViewActions, useTreeView } from "../canvas/useTreeView";
import { drawingLeafAxis } from "./narrow";

export function useDrawingView(rows: number): TreeView {
  const [width, setWidth] = useState<number | undefined>(undefined);
  const view = useTreeView(rows, drawingLeafAxis(width));
  const inner = view.actions;

  const actions = useMemo<TreeViewActions>(
    () => ({
      resize(size) {
        setWidth(size.width);
        inner.resize(size);
      },
      update(request) {
        inner.update(request);
      },
      zoomIn() {
        inner.zoomIn();
      },
      zoomOut() {
        inner.zoomOut();
      },
      fit() {
        inner.fit();
      },
      fitRows(range) {
        inner.fitRows(range);
      },
      panTo(leaf) {
        inner.panTo(leaf);
      },
      panBy(delta) {
        inner.panBy(delta);
      },
    }),
    [inner],
  );

  return { ...view, actions };
}
