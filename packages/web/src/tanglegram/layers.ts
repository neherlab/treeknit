import type { LayersList } from "@deck.gl/core";

import { type Rgba, withOpacity } from "../canvas/color";
import { type DrawingColors, mccColor } from "../canvas/drawingColors";
import { shortenLabel } from "../canvas/labels";
import { fillLayer } from "../canvas/layers/fillLayer";
import { labelLayer } from "../canvas/layers/labelLayer";
import { leaderLayer } from "../canvas/layers/leaderLayer";
import { branchLayer, hoverColor, REASSORTMENT_WIDTH_PX, ringLayer, selectionLayer } from "../canvas/layers/treeLayers";
import { emphasisOpacity, type PairEmphasis } from "../drawing/selection";
import { LABEL_GAP_PX } from "../drawing/spacing";
import { type BranchItem, type LinkItem, PAIR_LAYER, type TanglegramGeometry } from "./geometry";

export const RIBBON_OPACITY = 0.55;

export const SELECTED_LINK_WIDTH_PX = 2.5;

export const LINK_WIDTH_PX = 1;

export const LEADER_LAYER = "leaders";

export type BranchKind = "plain" | "added" | "reassortment";

export interface PairStyle {
  colors: DrawingColors;
  colorByMcc: boolean;
  emphasis: PairEmphasis;
  ribbons: boolean;
  labels: boolean;
  fontReady: boolean;
  fade: number;
}

export function branchColor(item: Pick<BranchItem, "mcc" | "slot">, kind: BranchKind, style: PairStyle): Rgba {
  const base = baseBranchColor(item.slot, kind, style);

  return withOpacity(base, emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function linkColor(item: Pick<LinkItem, "mcc" | "slot">, style: PairStyle): Rgba {
  return withOpacity(mccColor(style.colors, item.slot), emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function ribbonColor(item: Pick<LinkItem, "mcc" | "slot">, style: PairStyle): Rgba {
  return withOpacity(mccColor(style.colors, item.slot), RIBBON_OPACITY * emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function labelColor(mcc: number | null, style: PairStyle): Rgba {
  return withOpacity(style.colors.ink, emphasisOpacity(mcc, style.emphasis.mcc));
}

export function selectionPositions(geometry: TanglegramGeometry, emphasis: PairEmphasis) {
  const leaf = [
    emphasis.leaf.left === undefined ? undefined : geometry.nodes.left[emphasis.leaf.left],
    emphasis.leaf.right === undefined ? undefined : geometry.nodes.right[emphasis.leaf.right],
  ];

  const node = emphasis.node === undefined ? undefined : geometry.nodes[emphasis.node.side][emphasis.node.node];

  return [...leaf, node].filter((position) => position !== undefined);
}

export function tanglegramLayers(geometry: TanglegramGeometry, style: PairStyle): LayersList {
  const { colors, emphasis } = style;
  const triggers = [colors, style.colorByMcc, emphasis.mcc];
  const branch = (kind: BranchKind) => (item: BranchItem) => branchColor(item, kind, style);
  const selectedLink = geometry.links.find((item) => item.link === emphasis.leaf.link);

  return [
    fillLayer({
      id: PAIR_LAYER.ribbons,
      data: geometry.ribbons,
      getPolygon: (item) => item.polygon,
      getFillColor: (item) => ribbonColor(item, style),
      visible: style.ribbons,
      opacity: style.fade,
      highlightColor: hoverColor(colors),
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.links,
      data: geometry.links,
      getPath: (item) => item.path,
      getColor: (item) => linkColor(item, style),
      colors,
      widthPx: LINK_WIDTH_PX,
      visible: !style.ribbons,
      opacity: style.fade,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.plainBranches,
      data: geometry.branches.plain,
      getPath: (item) => item.path,
      getColor: branch("plain"),
      colors,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.addedBranches,
      data: geometry.branches.added,
      getPath: (item) => item.path,
      getColor: branch("added"),
      colors,
      dashed: true,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.reassortmentBranches,
      data: geometry.branches.reassortment,
      getPath: (item) => item.path,
      getColor: branch("reassortment"),
      colors,
      widthPx: REASSORTMENT_WIDTH_PX,
      colorTriggers: triggers,
    }),
    leaderLayer({ id: LEADER_LAYER, data: geometry.leaders, colors, visible: style.labels }),
    ringLayer({
      id: PAIR_LAYER.imputedMarks,
      data: geometry.marks.imputed,
      getPosition: (item) => item.position,
      getLineColor: (item) =>
        withOpacity(baseBranchColor(item.slot, "plain", style), emphasisOpacity(item.mcc, emphasis.mcc)),
      colors,
      colorTriggers: triggers,
    }),
    ringLayer({
      id: PAIR_LAYER.reassortmentMarks,
      data: geometry.marks.reassortment,
      getPosition: (item) => item.position,
      getLineColor: (item) => withOpacity(colors.signal, emphasisOpacity(item.mcc, emphasis.mcc)),
      colors,
      colorTriggers: triggers,
    }),
    labelLayer({
      id: PAIR_LAYER.leftLabels,
      data: geometry.labels.left,
      fontReady: style.fontReady,
      visible: style.labels,
      getPosition: (item) => item.position,
      getText: (item) => shortenLabel(item.name),
      getColor: (item) => labelColor(item.mcc, style),
      anchor: "start",
      offsetPx: [LABEL_GAP_PX, 0],
      pickable: true,
      colorTriggers: triggers,
    }),
    labelLayer({
      id: PAIR_LAYER.rightLabels,
      data: geometry.labels.right,
      fontReady: style.fontReady,
      visible: style.labels,
      getPosition: (item) => item.position,
      getText: (item) => shortenLabel(item.name),
      getColor: (item) => labelColor(item.mcc, style),
      anchor: "end",
      offsetPx: [-LABEL_GAP_PX, 0],
      pickable: true,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: "selected-link",
      data: selectedLink === undefined ? [] : [selectedLink],
      getPath: (item) => item.path,
      getColor: (item) => mccColor(colors, item.slot),
      colors,
      widthPx: SELECTED_LINK_WIDTH_PX,
      pickable: false,
      colorTriggers: triggers,
    }),
    selectionLayer("selection", selectionPositions(geometry, emphasis), colors),
  ];
}

function baseBranchColor(slot: number | null, kind: BranchKind, style: PairStyle): Rgba {
  if (kind === "reassortment") {
    return style.colors.signal;
  }

  return kind === "plain" && style.colorByMcc ? mccColor(style.colors, slot) : style.colors.inkMuted;
}
