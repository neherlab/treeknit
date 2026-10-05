import { describe, expect, test } from "vitest";

import { INSPECTOR_PANE_MIN_WIDTH_PX, RAIL_PANE_MIN_WIDTH_PX, SHELL_MEDIA_QUERIES, shellLayout } from "../layout";

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

  test("keeps the center at least 640 px wide plus padding when both panes show", () => {
    const railWidthPx = 336;
    const inspectorWidthPx = 320;
    const centerMinWidthPx = 640;

    expect(INSPECTOR_PANE_MIN_WIDTH_PX - railWidthPx - inspectorWidthPx).toBeGreaterThan(centerMinWidthPx);
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
