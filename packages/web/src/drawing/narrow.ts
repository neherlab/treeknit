import type { LeafAxis } from "../canvas/viewState";

export const NARROW_CANVAS_PX = 640;

export function drawingLeafAxis(canvasWidthPx: number | undefined): LeafAxis {
  return canvasWidthPx !== undefined && canvasWidthPx < NARROW_CANVAS_PX ? "x" : "y";
}
