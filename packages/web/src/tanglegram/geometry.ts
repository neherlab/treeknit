import type { Elbow, Mark, PairView } from "@neherlab/treeknit-wasm";

import {
  type Column,
  projectPath,
  projectPoint,
  sampleCubic,
  sampleCubicChain,
  wangSegmentCount,
  type WorldPosition,
} from "../canvas/projection";
import { type LeafAxis, worldPosition } from "../canvas/viewState";
import type { PairTarget } from "../drawing/selection";
import { type TreeSide, treeNodePoints } from "../drawing/trees";
import type { TanglegramColumns } from "./columns";

export interface BranchItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  slot: number | null;
  path: WorldPosition[];
}

export interface MarkItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  slot: number | null;
  position: WorldPosition;
}

export interface LinkItem {
  link: number;
  mcc: number;
  slot: number;
  path: WorldPosition[];
}

export interface RibbonItem {
  block: number;
  mcc: number;
  slot: number;
  polygon: WorldPosition[];
}

export interface LabelItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  name: string;
  position: WorldPosition;
}

export interface TanglegramGeometry {
  branches: { plain: BranchItem[]; added: BranchItem[]; reassortment: BranchItem[] };
  marks: { reassortment: MarkItem[]; imputed: MarkItem[] };
  links: LinkItem[];
  ribbons: RibbonItem[];
  labels: { left: LabelItem[]; right: LabelItem[] };
  nodes: { left: (WorldPosition | undefined)[]; right: (WorldPosition | undefined)[] };
}

export const PAIR_LAYER = {
  ribbons: "ribbons",
  links: "links",
  plainBranches: "branches-plain",
  addedBranches: "branches-added",
  reassortmentBranches: "branches-reassortment",
  reassortmentMarks: "marks-reassortment",
  imputedMarks: "marks-imputed",
  leftLabels: "labels-left",
  rightLabels: "labels-right",
} as const;

export type PairLayerId = (typeof PAIR_LAYER)[keyof typeof PAIR_LAYER];

export function tanglegramGeometry(view: PairView, columns: TanglegramColumns, leafAxis: LeafAxis): TanglegramGeometry {
  const left = treeGeometry(view, "left", columns.left, leafAxis);
  const right = treeGeometry(view, "right", columns.right, leafAxis);

  return {
    branches: {
      plain: [...left.branches.plain, ...right.branches.plain],
      added: [...left.branches.added, ...right.branches.added],
      reassortment: [...left.branches.reassortment, ...right.branches.reassortment],
    },
    marks: {
      reassortment: [...left.marks.reassortment, ...right.marks.reassortment],
      imputed: [...left.marks.imputed, ...right.marks.imputed],
    },
    links: view.shapes.links.map(({ link, slot, curve }) => ({
      link,
      mcc: view.links[link]?.mcc ?? 0,
      slot,
      path: projectPath(sampleCubic(curve, wangSegmentCount(curve, columns.links)), columns.links, leafAxis),
    })),
    ribbons: view.shapes.ribbons.map(({ block, slot, outline }) => ({
      block,
      mcc: view.blocks[block]?.mcc ?? 0,
      slot,
      polygon: projectPath(sampleCubicChain(outline, columns.links), columns.links, leafAxis),
    })),
    labels: {
      left: labelItems(view, "left", columns.leftLabels.start, leafAxis),
      right: labelItems(view, "right", columns.rightLabels.end, leafAxis),
    },
    nodes: { left: left.nodes, right: right.nodes },
  };
}

export function pairTargetAt(geometry: TanglegramGeometry, layer: string, index: number): PairTarget | undefined {
  return isPairLayer(layer) ? PAIR_PICKS[layer](geometry, index) : undefined;
}

function isPairLayer(layer: string): layer is PairLayerId {
  return Object.values<string>(PAIR_LAYER).includes(layer);
}

const PAIR_PICKS: Record<PairLayerId, (geometry: TanglegramGeometry, index: number) => PairTarget | undefined> = {
  [PAIR_LAYER.ribbons]: (geometry, index) => {
    const item = geometry.ribbons[index];

    return item === undefined ? undefined : { kind: "ribbon", block: item.block };
  },
  [PAIR_LAYER.links]: (geometry, index) => {
    const item = geometry.links[index];

    return item === undefined ? undefined : { kind: "link", link: item.link };
  },
  [PAIR_LAYER.plainBranches]: (geometry, index) => nodeTarget(geometry.branches.plain[index]),
  [PAIR_LAYER.addedBranches]: (geometry, index) => nodeTarget(geometry.branches.added[index]),
  [PAIR_LAYER.reassortmentBranches]: (geometry, index) => nodeTarget(geometry.branches.reassortment[index]),
  [PAIR_LAYER.reassortmentMarks]: (geometry, index) => nodeTarget(geometry.marks.reassortment[index]),
  [PAIR_LAYER.imputedMarks]: (geometry, index) => nodeTarget(geometry.marks.imputed[index]),
  [PAIR_LAYER.leftLabels]: (geometry, index) => nodeTarget(geometry.labels.left[index]),
  [PAIR_LAYER.rightLabels]: (geometry, index) => nodeTarget(geometry.labels.right[index]),
};

function nodeTarget(item: { side: TreeSide; node: number } | undefined): PairTarget | undefined {
  return item === undefined ? undefined : { kind: "node", side: item.side, node: item.node };
}

function treeGeometry(view: PairView, side: TreeSide, column: Column, leafAxis: LeafAxis) {
  const tree = view[side];
  const { elbows, marks } = view.shapes[side];

  const branch = (elbow: Elbow): BranchItem => ({
    side,
    node: elbow.node,
    mcc: tree.nodes[elbow.node]?.mcc ?? null,
    slot: elbow.slot,
    path: projectPath(elbow.points, column, leafAxis),
  });

  const mark = ({ node, at }: Mark): MarkItem => {
    const mcc = tree.nodes[node]?.mcc ?? null;

    return {
      side,
      node,
      mcc,
      slot: mcc === null ? null : (view.mccs[mcc]?.slot ?? null),
      position: projectPoint(at, column, leafAxis),
    };
  };

  const branches = (keep: (elbow: Elbow) => boolean) => elbows.flatMap((elbow) => (keep(elbow) ? [branch(elbow)] : []));
  const marksOf = (kind: Mark["kind"]) => marks.flatMap((item) => (item.kind === kind ? [mark(item)] : []));

  return {
    branches: {
      plain: branches((elbow) => !elbow.mccBreak && !elbow.added),
      added: branches((elbow) => !elbow.mccBreak && elbow.added),
      reassortment: branches((elbow) => elbow.mccBreak),
    },
    marks: { reassortment: marksOf("reassortment"), imputed: marksOf("imputed") },
    nodes: treeNodePoints(tree, elbows).map((point) =>
      point === undefined ? undefined : projectPoint(point, column, leafAxis),
    ),
  };
}

function labelItems(view: PairView, side: TreeSide, crossPx: number, leafAxis: LeafAxis): LabelItem[] {
  return view[side].nodes.flatMap((node, index) =>
    node.leaf
      ? [{ side, node: index, mcc: node.mcc, name: node.name, position: worldPosition(leafAxis, crossPx, node.y) }]
      : [],
  );
}
