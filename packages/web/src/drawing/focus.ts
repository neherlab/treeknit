import type { PairView } from "@neherlab/treeknit-wasm";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { TreeViewActions } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import { pairLeafRows, rowSpan } from "./trees";

export type FocusTarget = { kind: "leaf"; name: string } | { kind: "mcc"; mcc: number };

export interface FocusRequest {
  id: number;
  target: FocusTarget;
  pair: number | null;
}

interface FocusState {
  request: FocusRequest | null;
  focus(target: FocusTarget, pair: number | null): void;
  done(id: number): void;
}

const focusStore = createStore<FocusState>()((set, get) => ({
  request: null,
  focus(target, pair) {
    set({ request: { id: (get().request?.id ?? 0) + 1, target, pair } });
  },
  done(id) {
    if (get().request?.id === id) {
      set({ request: null });
    }
  },
}));

export function useFocusRequest(): FocusRequest | null {
  return useStore(focusStore, (state) => state.request);
}

export function requestFocus(target: FocusTarget, pair: number | null = null): void {
  focusStore.getState().focus(target, pair);
}

export function focusApplies(request: Pick<FocusRequest, "pair">, shownPair: number): boolean {
  return request.pair === null || request.pair === shownPair;
}

export function focusDone(id: number): void {
  focusStore.getState().done(id);
}

export function focusRows(view: PairView, target: FocusTarget): RowRange | null {
  if (target.kind === "leaf") {
    return pairLeafRows(view.left, view.right, target.name);
  }

  return mccRows(view, target.mcc);
}

export function mccRows(view: PairView, mcc: number): RowRange | null {
  return rowSpan(
    [view.left, view.right].flatMap((tree) =>
      tree.nodes.flatMap((node) => (node.leaf && node.mcc === mcc ? [node.y] : [])),
    ),
  );
}

export function revealLeafRows(actions: Pick<TreeViewActions, "panTo" | "fitRows">, range: RowRange): void {
  if (range.first === range.last) {
    actions.panTo(range.first);
  } else {
    actions.fitRows(range);
  }
}
