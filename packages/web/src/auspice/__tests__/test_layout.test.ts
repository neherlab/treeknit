import { describe, expect, test } from "vitest";

import { sidebarOverlays, treeSize } from "../layout";

describe("treeSize", () => {
  test("leaves the 26 px title row and the 16 px gap below the chart out of the height", () => {
    expect(treeSize({ width: 1200.6, height: 800.4 })).toStrictEqual({ width: 1200, height: 758 });
  });

  test.each([
    ["narrower than 320 px", { width: 319, height: 800 }],
    ["without height for the tree", { width: 1200, height: 42 }],
  ])("draws nothing in an area %s", (_case, area) => {
    expect(treeSize(area)).toBeNull();
  });

  test("draws a tree of one pixel row and 320 px width at the limits", () => {
    expect(treeSize({ width: 320, height: 43 })).toStrictEqual({ width: 320, height: 1 });
  });
});

describe("sidebarOverlays", () => {
  test.each([
    [899, true],
    [900, false],
  ])("an Auspice view %i px wide overlays the controls: %s", (width, overlays) => {
    expect(sidebarOverlays(width)).toBe(overlays);
  });
});
