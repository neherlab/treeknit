import type { AuspiceMccRoot, AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";
import { applyFilter, updateVisibleTipsAndBranchThicknesses } from "auspice/src/actions/tree";
import { toggleStreamTree } from "auspice/src/actions/treeStreams";
import { DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import { strainSymbol } from "auspice/src/util/globals";
import { getTraitFromNode } from "auspice/src/util/treeMiscHelpers";

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

const SHOWN_CLADE_MIN_TIPS = 20;

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

  if (controls.showStreamTrees) {
    store.dispatch(toggleStreamTree());
  }

  inactivateHidingFilters(store, shownLeaves(store.getState(), name));

  store.dispatch({
    type: SELECT_NODE,
    [FROM_WORKSPACE]: true,
    name,
    idx,
    isBranch: false,
    treeId: mainIdx === undefined ? "RIGHT" : "LEFT",
  });
  store.dispatch(applyFilter("add", strainSymbol, [name]));
  zoomOutToLeaf(store, name);

  return { name, existingFilter: existing === undefined ? null : existing.active ? "active" : "inactive" };
}

function inactivateHidingFilters(store: AuspiceStore, leaves: readonly AuspiceNode[]): void {
  const { filters } = store.getState().controls;

  for (const trait of Object.keys(filters)) {
    const active = (filters[trait] ?? []).filter((item) => item.active).map(({ value }) => value);
    const hides = leaves.some((leaf) => !active.includes(String(getTraitFromNode(leaf, trait))));

    if (active.length > 0 && hides) {
      store.dispatch(applyFilter("inactivate", trait, active));
    }
  }
}

function zoomOutToLeaf(store: AuspiceStore, name: string): void {
  const { tree, treeToo, controls } = store.getState();
  const main = cladeToShow(tree.nodes, tree.idxOfInViewRootNode, name);
  const second = isTreeTooShown(controls) ? cladeToShow(treeToo.nodes, treeToo.idxOfInViewRootNode, name) : undefined;

  if (main !== undefined || second !== undefined) {
    store.dispatch(updateVisibleTipsAndBranchThicknesses({ root: [main, second] }));
  }
}

function cladeToShow(
  nodes: readonly AuspiceNode[] | null | undefined,
  inViewRoot: number | undefined,
  name: string,
): number | undefined {
  const leaf = nodes?.find((node) => !node.hasChildren && node.name === name);

  if (leaf === undefined || inViewRoot === undefined || isWithin(leaf, inViewRoot)) {
    return undefined;
  }

  let clade = leaf.parent;

  while (clade.fullTipCount < SHOWN_CLADE_MIN_TIPS && clade.parent !== clade) {
    clade = clade.parent;
  }

  return clade.arrayIdx;
}

function isWithin(node: AuspiceNode, ancestorIdx: number): boolean {
  let current = node;

  while (current.arrayIdx !== ancestorIdx) {
    if (current.parent === current) {
      return false;
    }

    current = current.parent;
  }

  return true;
}

function shownLeaves(state: AuspiceState, name: string): AuspiceNode[] {
  const trees = isTreeTooShown(state.controls) ? [state.tree.nodes, state.treeToo.nodes] : [state.tree.nodes];

  return trees.flatMap((nodes) => nodes?.filter((node) => !node.hasChildren && node.name === name) ?? []);
}

function isTreeTooShown({ showTreeToo }: AuspiceState["controls"]): boolean {
  return showTreeToo !== false && showTreeToo !== undefined;
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
