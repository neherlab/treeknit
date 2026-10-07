import type { LayersList } from "@deck.gl/core";
import type { DrawingRules } from "@neherlab/treeknit-wasm";

import { type Rgba, withOpacity } from "../canvas/color";
import { type DrawingColors, mccColor } from "../canvas/drawingColors";
import { fillLayer } from "../canvas/layers/fillLayer";
import { labelLayer } from "../canvas/layers/labelLayer";
import { leaderLayer } from "../canvas/layers/leaderLayer";
import { branchLayer, hoverColor, ringLayer, selectionLayer } from "../canvas/layers/treeLayers";
import { emphasisOpacity, type PairEmphasis } from "../drawing/selection";
import { type BranchItem, type LinkItem, type MarkItem, PAIR_LAYER, type TanglegramGeometry } from "./geometry";

const SELECTED_LINK_WIDTH_PX = 2.5;

export type BranchKind = "plain" | "added" | "reassortment";

export type MarkKind = "imputed" | "reassortment";

export interface PairStyle {
  colors: DrawingColors;
  rules: DrawingRules;
  colorByMcc: boolean;
  emphasis: PairEmphasis;
  ribbons: boolean;
  labels: boolean;
  fontReady: boolean;
  fade: number;
}

export type ColorStyle = Pick<PairStyle, "colors" | "colorByMcc" | "rules"> & { emphasis: Pick<PairEmphasis, "mcc"> };

export function ribbonsShown(rowPx: number, rules: DrawingRules): boolean {
  return rowPx < rules.linkMinRowPx;
}

export function branchColor(item: Pick<BranchItem, "mcc" | "slot">, kind: BranchKind, style: ColorStyle): Rgba {
  const base = baseBranchColor(item.slot, kind, style);

  return withOpacity(base, emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function linkColor(item: Pick<LinkItem, "mcc" | "slot">, style: ColorStyle): Rgba {
  return withOpacity(mccColor(style.colors, item.slot), emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function ribbonColor(item: Pick<LinkItem, "mcc" | "slot">, style: ColorStyle): Rgba {
  return withOpacity(
    mccColor(style.colors, item.slot),
    style.rules.ribbonOpacity * emphasisOpacity(item.mcc, style.emphasis.mcc),
  );
}

export function markColor(item: Pick<MarkItem, "mcc" | "slot">, kind: MarkKind, style: ColorStyle): Rgba {
  const base = kind === "reassortment" ? style.colors.signal : baseBranchColor(item.slot, "plain", style);

  return withOpacity(base, emphasisOpacity(item.mcc, style.emphasis.mcc));
}

export function labelColor(mcc: number | null, style: ColorStyle): Rgba {
  return withOpacity(style.colors.ink, emphasisOpacity(mcc, style.emphasis.mcc));
}

export function leaderColor(mcc: number | null, style: ColorStyle): Rgba {
  return withOpacity(style.colors.inkMuted, style.rules.leaderOpacity * emphasisOpacity(mcc, style.emphasis.mcc));
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
  const { colors, rules, emphasis } = style;
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
      widthPx: rules.linkWidthPx,
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
      widthPx: rules.branchWidthPx,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.addedBranches,
      data: geometry.branches.added,
      getPath: (item) => item.path,
      getColor: branch("added"),
      colors,
      widthPx: rules.branchWidthPx,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: PAIR_LAYER.reassortmentBranches,
      data: geometry.branches.reassortment,
      getPath: (item) => item.path,
      getColor: branch("reassortment"),
      colors,
      widthPx: rules.reassortmentWidthPx,
      colorTriggers: triggers,
    }),
    leaderLayer({
      id: PAIR_LAYER.leaders,
      data: geometry.leaders,
      colors,
      rules,
      visible: style.labels && style.fontReady,
      getColor: (item) => leaderColor(item.mcc, style),
      colorTriggers: triggers,
    }),
    ringLayer({
      id: PAIR_LAYER.imputedMarks,
      data: geometry.marks.imputed,
      getPosition: (item) => item.position,
      getLineColor: (item) => markColor(item, "imputed", style),
      colors,
      rules,
      colorTriggers: triggers,
    }),
    ringLayer({
      id: PAIR_LAYER.reassortmentMarks,
      data: geometry.marks.reassortment,
      getPosition: (item) => item.position,
      getLineColor: (item) => markColor(item, "reassortment", style),
      colors,
      rules,
      colorTriggers: triggers,
    }),
    labelLayer({
      id: PAIR_LAYER.leftLabels,
      data: geometry.labels.left,
      fontReady: style.fontReady,
      visible: style.labels,
      getPosition: (item) => item.position,
      getText: (item) => item.text,
      getColor: (item) => labelColor(item.mcc, style),
      anchor: "start",
      offsetPx: [rules.labelGapPx, 0],
      pickable: true,
      colorTriggers: triggers,
    }),
    labelLayer({
      id: PAIR_LAYER.rightLabels,
      data: geometry.labels.right,
      fontReady: style.fontReady,
      visible: style.labels,
      getPosition: (item) => item.position,
      getText: (item) => item.text,
      getColor: (item) => labelColor(item.mcc, style),
      anchor: "end",
      offsetPx: [-rules.labelGapPx, 0],
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

function baseBranchColor(slot: number | null, kind: BranchKind, style: ColorStyle): Rgba {
  if (kind === "reassortment") {
    return style.colors.signal;
  }

  return kind === "plain" && style.colorByMcc ? mccColor(style.colors, slot) : style.colors.inkMuted;
}
