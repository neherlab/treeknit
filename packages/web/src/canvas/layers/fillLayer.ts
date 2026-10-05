import type { Accessor, Color } from "@deck.gl/core";
import { SolidPolygonLayer } from "@deck.gl/layers";

import type { Rgba } from "../color";
import type { WorldPosition } from "../projection";

export interface FillLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPolygon: (object: D) => WorldPosition[];
  getFillColor: Accessor<D, Color>;
  visible?: boolean;
  pickable?: boolean;
  opacity?: number;
  highlightColor?: Rgba;
  colorTriggers?: readonly unknown[];
}

export function fillLayer<D>({
  id,
  data,
  getPolygon,
  getFillColor,
  visible = true,
  pickable = true,
  opacity = 1,
  highlightColor,
  colorTriggers = [],
}: FillLayerOptions<D>): SolidPolygonLayer<D> {
  return new SolidPolygonLayer<D>({
    id,
    data,
    getPolygon,
    getFillColor,
    visible,
    pickable,
    opacity,
    autoHighlight: highlightColor !== undefined,
    ...(highlightColor === undefined ? undefined : { highlightColor }),
    updateTriggers: { getFillColor: colorTriggers },
  });
}
