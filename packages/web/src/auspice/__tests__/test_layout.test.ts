import { controlsPadding, controlsWidth } from "auspice/src/util/globals";
import { describe, expect, test } from "vitest";

import { cardTitleHeight, headerLayout, SIDEBAR_WIDTH_PX, sidebarOverlays, treeSize } from "../layout";

describe("treeSize", () => {
  test("takes Auspice's full panel: 40 px less width, 88 % of the height less 52 px (auspice computeResponsive)", () => {
    expect(treeSize({ width: 1240.6, height: 800 }, false)).toStrictEqual({ width: 1200, height: 652 });
  });

  test("leaves the 26 px header row out of the height while the header is stacked", () => {
    expect(treeSize({ width: 1240, height: 800 }, true)).toStrictEqual({ width: 1200, height: 626 });
  });

  test.each([
    ["narrower than 320 px of tree", { width: 359, height: 800 }],
    ["without height for the tree", { width: 1200, height: 59 }],
  ])("draws nothing in an area %s", (_case, area) => {
    expect(treeSize(area, false)).toBeNull();
  });

  test("draws a tree of 320 px width at the limit", () => {
    expect(treeSize({ width: 360, height: 800 }, false)).toStrictEqual({ width: 320, height: 652 });
  });
});

describe("sidebarOverlays", () => {
  test.each([
    [780, true],
    [781, false],
  ])("an Auspice view %i px wide overlays the controls: %s (auspice: open above 780 px)", (width, overlays) => {
    expect(sidebarOverlays(width)).toBe(overlays);
  });
});

describe("sidebar width", () => {
  test("is Auspice's sidebar width, the controls and their padding", () => {
    expect(SIDEBAR_WIDTH_PX).toBe(controlsWidth + controlsPadding);
  });
});

describe("cardTitleHeight", () => {
  test("leaves room for the tree names that Auspice writes 25 px below the card top with two trees", () => {
    expect([cardTitleHeight(false), cardTitleHeight(true)]).toStrictEqual([26, 46]);
  });
});

describe("headerLayout", () => {
  test.each([
    ["shares the title row when 120 px of summary fit", { tree: 800, zoom: 300, downloads: 148 }, false],
    ["stacks when less than 120 px of summary fit", { tree: 800, zoom: 300, downloads: 349 }, true],
  ])("%s next to the zoom group", (_case, widths, stacked) => {
    expect(headerLayout(widths).stacked).toBe(stacked);
  });

  test.each([
    [599, true],
    [600, false],
  ])("folds the downloads into one menu below 600 px of tree: %i px gives %s", (tree, compactDownloads) => {
    expect(headerLayout({ tree, zoom: 0, downloads: 0 }).compactDownloads).toBe(compactDownloads);
  });
});
