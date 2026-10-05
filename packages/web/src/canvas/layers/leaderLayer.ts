import { withOpacity } from "../color";
import type { DrawingColors } from "../drawingColors";
import type { WorldPosition } from "../projection";
import { DOT_PX, pathLayer } from "./pathLayer";

export const LEADER_OPACITY = 0.5;

const LEADER_WIDTH_PX = 1;

export interface LeaderLayerOptions<D extends { path: WorldPosition[] }> {
  id: string;
  data: readonly D[];
  colors: DrawingColors;
  visible: boolean;
}

export function leaderLayer<D extends { path: WorldPosition[] }>({ id, data, colors, visible }: LeaderLayerOptions<D>) {
  return pathLayer({
    id,
    data,
    getPath: (item) => item.path,
    getColor: withOpacity(colors.inkMuted, LEADER_OPACITY),
    widthPx: LEADER_WIDTH_PX,
    dashed: true,
    dashArray: DOT_PX,
    pickable: false,
    visible,
    colorTriggers: [colors],
  });
}
