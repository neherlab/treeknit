import type { ColorRole, Elbow, Leader, Mark, PairView, Point } from "@neherlab/treeknit-wasm";

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
import { itemAt } from "../drawing/lookup";
import { type LayerPicks, pickTarget } from "../drawing/picking";
import type { PairTarget } from "../drawing/selection";
import { type TreeSide, treeNodePoints } from "../drawing/trees";
import type { TanglegramColumns } from "./columns";

export interface BranchItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  slot: number | null;
  color: ColorRole;
  path: WorldPosition[];
}

export interface MarkItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  slot: number | null;
  color: ColorRole;
  position: WorldPosition;
}

export interface LeaderItem {
  side: TreeSide;
  node: number;
  mcc: number | null;
  path: WorldPosition[];
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
  text: string;
  position: WorldPosition;
}

export interface TanglegramGeometry {
  branches: { plain: BranchItem[]; added: BranchItem[]; reassortment: BranchItem[] };
  marks: { reassortment: MarkItem[]; imputed: MarkItem[] };
  leaders: LeaderItem[];
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
  leaders: "leaders",
  reassortmentMarks: "marks-reassortment",
  imputedMarks: "marks-imputed",
  leftLabels: "labels-left",
  rightLabels: "labels-right",
} as const;

export type PairLayerId = (typeof PAIR_LAYER)[keyof typeof PAIR_LAYER];

export type TanglegramCurves = Pick<TanglegramGeometry, "links" | "ribbons">;

export function tanglegramFrame(
  view: PairView,
  columns: TanglegramColumns,
  leafAxis: LeafAxis,
): Omit<TanglegramGeometry, keyof TanglegramCurves> {
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
    leaders: [...left.leaders, ...right.leaders],
    labels: {
      left: labelItems(view, "left", columns.leftLabels.start, leafAxis),
      right: labelItems(view, "right", columns.rightLabels.end, leafAxis),
    },
    nodes: { left: left.nodes, right: right.nodes },
  };
}

export function tanglegramCurves(
  view: PairView,
  columns: TanglegramColumns,
  leafAxis: LeafAxis,
  curveRowPx: number,
): TanglegramCurves {
  return {
    links: view.shapes.links.map(({ link, mcc, slot, curve }) => ({
      link,
      mcc,
      slot,
      path: projectPath(
        sampleCubic(curve, wangSegmentCount(curve, columns.links, curveRowPx)),
        columns.links,
        leafAxis,
      ),
    })),
    ribbons: view.shapes.ribbons.map(({ block, mcc, slot, outline }) => ({
      block,
      mcc,
      slot,
      polygon: projectPath(sampleCubicChain(outline, columns.links, curveRowPx), columns.links, leafAxis),
    })),
  };
}

export function pairTargetAt(geometry: TanglegramGeometry, layer: string, index: number): PairTarget | undefined {
  return pickTarget(PAIR_PICKS, geometry, layer, index);
}

const PAIR_PICKS: LayerPicks<PairLayerId, TanglegramGeometry, PairTarget> = {
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
  [PAIR_LAYER.leaders]: () => undefined,
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
  const { elbows, marks, leaders } = view.shapes[side];

  const branch = (
    elbow: Elbow,
    points: readonly Point[] = elbow.points,
    color: ColorRole = elbow.color,
  ): BranchItem => ({
    side,
    node: elbow.node,
    mcc: elbow.mcc,
    slot: elbow.slot,
    color,
    path: projectPath(points, column, leafAxis),
  });

  const mark = ({ node, mcc, slot, color, at }: Mark): MarkItem => ({
    side,
    node,
    mcc,
    slot,
    color,
    position: projectPoint(at, column, leafAxis),
  });

  const plain = (elbow: Elbow) => (elbow.added ? branch(elbow, elbow.points.slice(0, 2)) : branch(elbow));
  const marksOf = (kind: Mark["kind"]) => marks.flatMap((item) => (item.kind === kind ? [mark(item)] : []));

  return {
    branches: {
      plain: elbows.flatMap((elbow) => (elbow.mccBreak ? [] : [plain(elbow)])),
      added: elbows.flatMap((elbow) =>
        !elbow.mccBreak && elbow.added ? [branch(elbow, elbow.points.slice(1), "inkMuted")] : [],
      ),
      reassortment: elbows.flatMap((elbow) => (elbow.mccBreak ? [branch(elbow)] : [])),
    },
    marks: { reassortment: marksOf("reassortment"), imputed: marksOf("imputed") },
    leaders: leaders.map(({ node, from, to }: Leader): LeaderItem => ({
      side,
      node,
      mcc: itemAt(tree.nodes, node, "node").mcc,
      path: projectPath([from, to], column, leafAxis),
    })),
    nodes: treeNodePoints(tree, elbows).map((point) =>
      point === undefined ? undefined : projectPoint(point, column, leafAxis),
    ),
  };
}

function labelItems(view: PairView, side: TreeSide, crossPx: number, leafAxis: LeafAxis): LabelItem[] {
  return view[side].nodes.flatMap((node, index) =>
    node.leaf
      ? [{ side, node: index, mcc: node.mcc, text: node.shortName, position: worldPosition(leafAxis, crossPx, node.y) }]
      : [],
  );
}
