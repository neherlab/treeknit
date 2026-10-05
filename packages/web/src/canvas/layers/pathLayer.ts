import type { Accessor, Color } from "@deck.gl/core";
import { PathStyleExtension, type PathStyleExtensionProps } from "@deck.gl/extensions";
import { PathLayer, type PathLayerProps } from "@deck.gl/layers";

import type { Rgba } from "../color";
import type { WorldPosition } from "../projection";

export const DASH_PX: [number, number] = [4, 3];

export const DOT_PX: [number, number] = [1, 3];

const DASH_EXTENSIONS = [new PathStyleExtension({ dash: true })];

export type StyledPathLayer<D> = PathLayer<D, PathStyleExtensionProps<D>>;

export interface PathLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPath: (object: D) => WorldPosition[];
  getColor: Accessor<D, Color>;
  widthPx: number;
  dashed?: boolean;
  dashArray?: [number, number];
  pickable?: boolean;
  opacity?: number;
  visible?: boolean;
  highlightColor?: Rgba;
  colorTriggers?: readonly unknown[];
}

export function pathLayer<D>({
  id,
  data,
  getPath,
  getColor,
  widthPx,
  dashed = false,
  dashArray = DASH_PX,
  pickable = true,
  opacity = 1,
  visible = true,
  highlightColor,
  colorTriggers = [],
}: PathLayerOptions<D>): StyledPathLayer<D> {
  const solid: PathLayerProps<D> & PathStyleExtensionProps<D> = {
    id,
    data,
    getPath,
    getColor,
    getWidth: widthPx,
    widthUnits: "pixels",
    jointRounded: false,
    capRounded: false,
    pickable,
    opacity,
    visible,
    autoHighlight: highlightColor !== undefined,
    ...(highlightColor === undefined ? undefined : { highlightColor }),
    updateTriggers: { getColor: colorTriggers },
  };

  return new PathLayer<D, PathStyleExtensionProps<D>>(
    dashed ? { ...solid, extensions: DASH_EXTENSIONS, getDashArray: dashArray, dashUnits: "pixels" } : solid,
  );
}
