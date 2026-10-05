import type { ArgView, EdgePath } from "@neherlab/treeknit-wasm";

import {
  type Column,
  projectPath,
  projectPoint,
  sampleCubic,
  wangSegmentCount,
  type WorldPosition,
} from "../canvas/projection";
import { type LeafAxis, worldPosition } from "../canvas/viewState";
import { itemAt } from "../drawing/lookup";
import { type LayerPicks, pickTarget } from "../drawing/picking";
import type { ArgTarget } from "../drawing/selection";
import { DRAWING_MARGIN_PX } from "../drawing/spacing";
import { argNodePoints } from "../drawing/trees";

export interface EdgeItem {
  edge: number;
  segments: readonly number[];
  path: WorldPosition[];
}

export interface HybridItem {
  node: number;
  position: WorldPosition;
}

export interface ArgLeaderItem {
  node: number;
  path: WorldPosition[];
}

export interface ArgLabelItem {
  node: number;
  name: string;
  position: WorldPosition;
}

export interface ArgGeometry {
  edges: EdgeItem[];
  reticulations: EdgeItem[];
  hybrids: HybridItem[];
  leaders: ArgLeaderItem[];
  labels: ArgLabelItem[];
  nodes: (WorldPosition | undefined)[];
}

export const ARG_LAYER = {
  edges: "arg-edges",
  reticulations: "arg-reticulations",
  hybrids: "arg-hybrids",
  labels: "arg-labels",
} as const;

type DrawnEdge = ArgView["shapes"]["edges"][number];

type ArgLayerId = (typeof ARG_LAYER)[keyof typeof ARG_LAYER];

export function argColumn(crossPx: number, labelPx: number): Column {
  return {
    start: DRAWING_MARGIN_PX,
    end: Math.max(crossPx - DRAWING_MARGIN_PX - labelPx, DRAWING_MARGIN_PX),
    mirrored: false,
  };
}

export function argGeometry(view: ArgView, column: Column, leafAxis: LeafAxis, curveRowPx: number): ArgGeometry {
  const item = ({ edge, path }: DrawnEdge): EdgeItem => ({
    edge,
    segments: itemAt(view.edges, edge, "edge").segments,
    path: projectPath(edgePoints(path, column, curveRowPx), column, leafAxis),
  });

  const reticulation = (drawn: DrawnEdge) => itemAt(view.edges, drawn.edge, "edge").reticulation;

  return {
    edges: view.shapes.edges.flatMap((drawn) => (reticulation(drawn) ? [] : [item(drawn)])),
    reticulations: view.shapes.edges.flatMap((drawn) => (reticulation(drawn) ? [item(drawn)] : [])),
    hybrids: view.shapes.marks.flatMap(({ kind, node, at }) =>
      kind === "hybrid" ? [{ node, position: projectPoint(at, column, leafAxis) }] : [],
    ),
    leaders: view.shapes.leaders.map(({ node, from, to }) => ({
      node,
      path: projectPath([from, to], column, leafAxis),
    })),
    labels: view.nodes.flatMap((node, index) =>
      node.leaf ? [{ node: index, name: node.label, position: worldPosition(leafAxis, column.end, node.y) }] : [],
    ),
    nodes: argNodePoints(view).map((point) =>
      point === undefined ? undefined : projectPoint(point, column, leafAxis),
    ),
  };
}

export function argTargetAt(geometry: ArgGeometry, layer: string, index: number): ArgTarget | undefined {
  return pickTarget(ARG_PICKS, geometry, layer, index);
}

const ARG_PICKS: LayerPicks<ArgLayerId, ArgGeometry, ArgTarget> = {
  [ARG_LAYER.edges]: (geometry, index) => edgeTarget(geometry.edges[index]),
  [ARG_LAYER.reticulations]: (geometry, index) => edgeTarget(geometry.reticulations[index]),
  [ARG_LAYER.hybrids]: (geometry, index) => nodeTarget(geometry.hybrids[index]),
  [ARG_LAYER.labels]: (geometry, index) => nodeTarget(geometry.labels[index]),
};

function edgeTarget(item: EdgeItem | undefined): ArgTarget | undefined {
  return item === undefined ? undefined : { kind: "edge", edge: item.edge };
}

function nodeTarget(item: { node: number } | undefined): ArgTarget | undefined {
  return item === undefined ? undefined : { kind: "node", node: item.node };
}

function edgePoints(path: EdgePath, column: Column, curveRowPx: number) {
  return path.kind === "elbow"
    ? path.points
    : sampleCubic(path.curve, wangSegmentCount(path.curve, column, curveRowPx));
}
