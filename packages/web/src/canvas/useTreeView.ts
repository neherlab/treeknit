import { match } from "ts-pattern";
import { useStore } from "zustand";
import { createStore, type StoreApi } from "zustand/vanilla";

import {
  type CanvasFrame,
  constrainViewState,
  fitRowsViewState,
  fitViewState,
  leafAxisFor,
  leafTarget,
  leafZoom,
  type MeasuredSize,
  panViewState,
  resizeViewState,
  type RowRange,
  rowPixels,
  TOOLBAR_ZOOM_FACTOR,
  type TreeViewState,
  type ViewStateRequest,
  zoomLimits,
  zoomViewState,
} from "./viewState";

export interface TreeViewActions {
  resize(measured: MeasuredSize): void;
  update(request: ViewStateRequest): void;
  zoomIn(): void;
  zoomOut(): void;
  fit(): void;
  fitRows(range: RowRange): void;
  panTo(leaf: number): void;
  panBy(delta: number): void;
}

export interface TreeView {
  frame: CanvasFrame | undefined;
  viewState: TreeViewState | undefined;
  rowPx: number;
  canZoomIn: boolean;
  canZoomOut: boolean;
  actions: TreeViewActions;
}

export interface StoredView {
  frame: CanvasFrame;
  viewState: TreeViewState;
}

export interface Drawing {
  rows: number;
}

type ViewChange =
  | { type: "update"; request: ViewStateRequest }
  | { type: "zoom"; factor: number }
  | { type: "fit" }
  | { type: "fitRows"; range: RowRange }
  | { type: "pan"; leaf: number }
  | { type: "panBy"; delta: number };

type Resize = { type: "resize" } & MeasuredSize;

export type TreeViewAction = Drawing & (Resize | ViewChange);

export interface TreeViewHandle {
  store: TreeViewStore;
  drawing: Drawing;
  actions: TreeViewActions;
}

export type TreeViewStore = StoreApi<{ stored: StoredView | undefined }>;

export function createTreeViewStore(): TreeViewStore {
  return createStore(() => ({ stored: undefined }));
}

export function treeViewActions(store: TreeViewStore, drawing: Drawing): TreeViewActions {
  const dispatch = (change: Resize | ViewChange) => {
    store.setState(({ stored }) => ({ stored: treeViewReducer(stored, { ...drawing, ...change }) }));
  };

  return {
    resize(measured) {
      dispatch({ type: "resize", ...measured });
    },
    update(request) {
      dispatch({ type: "update", request });
    },
    zoomIn() {
      dispatch({ type: "zoom", factor: TOOLBAR_ZOOM_FACTOR });
    },
    zoomOut() {
      dispatch({ type: "zoom", factor: 1 / TOOLBAR_ZOOM_FACTOR });
    },
    fit() {
      dispatch({ type: "fit" });
    },
    fitRows(range) {
      dispatch({ type: "fitRows", range });
    },
    panTo(leaf) {
      dispatch({ type: "pan", leaf });
    },
    panBy(delta) {
      dispatch({ type: "panBy", delta });
    },
  };
}

export function useTreeView({ store, drawing, actions }: TreeViewHandle): TreeView {
  const stored = useStore(store, (state) => state.stored);
  const current = currentView(stored, drawing);

  if (current === undefined) {
    return { frame: undefined, viewState: undefined, rowPx: 0, canZoomIn: false, canZoomOut: false, actions };
  }

  return {
    frame: current.frame,
    viewState: current.viewState,
    rowPx: rowPixels(current.viewState),
    ...zoomRoom(current),
    actions,
  };
}

export function useZoomRoom({ store, drawing }: TreeViewHandle): Pick<TreeView, "canZoomIn" | "canZoomOut"> {
  const canZoomIn = useStore(store, (state) => zoomRoomOf(state.stored, drawing).canZoomIn);
  const canZoomOut = useStore(store, (state) => zoomRoomOf(state.stored, drawing).canZoomOut);

  return { canZoomIn, canZoomOut };
}

export function useTreeViewReady({ store, drawing }: TreeViewHandle): boolean {
  return useStore(store, (state) => currentView(state.stored, drawing) !== undefined);
}

export function treeViewReducer(stored: StoredView | undefined, action: TreeViewAction): StoredView | undefined {
  const current = currentView(stored, action);

  if (action.type === "resize") {
    return resized(current, action);
  }

  if (current === undefined) {
    return undefined;
  }

  return { frame: current.frame, viewState: changed(current, action) };
}

export function currentView(stored: StoredView | undefined, { rows }: Drawing): StoredView | undefined {
  if (stored === undefined || stored.frame.rows === rows) {
    return stored;
  }

  const frame = { ...stored.frame, rows };

  return { frame, viewState: fitViewState(frame) };
}

function zoomRoomOf(stored: StoredView | undefined, drawing: Drawing): Pick<TreeView, "canZoomIn" | "canZoomOut"> {
  const current = currentView(stored, drawing);

  return current === undefined ? { canZoomIn: false, canZoomOut: false } : zoomRoom(current);
}

function zoomRoom({ frame, viewState }: StoredView): Pick<TreeView, "canZoomIn" | "canZoomOut"> {
  const [minZoom, maxZoom] = zoomLimits(frame);
  const zoom = leafZoom(viewState);

  return { canZoomIn: zoom < maxZoom, canZoomOut: zoom > minZoom };
}

function resized(current: StoredView | undefined, { rows, canvas: size, areaWidth }: Drawing & MeasuredSize) {
  if (size.width <= 0 || size.height <= 0) {
    return current;
  }

  const frame: CanvasFrame = { size, rows, leafAxis: leafAxisFor(areaWidth) };

  if (current === undefined || current.frame.leafAxis !== frame.leafAxis) {
    return { frame, viewState: fitViewState(frame) };
  }

  if (current.frame.size.width === size.width && current.frame.size.height === size.height) {
    return current;
  }

  return { frame, viewState: resizeViewState(current.viewState, current.frame, frame) };
}

function changed({ frame, viewState }: StoredView, change: ViewChange): TreeViewState {
  return match(change)
    .with({ type: "update" }, ({ request }) => constrainViewState(request, frame, viewState))
    .with({ type: "zoom" }, ({ factor }) => zoomViewState(viewState, frame, factor))
    .with({ type: "fit" }, () => fitViewState(frame))
    .with({ type: "fitRows" }, ({ range }) => fitRowsViewState(frame, range))
    .with({ type: "pan" }, ({ leaf }) => panViewState(viewState, frame, leaf))
    .with({ type: "panBy" }, ({ delta }) => panViewState(viewState, frame, leafTarget(viewState) + delta))
    .exhaustive();
}
