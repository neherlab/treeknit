import type { ArgView, DrawTree, Elbow, PairView, Point } from "@neherlab/treeknit-wasm";

import type { RowRange } from "../canvas/viewState";
import { itemAt } from "./lookup";

export type TreeSide = "left" | "right";

export const TREE_SIDES: readonly TreeSide[] = ["left", "right"];

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

export function leafRows(
  nodes: readonly { children: readonly number[]; leaf: boolean; y: number }[],
  node: number,
): RowRange | null {
  return nodes[node] === undefined ? null : rowSpan(descendantLeaves(nodes, node).map((leaf) => leaf.y));
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
  const index = tree.nodes.findIndex((node) => node.leaf && node.name === name);

  return index === -1 ? undefined : index;
}

export function pairLeafRows(left: DrawTree, right: DrawTree, name: string): RowRange | null {
  return rowSpan(
    [left, right].flatMap((tree) => tree.nodes.flatMap((node) => (node.leaf && node.name === name ? [node.y] : []))),
  );
}

export function nodeIndex(tree: DrawTree, name: string): number | undefined {
  const index = tree.nodes.findIndex((node) => node.name === name);

  return index === -1 ? undefined : index;
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

export function argLeafRows(view: ArgView, name: string): RowRange | null {
  return rowSpan(view.nodes.flatMap((node) => (node.leaf && node.label === name ? [node.y] : [])));
}

export function rowCount(...trees: readonly { nodes: readonly { leaf: boolean }[] }[]): number {
  return Math.max(1, ...trees.map((tree) => tree.nodes.filter((node) => node.leaf).length));
}

function descendantLeaves<N extends { children: readonly number[]; leaf: boolean }>(
  nodes: readonly N[],
  root: number,
): N[] {
  const leaves: N[] = [];
  const seen = new Set<number>();
  const stack = [root];

  for (let next = stack.pop(); next !== undefined; next = stack.pop()) {
    if (seen.has(next)) {
      continue;
    }

    const node = itemAt(nodes, next, "node");

    seen.add(next);

    if (node.leaf) {
      leaves.push(node);
    }

    for (const child of node.children) {
      stack.push(child);
    }
  }

  return leaves;
}
