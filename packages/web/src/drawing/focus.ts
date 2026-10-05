import type { PairView } from "@neherlab/treeknit-wasm";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { RowRange } from "../canvas/viewState";
import { leafRow, rowSpan } from "./trees";

export type FocusTarget = { kind: "leaf"; name: string } | { kind: "mcc"; mcc: number };

export interface FocusRequest {
  id: number;
  target: FocusTarget;
}

interface FocusState {
  request: FocusRequest | null;
  focus(target: FocusTarget): void;
  done(id: number): void;
}

const focusStore = createStore<FocusState>()((set, get) => ({
  request: null,
  focus(target) {
    set({ request: { id: (get().request?.id ?? 0) + 1, target } });
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

export function requestFocus(target: FocusTarget): void {
  focusStore.getState().focus(target);
}

export function focusDone(id: number): void {
  focusStore.getState().done(id);
}

export function focusRows(view: PairView, target: FocusTarget): RowRange | null {
  if (target.kind === "leaf") {
    const row = leafRow(view.left, view.right, target.name);

    return row === undefined ? null : { first: row, last: row };
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
