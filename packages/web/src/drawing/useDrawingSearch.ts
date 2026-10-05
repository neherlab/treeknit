import type { LabelMode, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";

import type { TreeViewActions } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import { selectPair, type WorkspaceSearch } from "../workspace/search";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { revealLeafRows } from "./focus";
import { hasSelection, selectionOf, type Selection, withSelection } from "./selection";

export interface DrawingSearch {
  search: WorkspaceSearch;
  update: (change: (written: WorkspaceSearch) => WorkspaceSearch) => void;
  selection: Selection;
  select: (next: Selection) => void;
  clear: () => void;
  clearOnEscape: (() => void) | null;
  choosePair: (next: number) => void;
  chooseVersion: (next: TreeVersion) => void;
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

  const choosePair = useCallback(
    (next: number) => {
      update((written) => selectPair(written, next));
    },
    [update],
  );

  const chooseVersion = useCallback(
    (next: TreeVersion) => {
      update((written) => ({ ...written, version: next }));
    },
    [update],
  );

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

  const clearOnEscape = hasSelection(selection) ? clear : null;

  return {
    search,
    update,
    selection,
    select,
    clear,
    clearOnEscape,
    choosePair,
    chooseVersion,
    chooseScale,
    chooseLabels,
  };
}

export function useFindLeaf<D>(
  data: D | undefined,
  rowsOf: (data: D, name: string) => RowRange | null,
  select: (next: Selection) => void,
  actions: Pick<TreeViewActions, "panTo" | "fitRows">,
): (name: string) => void {
  return useCallback(
    (name: string) => {
      select({ leaf: name });

      const rows = data === undefined ? null : rowsOf(data, name);

      if (rows !== null) {
        revealLeafRows(actions, rows);
      }
    },
    [data, rowsOf, select, actions],
  );
}
