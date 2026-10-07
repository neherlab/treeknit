import type {
  ArgNodeView,
  ArgView,
  ConstellationCell,
  ConstellationTable,
  DrawNode,
  MccInfo,
  PairView,
} from "@neherlab/treeknit-wasm";

import { itemAt } from "../drawing/lookup";
import type { Selection } from "../drawing/selection";
import { internalNodeIndex, leafIndex, TREE_SIDES, type TreeSide } from "../drawing/trees";

export interface InspectorData {
  pair: PairView | undefined;
  arg: ArgView | null | undefined;
  constellation: ConstellationTable | undefined;
}

export interface LeafCopy {
  side: TreeSide;
  tree: string;
  node: DrawNode;
}

export interface LeafPair {
  pair: number;
  title: string;
  cell: NamedCell | null;
}

export interface NamedCell extends ConstellationCell {
  name: string;
}

export type InspectorSubject =
  | { kind: "none"; mccs: readonly MccInfo[] }
  | { kind: "leaf"; name: string; copies: LeafCopy[]; mcc: MccInfo | undefined; ambiguous: boolean; pairs: LeafPair[] }
  | { kind: "mcc"; mcc: MccInfo }
  | { kind: "node"; side: TreeSide; tree: string; node: DrawNode; mcc: MccInfo | undefined }
  | { kind: "argNode"; node: ArgNodeView; segments: readonly [string, string] };

export function inspectorParent(subject: InspectorSubject): MccInfo | null {
  return (subject.kind === "leaf" || subject.kind === "node") && subject.mcc !== undefined ? subject.mcc : null;
}

export function inspectorSubject(selection: Selection, data: InspectorData): InspectorSubject {
  const { pair, arg } = data;
  const { node, leaf, mcc } = selection;

  if (node?.side === "arg") {
    const argNode = arg?.nodes.find((candidate) => candidate.label === node.name);

    if (arg !== undefined && arg !== null && argNode !== undefined) {
      return { kind: "argNode", node: argNode, segments: arg.segments };
    }
  } else if (node !== undefined && pair !== undefined) {
    const tree = pair[node.side];
    const index = internalNodeIndex(tree, node.name);
    const drawn = index === undefined ? undefined : tree.nodes[index];

    if (index !== undefined && drawn !== undefined) {
      return {
        kind: "node",
        side: node.side,
        tree: tree.label,
        node: drawn,
        mcc: drawn.mcc === null ? undefined : pair.mccs[drawn.mcc],
      };
    }
  }

  if (leaf !== undefined) {
    return leafSubject(leaf, data);
  }

  const info = mcc === undefined ? undefined : pair?.mccs[mcc];

  return info === undefined ? { kind: "none", mccs: pair?.mccs ?? [] } : { kind: "mcc", mcc: info };
}

export function leafPairs(constellation: ConstellationTable | undefined, name: string): LeafPair[] {
  const row = constellation?.leaves.indexOf(name) ?? -1;
  const cells = row === -1 ? undefined : constellation?.cells[row];

  return (constellation?.pairTitles ?? []).map((title, pair) => {
    const cell = cells?.[pair] ?? null;

    return {
      pair,
      title,
      cell: cell === null ? null : { ...cell, name: itemAt(constellation?.mccNames ?? [], cell.mcc, "MCC name") },
    };
  });
}

function leafSubject(name: string, { pair, constellation }: InspectorData): InspectorSubject {
  const copies = TREE_SIDES.flatMap((side): LeafCopy[] => {
    const tree = pair?.[side];
    const index = tree === undefined ? undefined : leafIndex(tree, name);
    const node = index === undefined ? undefined : tree?.nodes[index];

    return tree === undefined || node === undefined ? [] : [{ side, tree: tree.label, node }];
  });

  const mccIndex = copies.find(({ node }) => node.mcc !== null)?.node.mcc ?? undefined;
  const mcc = mccIndex === undefined ? undefined : pair?.mccs[mccIndex];

  return {
    kind: "leaf",
    name,
    copies,
    mcc,
    ambiguous: mcc?.ambiguousLeaves.includes(name) ?? false,
    pairs: leafPairs(constellation, name),
  };
}
