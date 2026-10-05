import { describe, expect, test } from "vitest";

import {
  createTreeViewStore,
  currentView,
  type Drawing,
  type StoredView,
  treeViewActions,
  treeViewReducer,
} from "../useTreeView";
import { type CanvasSize, fitViewState, type MeasuredSize, rowPixels, visibleLeafRange } from "../viewState";

const DRAWING: Drawing = { rows: 300 };

const SIZE = { width: 800, height: 600 };

const FRAME = { size: SIZE, rows: 300, leafAxis: "y" } as const;

function sized(): StoredView {
  const stored = treeViewReducer(undefined, { ...DRAWING, type: "resize", ...measured(SIZE) });

  if (stored === undefined) {
    throw new Error("A sized canvas has a view");
  }

  return stored;
}

const REAL = { size: { width: 930, height: 835 }, rows: 300, leafAxis: "y" } as const;

function resize(stored: StoredView | undefined, canvas: CanvasSize, areaWidth = canvas.width): StoredView | undefined {
  return treeViewReducer(stored, { ...DRAWING, type: "resize", ...measured(canvas, areaWidth) });
}

function measured(canvas: CanvasSize, areaWidth = canvas.width): MeasuredSize {
  return { canvas, areaWidth };
}

describe("treeViewReducer", () => {
  test("fits the drawing on the first size", () => {
    expect(sized()).toStrictEqual({
      frame: FRAME,
      viewState: fitViewState(FRAME),
    });
  });

  test("has no view before the canvas has a size", () => {
    expect(treeViewReducer(undefined, { ...DRAWING, type: "zoom", factor: 2 })).toBeUndefined();
    expect(
      treeViewReducer(sized(), { ...DRAWING, type: "resize", ...measured({ width: 800, height: 0 }) }),
    ).toBeUndefined();
  });

  test("keeps the same view object for a resize to the same size", () => {
    const stored = sized();

    expect(treeViewReducer(stored, { ...DRAWING, type: "resize", ...measured({ ...SIZE }) })).toBe(stored);
  });

  test("keeps the visible rows when the canvas resizes", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "fitRows", range: { first: 100, last: 149 } });

    const resized = treeViewReducer(zoomed, { ...DRAWING, type: "resize", ...measured({ width: 800, height: 900 }) });

    expect(resized === undefined ? [] : visibleLeafRange(resized.viewState, resized.frame)).toStrictEqual([
      99.5, 149.5,
    ]);
  });

  test("zooms, pans, and fits through the view-state rules", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "zoom", factor: 4 });

    const panned = treeViewReducer(zoomed, { ...DRAWING, type: "pan", leaf: 1_000 });
    const fitted = treeViewReducer(panned, { ...DRAWING, type: "fit" });

    const zoomedRowPx = zoomed === undefined ? Number.NaN : rowPixels(zoomed.viewState);
    const pannedEnd = panned === undefined ? Number.NaN : visibleLeafRange(panned.viewState, panned.frame)[1];

    expect(zoomedRowPx).toBe(8);
    expect(pannedEnd).toBe(299.5);
    expect(fitted?.viewState).toStrictEqual(sized().viewState);
  });

  test("constrains a controller update to the leaf axis", () => {
    const updated = treeViewReducer(sized(), {
      ...DRAWING,
      type: "update",
      request: { target: [5, 150], zoomX: 3, zoomY: 3 },
    });

    expect(updated?.viewState.zoomX).toBe(0);
    expect(updated?.viewState.target).toStrictEqual([400, 150]);
    expect(updated?.viewState.zoomY).toBe(3);
  });

  test("keeps the current center and zoom for a controller update without target and zoom", () => {
    const panned = treeViewReducer(sized(), { ...DRAWING, type: "fitRows", range: { first: 200, last: 219 } });
    const updated = treeViewReducer(panned, { ...DRAWING, type: "update", request: {} });

    expect(updated?.viewState).toStrictEqual(panned?.viewState);
  });

  test("adds two drag moves queued before a render, each to the latest center", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "fitRows", range: { first: 100, last: 119 } });

    const once = treeViewReducer(zoomed, { ...DRAWING, type: "panBy", delta: 10 });
    const twice = treeViewReducer(once, { ...DRAWING, type: "panBy", delta: 15 });

    expect(twice === undefined ? [] : visibleLeafRange(twice.viewState, twice.frame)).toStrictEqual([124.5, 144.5]);
  });

  test("stops a drag move at the drawing edge", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "fitRows", range: { first: 100, last: 119 } });

    const moved = treeViewReducer(zoomed, { ...DRAWING, type: "panBy", delta: 1_000 });

    expect(moved === undefined ? [] : visibleLeafRange(moved.viewState, moved.frame)).toStrictEqual([279.5, 299.5]);
  });

  test("refits when the drawing gets other rows", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "zoom", factor: 4 });

    expect(currentView(zoomed, { rows: 50 })?.viewState).toStrictEqual(fitViewState({ ...FRAME, rows: 50 }));
    expect(currentView(zoomed, DRAWING)).toBe(zoomed);
  });

  test("ends in the wide layout, fitted, after a 0 x 0 canvas gets its real size", () => {
    const unsized = resize(undefined, { width: 0, height: 0 });
    const real = resize(unsized, { width: 930, height: 835 }, 1043);

    expect({ unsized, real }).toStrictEqual({
      unsized: undefined,
      real: { frame: REAL, viewState: fitViewState(REAL) },
    });
  });

  test("leaves the narrow layout of a placeholder size and fits the drawing again at the real size", () => {
    const placeholder = resize(undefined, { width: 300, height: 150 });
    const zoomed = treeViewReducer(placeholder, { ...DRAWING, type: "zoom", factor: 4 });
    const real = resize(zoomed, { width: 930, height: 835 }, 1043);

    expect({ placeholder: placeholder?.frame.leafAxis, real }).toStrictEqual({
      placeholder: "x",
      real: { frame: REAL, viewState: fitViewState(REAL) },
    });
  });

  test("keeps the wide layout of a 700 px area whether the minimap strip narrows the canvas or not", () => {
    const beside = resize(undefined, { width: 587, height: 835 }, 700);
    const below = resize(beside, { width: 700, height: 722 }, 700);

    expect([beside?.frame.leafAxis, below?.frame.leafAxis]).toStrictEqual(["y", "y"]);
  });
});

describe("treeViewActions", () => {
  test("write each reduced view into the store", () => {
    const store = createTreeViewStore();
    const actions = treeViewActions(store, DRAWING);

    actions.resize(measured(SIZE));
    actions.fitRows({ first: 100, last: 119 });
    actions.panBy(10);

    const stored = store.getState().stored;

    expect(stored === undefined ? [] : visibleLeafRange(stored.viewState, stored.frame)).toStrictEqual([109.5, 129.5]);
  });
});
