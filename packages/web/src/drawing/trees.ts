import type { ArgView, DrawTree, Elbow, PairView, Point, RowSpan } from "@neherlab/treeknit-wasm";

import type { RowRange } from "../canvas/viewState";
import { itemAt } from "./lookup";

export type TreeSide = "left" | "right";

export const TREE_SIDES: readonly TreeSide[] = ["left", "right"];

const NAMED_NODES = new WeakMap<DrawTree, NamedNodes>();

export function treeNodePoints(tree: DrawTree, elbows: readonly Elbow[]): (Point | undefined)[] {
  const points: (Point | undefined)[] = tree.nodes.map(() => undefined);

  for (const { node, points: path } of elbows) {
    const { parent } = itemAt(tree.nodes, node, "node");

    points[node] = path[2];

    if (parent !== null) {
      points[parent] ??= path[0];
    }
  }

  return points;
}

export function argNodePoints(view: ArgView): (Point | undefined)[] {
  const points: (Point | undefined)[] = view.nodes.map(() => undefined);

  for (const { edge, path } of view.shapes.edges) {
    const ends = itemAt(view.edges, edge, "edge");
    const [from, to] = path.kind === "elbow" ? [path.points[0], path.points[2]] : [path.curve.from, path.curve.to];

    points[ends.child] = to;
    points[ends.parent] ??= from;
  }

  return points;
}

export function nodeRows(...nodes: readonly ({ rows: RowSpan } | undefined)[]): RowRange | null {
  return rowSpan(nodes.flatMap((node) => (node === undefined ? [] : [node.rows.first, node.rows.last])));
}

export function rowSpan(rows: Iterable<number>): RowRange | null {
  let first = Number.POSITIVE_INFINITY;
  let last = Number.NEGATIVE_INFINITY;

  for (const row of rows) {
    first = Math.min(first, row);
    last = Math.max(last, row);
  }

  return first > last ? null : { first, last };
}

export function leafIndex(tree: DrawTree, name: string): number | undefined {
  return namedNodes(tree).leaves.get(name);
}

export function pairLeafRows(view: Pick<PairView, "left" | "right">, name: string): RowRange | null {
  return nodeRows(
    ...TREE_SIDES.map((side) => {
      const index = leafIndex(view[side], name);

      return index === undefined ? undefined : view[side].nodes[index];
    }),
  );
}

export function internalNodeIndex(tree: DrawTree, name: string): number | undefined {
  return namedNodes(tree).internal.get(name);
}

export function leafNames(tree: DrawTree): string[] {
  return tree.nodes.filter((node) => node.leaf).map((node) => node.name);
}

export function pairLeafNames(view: Pick<PairView, "left" | "right">): string[] {
  return [...new Set([...leafNames(view.left), ...leafNames(view.right)])];
}

export function argLeafNames(view: ArgView): string[] {
  return view.nodes.flatMap((node) => (node.leaf ? [node.label] : []));
}

export function pairLeafLabels(view: Pick<PairView, "left" | "right">): string[] {
  return [
    ...new Set(
      [view.left, view.right].flatMap((tree) => tree.nodes.flatMap((node) => (node.leaf ? [node.shortName] : []))),
    ),
  ];
}

export function argLeafLabels(view: ArgView): string[] {
  return view.nodes.flatMap((node) => (node.leaf ? [node.shortLabel] : []));
}

export function argLeafRows(view: ArgView, name: string): RowRange | null {
  return nodeRows(view.nodes.find((node) => node.leaf && node.label === name));
}

export function rowCount(...trees: readonly { nodes: readonly { leaf: boolean }[] }[]): number {
  return Math.max(1, ...trees.map((tree) => tree.nodes.filter((node) => node.leaf).length));
}

function namedNodes(tree: DrawTree): NamedNodes {
  const known = NAMED_NODES.get(tree);

  if (known !== undefined) {
    return known;
  }

  const named: NamedNodes = { leaves: new Map(), internal: new Map() };

  tree.nodes.forEach((node, index) => {
    const names = node.leaf ? named.leaves : named.internal;

    if (!names.has(node.name)) {
      names.set(node.name, index);
    }
  });
  NAMED_NODES.set(tree, named);

  return named;
}

interface NamedNodes {
  leaves: Map<string, number>;
  internal: Map<string, number>;
}
