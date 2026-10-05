import { describe, expect, test } from "vitest";

import { currentView, type Drawing, type StoredView, treeViewReducer } from "../useTreeView";
import { fitViewState, rowPixels, visibleLeafRange } from "../viewState";

const DRAWING: Drawing = { rows: 300, leafAxis: "y" };

const SIZE = { width: 800, height: 600 };

function sized(): StoredView {
  const stored = treeViewReducer(undefined, { ...DRAWING, type: "resize", size: SIZE });

  if (stored === undefined) {
    throw new Error("A sized canvas has a view");
  }

  return stored;
}

describe("treeViewReducer", () => {
  test("fits the drawing on the first size", () => {
    expect(sized()).toStrictEqual({
      frame: { size: SIZE, ...DRAWING },
      viewState: fitViewState({ size: SIZE, ...DRAWING }),
    });
  });

  test("has no view before the canvas has a size", () => {
    expect(treeViewReducer(undefined, { ...DRAWING, type: "zoom", factor: 2 })).toBeUndefined();
    expect(treeViewReducer(sized(), { ...DRAWING, type: "resize", size: { width: 800, height: 0 } })).toBeUndefined();
  });

  test("keeps the same view object for a resize to the same size", () => {
    const stored = sized();

    expect(treeViewReducer(stored, { ...DRAWING, type: "resize", size: { ...SIZE } })).toBe(stored);
  });

  test("keeps the visible rows when the canvas resizes", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "fitRows", range: { first: 100, last: 149 } });
    const resized = treeViewReducer(zoomed, { ...DRAWING, type: "resize", size: { width: 800, height: 900 } });

    expect(resized === undefined ? [] : visibleLeafRange(resized.viewState, resized.frame)).toStrictEqual([
      99.5, 149.5,
    ]);
  });

  test("zooms, pans, and fits through the view-state rules", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "zoom", factor: 4 });
    const panned = treeViewReducer(zoomed, { ...DRAWING, type: "pan", leaf: 1_000 });
    const fitted = treeViewReducer(panned, { ...DRAWING, type: "fit" });

    expect(zoomed === undefined ? 0 : rowPixels(zoomed.viewState)).toBeCloseTo(8, 12);
    expect(panned === undefined ? [] : visibleLeafRange(panned.viewState, panned.frame)[1]).toBeCloseTo(299.5, 9);
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

  test("refits when the drawing gets other rows or another leaf axis", () => {
    const zoomed = treeViewReducer(sized(), { ...DRAWING, type: "zoom", factor: 4 });
    const other: Drawing = { rows: 50, leafAxis: "x" };

    expect(currentView(zoomed, other)?.viewState).toStrictEqual(fitViewState({ size: SIZE, ...other }));
    expect(currentView(zoomed, DRAWING)).toBe(zoomed);
  });
});
