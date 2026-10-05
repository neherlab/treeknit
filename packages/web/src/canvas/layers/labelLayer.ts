import type { Accessor, Color, Position } from "@deck.gl/core";
import { TextLayer } from "@deck.gl/layers";

import { LABEL_FONT_FAMILY, LABEL_FONT_SIZE_PX, LABEL_FONT_WEIGHT } from "../labels";

export type LabelAnchor = "start" | "middle" | "end";

export interface LabelLayerOptions<D> {
  id: string;
  data: readonly D[];
  fontReady: boolean;
  visible: boolean;
  getPosition: Accessor<D, Position>;
  getText: (object: D) => string;
  getColor: Accessor<D, Color>;
  anchor: LabelAnchor;
  offsetPx: [number, number];
  pickable?: boolean;
  colorTriggers?: readonly unknown[];
}

export function labelLayer<D>({
  id,
  data,
  fontReady,
  visible,
  getPosition,
  getText,
  getColor,
  anchor,
  offsetPx,
  pickable = false,
  colorTriggers = [],
}: LabelLayerOptions<D>): TextLayer<D> | null {
  if (!fontReady) {
    return null;
  }

  return new TextLayer<D>({
    id,
    data,
    visible,
    pickable,
    getPosition,
    getText,
    getColor,
    characterSet: "auto",
    fontFamily: LABEL_FONT_FAMILY,
    fontWeight: LABEL_FONT_WEIGHT,
    getSize: LABEL_FONT_SIZE_PX,
    sizeUnits: "pixels",
    getTextAnchor: anchor,
    getAlignmentBaseline: "center",
    getPixelOffset: offsetPx,
    updateTriggers: { getColor: colorTriggers },
  });
}
