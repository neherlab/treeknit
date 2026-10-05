import { OrthographicViewport } from "@deck.gl/core";
import { describe, expect, test } from "vitest";

import {
  type CanvasFrame,
  constrainViewState,
  fitRowsViewState,
  fitViewState,
  MAX_ROW_PX,
  minimapLeafAt,
  minimapShown,
  minimapSize,
  minimapViewState,
  panViewState,
  rowPixels,
  snappedZoom,
  subpixelZoom,
  type TreeViewState,
  visibleLeafRange,
  visibleWorldRect,
  worldPosition,
  zoomLimits,
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
          zoomed,
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

describe("snappedZoom", () => {
  const ODD: CanvasFrame = { size: { width: 800, height: 300 }, rows: 123, leafAxis: "y" };
  const [minZoom, maxZoom] = zoomLimits(ODD);
  const snap = subpixelZoom(ODD);

  test("returns to the exact fitted zoom after zooming in and out by the same toolbar steps", () => {
    const zoomedIn = [1, 2, 3].reduce((state) => zoomViewState(state, ODD, 2), fitViewState(ODD));
    const zoomedOut = [1, 2, 3].reduce((state) => zoomViewState(state, ODD, 1 / 2), zoomedIn);

    expect(zoomedOut.zoomY).toBe(minZoom);
  });

  test.each([
    { name: "below the fitted zoom", zoom: minZoom - 1, snapped: minZoom },
    { name: "within a pixel of the fitted zoom", zoom: minZoom + snap / 2, snapped: minZoom },
    { name: "two pixels above the fitted zoom", zoom: minZoom + 2 * snap, snapped: minZoom + 2 * snap },
    { name: "within a pixel of the largest zoom", zoom: maxZoom - snap / 2, snapped: maxZoom },
    { name: "above the largest zoom", zoom: maxZoom + 1, snapped: maxZoom },
  ])("snaps a zoom $name to $snapped", ({ zoom, snapped }) => {
    expect(snappedZoom(ODD, zoom)).toBe(snapped);
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

describe("minimap", () => {
  const LARGE: CanvasFrame = { size: { width: 900, height: 800 }, rows: 2_000, leafAxis: "y" };

  test("appears only for drawings with more than 200 leaves", () => {
    expect(minimapShown({ ...LARGE, rows: 200 })).toBe(false);
    expect(minimapShown({ ...LARGE, rows: 201 })).toBe(true);
  });

  test("stays hidden when it would cover more than half the leaf axis", () => {
    expect(minimapShown({ ...LARGE, size: { width: 900, height: 399 } })).toBe(false);
    expect(minimapShown({ ...LARGE, size: { width: 900, height: 400 } })).toBe(true);
  });

  test("is 160 px across the cross axis", () => {
    expect(minimapSize(LARGE)).toStrictEqual({ width: 160, height: 200 });
    expect(minimapSize(NARROW)).toStrictEqual({ width: 200, height: 160 });
  });

  test.each([LARGE, { ...LARGE, leafAxis: "x" as const }])(
    "projects the whole drawing onto the minimap (%o)",
    (frame) => {
      const mini = minimapViewState(frame);
      const viewport = new OrthographicViewport({ ...minimapSize(frame), ...mini });
      const cross = frame.leafAxis === "y" ? frame.size.width : frame.size.height;
      const [left = Number.NaN, top = Number.NaN] = viewport.project(worldPosition(frame.leafAxis, 0, -0.5));
      const [right = Number.NaN, bottom = Number.NaN] = viewport.project(worldPosition(frame.leafAxis, cross, 1_999.5));
      const size = minimapSize(frame);

      expect(left).toBeCloseTo(0, 9);
      expect(top).toBeCloseTo(0, 9);
      expect(right).toBeCloseTo(size.width, 9);
      expect(bottom).toBeCloseTo(size.height, 9);
    },
  );

  test("frames the visible rows of the main view across the whole width", () => {
    const state = fitRowsViewState(LARGE, { first: 500, last: 999 });
    const corners = visibleWorldRect(state, LARGE).flat();

    expect(corners.map((value) => Math.round(value * 1e9) / 1e9)).toStrictEqual([
      0, 499.5, 900, 499.5, 900, 999.5, 0, 999.5,
    ]);
  });

  test("maps a pointer offset back to a leaf row", () => {
    expect(minimapLeafAt(LARGE, 0)).toBe(-0.5);
    expect(minimapLeafAt(LARGE, 100)).toBe(999.5);
    expect(minimapLeafAt(LARGE, 200)).toBe(1_999.5);
  });
});
