import type { Accessor, Color } from "@deck.gl/core";

import { type Rgba, withOpacity } from "../color";
import type { DrawingColors } from "../drawingColors";
import type { WorldPosition } from "../projection";
import { markLayer } from "./markLayer";
import { pathLayer } from "./pathLayer";

export const BRANCH_WIDTH_PX = 1.5;

export const REASSORTMENT_WIDTH_PX = 2;

export const MARK_RADIUS_PX = 3.5;

export const MARK_LINE_PX = 1.5;

export const SELECTION_RADIUS_PX = 7;

export const SELECTION_LINE_PX = 2;

export const HOVER_OPACITY = 0.35;

export interface BranchLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPath: (object: D) => WorldPosition[];
  getColor: Accessor<D, Color>;
  colors: DrawingColors;
  widthPx?: number;
  dashed?: boolean;
  opacity?: number;
  visible?: boolean;
  colorTriggers: readonly unknown[];
}

export function branchLayer<D>({
  id,
  data,
  getPath,
  getColor,
  colors,
  widthPx = BRANCH_WIDTH_PX,
  dashed = false,
  opacity = 1,
  visible = true,
  colorTriggers,
}: BranchLayerOptions<D>) {
  return pathLayer({
    id,
    data,
    getPath,
    getColor,
    widthPx,
    dashed,
    opacity,
    visible,
    highlightColor: hoverColor(colors),
    colorTriggers,
  });
}

export interface RingLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPosition: (object: D) => WorldPosition;
  getLineColor: Accessor<D, Color>;
  colors: DrawingColors;
  hollow?: boolean;
  colorTriggers: readonly unknown[];
}

export function ringLayer<D>({
  id,
  data,
  getPosition,
  getLineColor,
  colors,
  hollow = true,
  colorTriggers,
}: RingLayerOptions<D>) {
  return markLayer({
    id,
    data,
    getPosition,
    getLineColor,
    ...(hollow ? { getFillColor: colors.ground } : undefined),
    radiusPx: MARK_RADIUS_PX,
    lineWidthPx: MARK_LINE_PX,
    colorTriggers,
  });
}

export function selectionLayer(id: string, positions: readonly WorldPosition[], colors: DrawingColors) {
  return markLayer({
    id,
    data: positions,
    getPosition: (position) => position,
    getLineColor: colors.focus,
    radiusPx: SELECTION_RADIUS_PX,
    lineWidthPx: SELECTION_LINE_PX,
    pickable: false,
    colorTriggers: [colors],
  });
}

export function hoverColor(colors: DrawingColors): Rgba {
  return withOpacity(colors.focus, HOVER_OPACITY);
}
