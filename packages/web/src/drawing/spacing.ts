export const DRAWING_MARGIN_PX = 16;

export const LABEL_GAP_PX = 6;

export function innerWidthPx(crossPx: number): number {
  return Math.max(crossPx - 2 * DRAWING_MARGIN_PX, 0);
}
