import type { PairView } from "@neherlab/treeknit-wasm";
import { createStore, type StoreApi } from "zustand/vanilla";

import type { TreeViewActions } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import { pairLeafRows } from "./trees";

export type FocusTarget = { kind: "leaf"; name: string } | { kind: "mcc"; mcc: number };

export interface FocusRequest {
  id: number;
  target: FocusTarget;
  pair: number | null;
}

export interface FocusState {
  request: FocusRequest | null;
  focus(target: FocusTarget, pair?: number | null): void;
  done(id: number): void;
}

export type FocusStore = StoreApi<FocusState>;

export function createFocusStore(): FocusStore {
  return createStore<FocusState>()((set, get) => ({
    request: null,
    focus(target, pair = null) {
      set({ request: { id: (get().request?.id ?? 0) + 1, target, pair } });
    },
    done(id) {
      if (get().request?.id === id) {
        set({ request: null });
      }
    },
  }));
}

export function focusApplies(request: Pick<FocusRequest, "pair">, shownPair: number): boolean {
  return request.pair === null || request.pair === shownPair;
}

export function focusRows(view: PairView, target: FocusTarget): RowRange | null {
  return target.kind === "leaf" ? pairLeafRows(view, target.name) : (view.mccs[target.mcc]?.rows ?? null);
}

export function revealLeafRows(actions: Pick<TreeViewActions, "panTo" | "fitRows">, range: RowRange): void {
  if (range.first === range.last) {
    actions.panTo(range.first);
  } else {
    actions.fitRows(range);
  }
}
