import { describe, expect, test } from "vitest";

import { zoomButtonStates, zoomRoots, type ZoomTree } from "../zoom";

describe("zoomButtonStates", () => {
  test.each<{ name: string; trees: ZoomTree[]; expected: ReturnType<typeof zoomButtonStates> }>([
    {
      name: "trees at their root and unfiltered",
      trees: [tree(0, undefined), tree(0, undefined)],
      expected: { out: false, selected: false, root: false },
    },
    {
      name: "a filter whose clade is the root",
      trees: [tree(0, 0)],
      expected: { out: false, selected: false, root: false },
    },
    {
      name: "a filter whose clade is not in view as root",
      trees: [tree(0, 7)],
      expected: { out: false, selected: true, root: false },
    },
    {
      name: "a tree zoomed to the clade of its filter",
      trees: [tree(7, 7)],
      expected: { out: true, selected: false, root: true },
    },
    {
      name: "only the second tree zoomed",
      trees: [tree(0, undefined), tree(4, undefined)],
      expected: { out: true, selected: false, root: true },
    },
  ])("$name", ({ trees, expected }) => {
    expect(zoomButtonStates(trees)).toStrictEqual(expected);
  });
});

describe("zoomRoots", () => {
  test("zooms out each zoomed tree to the parent of its in-view root and leaves the others", () => {
    expect(zoomRoots("out", [tree(5, undefined, 2), tree(0, undefined, 9)])).toStrictEqual([2, undefined]);
  });

  test("zooms each filtered tree to its filtered root", () => {
    expect(zoomRoots("selected", [tree(0, 6), tree(3, 3)])).toStrictEqual([6, undefined]);
  });

  test("returns each zoomed tree to its root", () => {
    expect(zoomRoots("root", [tree(5, undefined), tree(4, undefined)])).toStrictEqual([0, 0]);
  });

  test("leaves the second tree alone when only one tree is shown", () => {
    expect(zoomRoots("root", [tree(5, undefined), undefined])).toStrictEqual([0, undefined]);
  });
});

function tree(inViewRoot: number, filteredRoot: number | undefined, parent = 0): ZoomTree {
  return { inViewRoot, filteredRoot, parentOfInViewRoot: () => parent };
}
