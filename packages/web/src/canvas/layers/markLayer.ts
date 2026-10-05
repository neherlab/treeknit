import type { Accessor, Color } from "@deck.gl/core";
import { ScatterplotLayer } from "@deck.gl/layers";

import type { WorldPosition } from "../projection";

export interface MarkLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPosition: (object: D) => WorldPosition;
  getLineColor: Accessor<D, Color>;
  getFillColor?: Accessor<D, Color>;
  radiusPx: number;
  lineWidthPx: number;
  pickable?: boolean;
  colorTriggers?: readonly unknown[];
}

export function markLayer<D>({
  id,
  data,
  getPosition,
  getLineColor,
  getFillColor,
  radiusPx,
  lineWidthPx,
  pickable = true,
  colorTriggers = [],
}: MarkLayerOptions<D>): ScatterplotLayer<D> {
  return new ScatterplotLayer<D>({
    id,
    data,
    getPosition,
    getLineColor,
    getFillColor: getFillColor ?? [0, 0, 0, 0],
    filled: getFillColor !== undefined,
    stroked: true,
    getRadius: radiusPx,
    radiusUnits: "pixels",
    getLineWidth: lineWidthPx,
    lineWidthUnits: "pixels",
    pickable,
    updateTriggers: { getLineColor: colorTriggers, getFillColor: colorTriggers },
  });
}
