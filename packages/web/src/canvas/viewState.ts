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

export function viewStateAt(frame: CanvasFrame, zoom: number, center: number): TreeViewState {
  const [minZoom, maxZoom] = zoomLimits(frame);
  const leaf = Math.min(Math.max(zoom, minZoom), maxZoom);
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
  next: Pick<TreeViewState, "target"> & Partial<Pick<TreeViewState, "zoomX" | "zoomY">>,
  frame: CanvasFrame,
): TreeViewState {
  const zoom = frame.leafAxis === "y" ? next.zoomY : next.zoomX;
  const center = frame.leafAxis === "y" ? next.target[1] : next.target[0];

  return viewStateAt(frame, zoom ?? fittedZoom(frame), center);
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
