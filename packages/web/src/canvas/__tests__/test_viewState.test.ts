import { OrthographicViewport } from "@deck.gl/core";
import { describe, expect, test } from "vitest";

import {
  type CanvasFrame,
  constrainViewState,
  fitRowsViewState,
  fitViewState,
  MAX_ROW_PX,
  panViewState,
  rowPixels,
  type TreeViewState,
  visibleLeafRange,
  worldPosition,
  zoomViewState,
} from "../viewState";

const WIDE: CanvasFrame = { size: { width: 800, height: 600 }, rows: 300, leafAxis: "y" };

const NARROW: CanvasFrame = { size: { width: 500, height: 700 }, rows: 300, leafAxis: "x" };

function crossSpan(state: TreeViewState, frame: CanvasFrame): [number, number] {
  const axis = frame.leafAxis === "y" ? 0 : 1;
  const extent = frame.leafAxis === "y" ? frame.size.width : frame.size.height;
  const start = project(state, frame, 0, 10)[axis] ?? Number.NaN;
  const end = project(state, frame, extent, 10)[axis] ?? Number.NaN;

  return [start, end];
}

function project(state: TreeViewState, frame: CanvasFrame, cross: number, leaf: number): number[] {
  const viewport = new OrthographicViewport({ ...frame.size, ...state });

  return viewport.project(worldPosition(frame.leafAxis, cross, leaf));
}

describe("fitViewState", () => {
  test("shows every row, each half-row margin included, across the leaf axis", () => {
    const state = fitViewState(WIDE);

    expect(project(state, WIDE, 0, -0.5)[1]).toBeCloseTo(0, 9);
    expect(project(state, WIDE, 0, 299.5)[1]).toBeCloseTo(600, 9);
    expect(rowPixels(state)).toBeCloseTo(2, 12);
  });

  test("maps world x one to one onto pixels on the wide layout", () => {
    expect(crossSpan(fitViewState(WIDE), WIDE)).toStrictEqual([0, 800]);
  });

  test("puts the leaves on x and fits world y to the height on the narrow layout", () => {
    const state = fitViewState(NARROW);

    expect(state.zoomAxis).toBe("X");
    expect(state.zoomY).toBe(0);
    expect(project(state, NARROW, 0, -0.5)[0]).toBeCloseTo(0, 9);
    expect(project(state, NARROW, 0, 299.5)[0]).toBeCloseTo(500, 9);
    expect(crossSpan(state, NARROW)).toStrictEqual([0, 700]);
  });
});

describe("view-state update", () => {
  const updates = [WIDE, NARROW].flatMap((frame) => {
    const fitted = fitViewState(frame);
    const zoomed = zoomViewState(fitted, frame, 8);

    return [
      { frame, change: "toolbar zoom", fitted, state: zoomed },
      { frame, change: "pan", fitted, state: panViewState(zoomed, frame, 220) },
      {
        frame,
        change: "wheel zoom on both axes with a target outside the drawing",
        fitted,
        state: constrainViewState(
          { target: [12_345, -9_876], zoomX: zoomed.zoomX + 1.5, zoomY: zoomed.zoomY + 1.5 },
          frame,
        ),
      },
    ];
  });

  test.each(updates)(
    "keeps the projected cross span after $change on leaf axis $frame.leafAxis",
    ({ frame, fitted, state }) => {
      expect(crossSpan(state, frame)).toStrictEqual(crossSpan(fitted, frame));
    },
  );

  test("zooms the leaf axis by the toolbar factor", () => {
    const zoomed = zoomViewState(fitViewState(WIDE), WIDE, 2);

    expect(rowPixels(zoomed)).toBeCloseTo(4, 12);
  });

  test("never zooms out beyond the fitted view", () => {
    const state = zoomViewState(fitViewState(WIDE), WIDE, 1 / 16);

    expect(rowPixels(state)).toBeCloseTo(2, 12);
    expect(visibleLeafRange(state, WIDE)).toStrictEqual([-0.5, 299.5]);
  });

  test("stops zooming in at the largest row height", () => {
    const state = zoomViewState(fitViewState(WIDE), WIDE, 1024);

    expect(rowPixels(state)).toBeCloseTo(MAX_ROW_PX, 9);
  });

  test("clamps a pan so the drawing edge stays at the canvas edge", () => {
    const zoomed = zoomViewState(fitViewState(WIDE), WIDE, 4);
    const [top] = visibleLeafRange(panViewState(zoomed, WIDE, -1_000), WIDE);
    const [, bottom] = visibleLeafRange(panViewState(zoomed, WIDE, 1_000), WIDE);

    expect(top).toBeCloseTo(-0.5, 9);
    expect(bottom).toBeCloseTo(299.5, 9);
  });

  test("keeps a drawing smaller than the canvas centered", () => {
    const tiny: CanvasFrame = { size: { width: 800, height: 600 }, rows: 3, leafAxis: "y" };
    const state = panViewState(fitViewState(tiny), tiny, 50);

    expect(state.target[1]).toBe(1);
  });
});

describe("fitRowsViewState", () => {
  test("shows exactly the rows of a subtree with half a row of margin", () => {
    const state = fitRowsViewState(WIDE, { first: 40, last: 59 });
    const [low, high] = visibleLeafRange(state, WIDE);

    expect(low).toBeCloseTo(39.5, 9);
    expect(high).toBeCloseTo(59.5, 9);
    expect(rowPixels(state)).toBeCloseTo(30, 9);
  });

  test("accepts the rows in either order", () => {
    expect(fitRowsViewState(WIDE, { first: 59, last: 40 })).toStrictEqual(
      fitRowsViewState(WIDE, { first: 40, last: 59 }),
    );
  });

  test("centers a subtree too small to fill the canvas at the largest row height", () => {
    const state = fitRowsViewState(WIDE, { first: 100, last: 101 });

    expect(rowPixels(state)).toBeCloseTo(MAX_ROW_PX, 9);
    expect(state.target[1]).toBe(100.5);
  });

  test("shifts a subtree at the edge so the view stays inside the drawing", () => {
    const state = fitRowsViewState(WIDE, { first: 299, last: 299 });
    const [, high] = visibleLeafRange(state, WIDE);

    expect(high).toBeCloseTo(299.5, 9);
  });
});
