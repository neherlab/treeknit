import type { LabelMode, Scale } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";

import type { TreeViewActions } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import type { WorkspaceSearch } from "../workspace/search";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { selectionOf, type Selection, withSelection } from "./selection";
import { rowCenter } from "./trees";

export interface DrawingSearch {
  search: WorkspaceSearch;
  update: (change: (written: WorkspaceSearch) => WorkspaceSearch) => void;
  selection: Selection;
  select: (next: Selection) => void;
  clear: () => void;
  chooseScale: (next: Scale) => void;
  chooseLabels: (next: LabelMode) => void;
}

export function useDrawingSearch(): DrawingSearch {
  const { search, update } = useWorkspaceSearch();
  const selection = useMemo(() => selectionOf(search), [search]);

  const select = useCallback(
    (next: Selection) => {
      update((written) => withSelection(written, next));
    },
    [update],
  );

  const clear = useCallback(() => {
    select({});
  }, [select]);

  const chooseScale = useCallback(
    (next: Scale) => {
      update((written) => ({ ...written, x: next }));
    },
    [update],
  );

  const chooseLabels = useCallback(
    (next: LabelMode) => {
      update((written) => ({ ...written, labels: next }));
    },
    [update],
  );

  return { search, update, selection, select, clear, chooseScale, chooseLabels };
}

export function useFindLeaf<D>(
  data: D | undefined,
  rowsOf: (data: D, name: string) => RowRange | null,
  select: (next: Selection) => void,
  actions: Pick<TreeViewActions, "panTo">,
): (name: string) => void {
  return useCallback(
    (name: string) => {
      select({ leaf: name });

      const rows = data === undefined ? null : rowsOf(data, name);

      if (rows !== null) {
        actions.panTo(rowCenter(rows));
      }
    },
    [data, rowsOf, select, actions],
  );
}
