import type { ArgView, PairView } from "@neherlab/treeknit-wasm";
import { omit } from "remeda";

import type { RowRange } from "../canvas/viewState";
import type { NodeRef, WorkspaceSearch } from "../workspace/search";
import { mccRows } from "./focus";
import { leafIndex, leafRows, nodeIndex, pairLeafRows, type TreeSide } from "./trees";

export const UNSELECTED_OPACITY = 0.25;

export interface Selection {
  mcc?: number;
  leaf?: string;
  node?: NodeRef;
}

export const NO_SELECTION: Selection = {};

export type PairTarget =
  | { kind: "node"; side: TreeSide; node: number }
  | { kind: "link"; link: number }
  | { kind: "ribbon"; block: number };

export type ArgTarget = { kind: "node"; node: number } | { kind: "edge"; edge: number };

export interface PairEmphasis {
  mcc: number | undefined;
  leaf: { left: number | undefined; right: number | undefined; link: number | undefined };
  node: { side: TreeSide; node: number } | undefined;
}

export interface ArgEmphasis {
  leaf: number | undefined;
  node: number | undefined;
}

export function selectionOf(search: WorkspaceSearch): Selection {
  return {
    ...(search.mcc === undefined ? undefined : { mcc: search.mcc }),
    ...(search.leaf === undefined ? undefined : { leaf: search.leaf }),
    ...(search.node === undefined ? undefined : { node: search.node }),
  };
}

export function withSelection(search: WorkspaceSearch, selection: Selection): WorkspaceSearch {
  return { ...omit(search, ["mcc", "leaf", "node"]), ...selection };
}

export function pairClickSelection(view: PairView, target: PairTarget | undefined): Selection {
  if (target === undefined) {
    return NO_SELECTION;
  }

  if (target.kind === "link") {
    const link = view.links[target.link];

    return link === undefined ? NO_SELECTION : { mcc: link.mcc };
  }

  if (target.kind === "ribbon") {
    const block = view.blocks[target.block];

    return block === undefined ? NO_SELECTION : { mcc: block.mcc };
  }

  const node = view[target.side].nodes[target.node];

  if (node === undefined) {
    return NO_SELECTION;
  }

  if (node.leaf) {
    return { leaf: node.name };
  }

  const mcc = node.mcc === null ? undefined : { mcc: node.mcc };

  return node.name === "" ? { ...mcc } : { ...mcc, node: { side: target.side, name: node.name } };
}

export function argClickSelection(view: ArgView, target: ArgTarget | undefined): Selection {
  if (target === undefined) {
    return NO_SELECTION;
  }

  const index = target.kind === "node" ? target.node : view.edges[target.edge]?.child;
  const node = index === undefined ? undefined : view.nodes[index];

  if (node === undefined) {
    return NO_SELECTION;
  }

  if (node.leaf && !node.hybrid) {
    return { leaf: node.label };
  }

  return node.label === "" ? NO_SELECTION : { node: { side: "arg", name: node.label } };
}

export function pairEmphasis(view: PairView, selection: Selection): PairEmphasis {
  const { leaf, node } = selection;
  const left = leaf === undefined ? undefined : leafIndex(view.left, leaf);
  const right = leaf === undefined ? undefined : leafIndex(view.right, leaf);
  const link = left === undefined ? -1 : view.links.findIndex((candidate) => candidate.left === left);
  const side = node?.side === "left" || node?.side === "right" ? node.side : undefined;
  const selectedNode = side === undefined || node === undefined ? undefined : nodeIndex(view[side], node.name);

  return {
    mcc: selection.mcc,
    leaf: { left, right, link: link === -1 ? undefined : link },
    node: side === undefined || selectedNode === undefined ? undefined : { side, node: selectedNode },
  };
}

export function argEmphasis(view: ArgView, selection: Selection): ArgEmphasis {
  const { leaf, node } = selection;

  const leafNode =
    leaf === undefined ? -1 : view.nodes.findIndex((candidate) => candidate.leaf && candidate.label === leaf);

  const argNode = node?.side === "arg" ? view.nodes.findIndex((candidate) => candidate.label === node.name) : -1;

  return { leaf: leafNode === -1 ? undefined : leafNode, node: argNode === -1 ? undefined : argNode };
}

export function pairSelectionRows(view: PairView, selection: Selection): RowRange | null {
  const { node } = pairEmphasis(view, selection);

  if (node !== undefined) {
    return leafRows(view[node.side].nodes, node.node);
  }

  const rows = selection.leaf === undefined ? null : pairLeafRows(view.left, view.right, selection.leaf);

  if (rows !== null) {
    return rows;
  }

  return selection.mcc === undefined ? null : mccRows(view, selection.mcc);
}

export function argSelectionRows(view: ArgView, selection: Selection): RowRange | null {
  const { node, leaf } = argEmphasis(view, selection);
  const selected = node ?? leaf;

  return selected === undefined ? null : leafRows(view.nodes, selected);
}

export function emphasisOpacity(mcc: number | null, selected: number | undefined): number {
  return selected === undefined || mcc === selected ? 1 : UNSELECTED_OPACITY;
}
