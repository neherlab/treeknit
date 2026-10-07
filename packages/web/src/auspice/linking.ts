import type { AuspiceMccRoot, AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";
import { applyFilter, updateVisibleTipsAndBranchThicknesses } from "auspice/src/actions/tree";
import { DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import { strainSymbol } from "auspice/src/util/globals";

import type { Selection } from "../drawing/selection";
import { FROM_WORKSPACE } from "./selection";
import type { AuspiceNode, AuspiceState } from "./state";
import type { AuspiceStore } from "./store";

export interface AppliedMarks {
  leaf: MarkedLeaf | null;
  mcc: string | null;
}

export interface MarkedLeaf {
  name: string;
  existingFilter: "active" | "inactive" | null;
}

export const NO_MARKS: AppliedMarks = { leaf: null, mcc: null };

export function syncSelection(
  store: AuspiceStore,
  applied: AppliedMarks,
  selection: Selection,
  mccFilter: MccFilter,
): AppliedMarks {
  const wantedLeaf = selection.leaf ?? null;

  const wantedMcc = markedMccValue(selection, mccFilter);
  const key = mccFilter.mcc_key;

  let { leaf, mcc } = applied;

  if (wantedLeaf !== (leaf?.name ?? null)) {
    if (leaf !== null) {
      unmarkLeaf(store, leaf);
    }

    leaf = wantedLeaf === null ? null : markLeaf(store, wantedLeaf);
  }

  if (wantedMcc !== mcc) {
    if (mcc !== null && hasFilterValue(store.getState(), key, mcc)) {
      store.dispatch(applyFilter("remove", key, [mcc]));
    }

    if (wantedMcc !== null) {
      store.dispatch(applyFilter("set", key, [wantedMcc]));
    }

    mcc = wantedMcc;
  }

  return { leaf, mcc };
}

export function markedMccValue(selection: Selection, { mcc_values }: MccFilter): string | null {
  return selection.leaf === undefined && selection.mcc !== undefined ? (mcc_values[selection.mcc] ?? null) : null;
}

export type MccFilter = Pick<AuspicePair, "mcc_key" | "mcc_values">;

export function zoomToMcc(store: AuspiceStore, root: AuspiceMccRoot, trees: AuspiceTrees): void {
  const { tree, treeToo } = store.getState();
  const [main, second] = trees === "right" ? [root.right, root.left] : [root.left, root.right];
  const mainIdx = nodeIndex(tree.nodes, main);
  const secondIdx = trees === "both" ? nodeIndex(treeToo.nodes, second) : undefined;

  if (mainIdx !== undefined || secondIdx !== undefined) {
    store.dispatch(updateVisibleTipsAndBranchThicknesses({ root: [mainIdx, secondIdx] }));
  }
}

function markLeaf(store: AuspiceStore, name: string): MarkedLeaf | null {
  const { controls, tree, treeToo } = store.getState();
  const selected = controls.selectedNode;

  if (selected?.name === name) {
    return { name, existingFilter: selected.existingFilterState ?? null };
  }

  const mainIdx = leafIndex(tree.nodes, name);
  const secondIdx = mainIdx === undefined ? leafIndex(treeToo.nodes, name) : undefined;
  const idx = mainIdx ?? secondIdx;

  if (idx === undefined) {
    return null;
  }

  const existing = controls.filters[strainSymbol]?.find(({ value }) => value === name);

  store.dispatch({
    type: SELECT_NODE,
    [FROM_WORKSPACE]: true,
    name,
    idx,
    isBranch: false,
    treeId: mainIdx === undefined ? "RIGHT" : "LEFT",
  });
  store.dispatch(applyFilter("add", strainSymbol, [name]));

  return { name, existingFilter: existing === undefined ? null : existing.active ? "active" : "inactive" };
}

function unmarkLeaf(store: AuspiceStore, { name, existingFilter }: MarkedLeaf): void {
  const state = store.getState();

  if (existingFilter === null && hasFilterValue(state, strainSymbol, name)) {
    store.dispatch(applyFilter("remove", strainSymbol, [name]));
  } else if (existingFilter === "inactive" && hasFilterValue(state, strainSymbol, name)) {
    store.dispatch(applyFilter("inactivate", strainSymbol, [name]));
  }

  if (store.getState().controls.selectedNode?.name === name) {
    store.dispatch({ type: DESELECT_NODE, [FROM_WORKSPACE]: true });
  }
}

function hasFilterValue(state: AuspiceState, key: string | symbol, value: string): boolean {
  return state.controls.filters[key]?.some((item) => item.value === value) ?? false;
}

function leafIndex(nodes: readonly AuspiceNode[] | null | undefined, name: string): number | undefined {
  return nodes?.find((node) => !node.hasChildren && node.name === name)?.arrayIdx;
}

function nodeIndex(nodes: readonly AuspiceNode[] | null | undefined, name: string | null): number | undefined {
  return name === null ? undefined : nodes?.find((node) => node.name === name)?.arrayIdx;
}
