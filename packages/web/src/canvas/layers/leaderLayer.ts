import type { Accessor, Color } from "@deck.gl/core";
import type { DrawingRules } from "@neherlab/treeknit-wasm";

import { withOpacity } from "../color";
import type { DrawingColors } from "../drawingColors";
import type { WorldPosition } from "../projection";
import { pathLayer } from "./pathLayer";

export interface LeaderLayerOptions<D extends { path: WorldPosition[] }> {
  id: string;
  data: readonly D[];
  colors: DrawingColors;
  rules: DrawingRules;
  visible: boolean;
  getColor?: Accessor<D, Color>;
  colorTriggers?: readonly unknown[];
}

export function leaderLayer<D extends { path: WorldPosition[] }>({
  id,
  data,
  colors,
  rules,
  visible,
  getColor = withOpacity(colors.inkMuted, rules.leaderOpacity),
  colorTriggers = [colors],
}: LeaderLayerOptions<D>) {
  return pathLayer({
    id,
    data,
    getPath: (item) => item.path,
    getColor,
    widthPx: rules.leaderWidthPx,
    dashPx: rules.dotPx,
    pickable: false,
    visible,
    colorTriggers,
  });
}
