import { useMemo, useReducer } from "react";
import { match } from "ts-pattern";

import {
  type CanvasFrame,
  type CanvasSize,
  constrainViewState,
  fitRowsViewState,
  fitViewState,
  type LeafAxis,
  leafZoom,
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
  resize(size: CanvasSize): void;
  update(request: ViewStateRequest): void;
  zoomIn(): void;
  zoomOut(): void;
  fit(): void;
  fitRows(range: RowRange): void;
  panTo(leaf: number): void;
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
  leafAxis: LeafAxis;
}

type ViewChange =
  | { type: "update"; request: ViewStateRequest }
  | { type: "zoom"; factor: number }
  | { type: "fit" }
  | { type: "fitRows"; range: RowRange }
  | { type: "pan"; leaf: number };

export type TreeViewAction = Drawing & ({ type: "resize"; size: CanvasSize } | ViewChange);

const ZOOM_EPSILON = 1e-9;

export function useTreeView(rows: number, leafAxis: LeafAxis): TreeView {
  const [stored, dispatch] = useReducer(treeViewReducer, undefined);

  const actions = useMemo<TreeViewActions>(() => {
    const drawing = { rows, leafAxis };

    return {
      resize(size) {
        dispatch({ ...drawing, type: "resize", size });
      },
      update(request) {
        dispatch({ ...drawing, type: "update", request });
      },
      zoomIn() {
        dispatch({ ...drawing, type: "zoom", factor: TOOLBAR_ZOOM_FACTOR });
      },
      zoomOut() {
        dispatch({ ...drawing, type: "zoom", factor: 1 / TOOLBAR_ZOOM_FACTOR });
      },
      fit() {
        dispatch({ ...drawing, type: "fit" });
      },
      fitRows(range) {
        dispatch({ ...drawing, type: "fitRows", range });
      },
      panTo(leaf) {
        dispatch({ ...drawing, type: "pan", leaf });
      },
    };
  }, [rows, leafAxis]);

  const current = currentView(stored, { rows, leafAxis });

  if (current === undefined) {
    return { frame: undefined, viewState: undefined, rowPx: 0, canZoomIn: false, canZoomOut: false, actions };
  }

  const [minZoom, maxZoom] = zoomLimits(current.frame);
  const zoom = leafZoom(current.viewState);

  return {
    frame: current.frame,
    viewState: current.viewState,
    rowPx: rowPixels(current.viewState),
    canZoomIn: zoom < maxZoom - ZOOM_EPSILON,
    canZoomOut: zoom > minZoom + ZOOM_EPSILON,
    actions,
  };
}

export function treeViewReducer(stored: StoredView | undefined, action: TreeViewAction): StoredView | undefined {
  const current = currentView(stored, action);

  if (action.type === "resize") {
    return resized(current, action, action.size);
  }

  if (current === undefined) {
    return undefined;
  }

  return { frame: current.frame, viewState: changed(current, action) };
}

export function currentView(stored: StoredView | undefined, { rows, leafAxis }: Drawing): StoredView | undefined {
  if (stored === undefined || (stored.frame.rows === rows && stored.frame.leafAxis === leafAxis)) {
    return stored;
  }

  const frame = { size: stored.frame.size, rows, leafAxis };

  return { frame, viewState: fitViewState(frame) };
}

function resized(current: StoredView | undefined, { rows, leafAxis }: Drawing, size: CanvasSize) {
  if (size.width <= 0 || size.height <= 0) {
    return undefined;
  }

  if (current !== undefined && current.frame.size.width === size.width && current.frame.size.height === size.height) {
    return current;
  }

  const frame: CanvasFrame = { size, rows, leafAxis };

  return {
    frame,
    viewState: current === undefined ? fitViewState(frame) : resizeViewState(current.viewState, current.frame, frame),
  };
}

function changed({ frame, viewState }: StoredView, change: ViewChange): TreeViewState {
  return match(change)
    .with({ type: "update" }, ({ request }) => constrainViewState(request, frame))
    .with({ type: "zoom" }, ({ factor }) => zoomViewState(viewState, frame, factor))
    .with({ type: "fit" }, () => fitViewState(frame))
    .with({ type: "fitRows" }, ({ range }) => fitRowsViewState(frame, range))
    .with({ type: "pan" }, ({ leaf }) => panViewState(viewState, frame, leaf))
    .exhaustive();
}
