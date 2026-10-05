export type LeafAxis = "x" | "y";

export interface CanvasSize {
  width: number;
  height: number;
}

export interface CanvasFrame {
  size: CanvasSize;
  rows: number;
  leafAxis: LeafAxis;
}

export interface RowRange {
  first: number;
  last: number;
}

export interface TreeViewState {
  target: [number, number];
  zoomX: number;
  zoomY: number;
  zoomAxis: "X" | "Y";
  minZoomX: number;
  maxZoomX: number;
  minZoomY: number;
  maxZoomY: number;
}

export interface ViewStateRequest {
  target?: readonly number[] | undefined;
  zoomX?: number | undefined;
  zoomY?: number | undefined;
}

export const MAX_ROW_PX = 64;

export const TOOLBAR_ZOOM_FACTOR = 2;

const HALF_ROW = 0.5;

export function leafExtent({ size, leafAxis }: Pick<CanvasFrame, "size" | "leafAxis">): number {
  return leafAxis === "y" ? size.height : size.width;
}

export function crossExtent({ size, leafAxis }: Pick<CanvasFrame, "size" | "leafAxis">): number {
  return leafAxis === "y" ? size.width : size.height;
}

export function leafBounds(rows: number): [number, number] {
  return [-HALF_ROW, Math.max(rows, 1) - HALF_ROW];
}

export function fittedZoom(frame: CanvasFrame): number {
  const [low, high] = leafBounds(frame.rows);

  return Math.log2(Math.max(leafExtent(frame), 1) / (high - low));
}

export function zoomLimits(frame: CanvasFrame): [number, number] {
  const fitted = fittedZoom(frame);

  return [fitted, Math.max(fitted, Math.log2(MAX_ROW_PX))];
}

export function leafZoom(state: TreeViewState): number {
  return state.zoomAxis === "Y" ? state.zoomY : state.zoomX;
}

export function leafTarget(state: TreeViewState): number {
  return state.zoomAxis === "Y" ? state.target[1] : state.target[0];
}

export function rowPixels(state: TreeViewState): number {
  return 2 ** leafZoom(state);
}

export function subpixelZoom(frame: Pick<CanvasFrame, "size" | "leafAxis">): number {
  return Math.log2(1 + 1 / Math.max(leafExtent(frame), 1));
}

export function snappedZoom(frame: CanvasFrame, zoom: number): number {
  const [minZoom, maxZoom] = zoomLimits(frame);
  const snap = subpixelZoom(frame);

  if (zoom - minZoom < snap) {
    return minZoom;
  }

  return maxZoom - zoom < snap ? maxZoom : zoom;
}

export function viewStateAt(frame: CanvasFrame, zoom: number, center: number): TreeViewState {
  const [minZoom, maxZoom] = zoomLimits(frame);
  const leaf = snappedZoom(frame, zoom);
  const halfVisible = leafExtent(frame) / 2 / 2 ** leaf;
  const [low, high] = leafBounds(frame.rows);

  const leafCenter =
    high - low <= 2 * halfVisible
      ? (low + high) / 2
      : Math.min(Math.max(center, low + halfVisible), high - halfVisible);

  const crossCenter = crossExtent(frame) / 2;

  return frame.leafAxis === "y"
    ? {
        target: [crossCenter, leafCenter],
        zoomX: 0,
        zoomY: leaf,
        zoomAxis: "Y",
        minZoomX: 0,
        maxZoomX: 0,
        minZoomY: minZoom,
        maxZoomY: maxZoom,
      }
    : {
        target: [leafCenter, crossCenter],
        zoomX: leaf,
        zoomY: 0,
        zoomAxis: "X",
        minZoomX: minZoom,
        maxZoomX: maxZoom,
        minZoomY: 0,
        maxZoomY: 0,
      };
}

export function fitViewState(frame: CanvasFrame): TreeViewState {
  return viewStateAt(frame, fittedZoom(frame), 0);
}

export function constrainViewState(
  request: ViewStateRequest,
  frame: CanvasFrame,
  current: TreeViewState,
): TreeViewState {
  const zoom = frame.leafAxis === "y" ? request.zoomY : request.zoomX;
  const center = request.target?.[frame.leafAxis === "y" ? 1 : 0];

  return viewStateAt(frame, zoom ?? leafZoom(current), center ?? leafTarget(current));
}

export function zoomViewState(state: TreeViewState, frame: CanvasFrame, factor: number): TreeViewState {
  return viewStateAt(frame, leafZoom(state) + Math.log2(factor), leafTarget(state));
}

export function panViewState(state: TreeViewState, frame: CanvasFrame, center: number): TreeViewState {
  return viewStateAt(frame, leafZoom(state), center);
}

export function fitRowsViewState(frame: CanvasFrame, { first, last }: RowRange): TreeViewState {
  const low = Math.min(first, last) - HALF_ROW;
  const high = Math.max(first, last) + HALF_ROW;

  return viewStateAt(frame, Math.log2(Math.max(leafExtent(frame), 1) / (high - low)), (low + high) / 2);
}

export function visibleLeafRange(state: TreeViewState, frame: CanvasFrame): [number, number] {
  const halfVisible = leafExtent(frame) / 2 / rowPixels(state);
  const center = leafTarget(state);

  return [center - halfVisible, center + halfVisible];
}

export function resizeViewState(state: TreeViewState, previous: CanvasFrame, frame: CanvasFrame): TreeViewState {
  const [low, high] = visibleLeafRange(state, previous);

  return fitRowsViewState(frame, { first: low + HALF_ROW, last: high - HALF_ROW });
}

export function worldPosition(leafAxis: LeafAxis, cross: number, leaf: number): [number, number] {
  return leafAxis === "y" ? [cross, leaf] : [leaf, cross];
}

export const MINIMAP_MIN_ROWS = 200;

export const MINIMAP_CROSS_PX = 160;

export const MINIMAP_LEAF_PX = 200;

const MINIMAP_MIN_CANVAS_SHARE = 2;

export function minimapShown(frame: CanvasFrame): boolean {
  return frame.rows > MINIMAP_MIN_ROWS && leafExtent(frame) >= MINIMAP_MIN_CANVAS_SHARE * MINIMAP_LEAF_PX;
}

export function minimapSize({ leafAxis }: Pick<CanvasFrame, "leafAxis">): CanvasSize {
  return leafAxis === "y"
    ? { width: MINIMAP_CROSS_PX, height: MINIMAP_LEAF_PX }
    : { width: MINIMAP_LEAF_PX, height: MINIMAP_CROSS_PX };
}

export function minimapViewState(frame: CanvasFrame, size: CanvasSize): TreeViewState {
  const mini: CanvasFrame = { ...frame, size };
  const fitted = fitViewState(mini);
  const crossCenter = crossExtent(frame) / 2;
  const crossZoom = Math.log2(Math.max(crossExtent(mini), 1) / Math.max(crossExtent(frame), 1));

  return frame.leafAxis === "y"
    ? {
        ...fitted,
        target: [crossCenter, fitted.target[1]],
        zoomX: crossZoom,
        minZoomX: crossZoom,
        maxZoomX: crossZoom,
      }
    : {
        ...fitted,
        target: [fitted.target[0], crossCenter],
        zoomY: crossZoom,
        minZoomY: crossZoom,
        maxZoomY: crossZoom,
      };
}

export function visibleWorldRect(state: TreeViewState, frame: CanvasFrame): [number, number][] {
  const [first, last] = visibleLeafRange(state, frame);
  const cross = crossExtent(frame);

  return [
    worldPosition(frame.leafAxis, 0, first),
    worldPosition(frame.leafAxis, cross, first),
    worldPosition(frame.leafAxis, cross, last),
    worldPosition(frame.leafAxis, 0, last),
  ];
}

export function minimapLeafOffset(frame: CanvasFrame, size: CanvasSize, offsetPx: number): number {
  const [low, high] = leafBounds(frame.rows);

  return (offsetPx / Math.max(leafExtent({ size, leafAxis: frame.leafAxis }), 1)) * (high - low);
}

export function minimapLeafAt(frame: CanvasFrame, size: CanvasSize, offsetPx: number): number {
  return leafBounds(frame.rows)[0] + minimapLeafOffset(frame, size, offsetPx);
}
