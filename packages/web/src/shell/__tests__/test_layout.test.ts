import { describe, expect, test } from "vitest";

import {
  CENTER_MIN_WIDTH_PX,
  INSPECTOR_PANE_MIN_WIDTH_PX,
  INSPECTOR_WIDTH_CLASS,
  INSPECTOR_WIDTH_PX,
  keepSheetOpen,
  RAIL_PANE_MIN_WIDTH_PX,
  RAIL_WIDTH_CLASS,
  RAIL_WIDTH_PX,
  SHELL_MEDIA_QUERIES,
  shellLayout,
} from "../layout";

describe("shellLayout", () => {
  test.each([
    { width: 1920, rail: "pane", inspector: "pane" },
    { width: 1320, rail: "pane", inspector: "pane" },
    { width: 1319, rail: "pane", inspector: "sheet" },
    { width: 1280, rail: "pane", inspector: "sheet" },
    { width: 1024, rail: "pane", inspector: "sheet" },
    { width: 1023, rail: "sheet", inspector: "sheet" },
    { width: 390, rail: "sheet", inspector: "sheet" },
  ] as const)("places the rail as $rail and the inspector as $inspector at $width px", ({ width, rail, inspector }) => {
    expect(shellLayout(width)).toStrictEqual({ rail, inspector });
  });

  test("keeps the center at its minimum width when both panes show", () => {
    expect(INSPECTOR_PANE_MIN_WIDTH_PX - RAIL_WIDTH_PX - INSPECTOR_WIDTH_PX).toBeGreaterThanOrEqual(
      CENTER_MIN_WIDTH_PX,
    );
  });

  test.each([
    { pane: "rail", widthClass: RAIL_WIDTH_CLASS, widthPx: RAIL_WIDTH_PX },
    { pane: "inspector", widthClass: INSPECTOR_WIDTH_CLASS, widthPx: INSPECTOR_WIDTH_PX },
  ])("sizes the $pane by its width in pixels", ({ widthClass, widthPx }) => {
    expect(widthClass).toBe(`w-[min(${widthPx}px,100vw-48px)]`);
  });

  test("keeps the center at its minimum width when only the rail shows as a pane", () => {
    expect(shellLayout(RAIL_PANE_MIN_WIDTH_PX)).toStrictEqual({ rail: "pane", inspector: "sheet" });
    expect(RAIL_PANE_MIN_WIDTH_PX - RAIL_WIDTH_PX).toBeGreaterThanOrEqual(CENTER_MIN_WIDTH_PX);
  });

  test("returns the same object for widths with the same layout", () => {
    expect(shellLayout(1400)).toBe(shellLayout(2000));
  });

  test("watches the media queries of both breakpoints", () => {
    expect(SHELL_MEDIA_QUERIES).toStrictEqual([
      `(min-width: ${RAIL_PANE_MIN_WIDTH_PX}px)`,
      `(min-width: ${INSPECTOR_PANE_MIN_WIDTH_PX}px)`,
    ]);
  });
});

describe("keepSheetOpen", () => {
  test("closes a sheet that became a pane, so it stays closed when it turns back into a sheet", () => {
    const narrow = shellLayout(390).rail;
    const wide = shellLayout(1920).rail;

    const afterWide = keepSheetOpen(true, wide);
    const afterNarrow = keepSheetOpen(afterWide, narrow);

    expect([afterWide, afterNarrow]).toStrictEqual([false, false]);
  });

  test("keeps an open sheet open while it stays a sheet", () => {
    expect(keepSheetOpen(true, "sheet")).toBe(true);
  });
});
