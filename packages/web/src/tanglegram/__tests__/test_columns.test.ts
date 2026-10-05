import { describe, expect, test } from "vitest";

import { exampleDrawingRules } from "../../drawing/__tests__/fixtures";
import { tanglegramColumns } from "../columns";

const RULES = exampleDrawingRules();

function width({ start, end }: { start: number; end: number }) {
  return end - start;
}

describe("tanglegramColumns", () => {
  test.each([
    [1232, 0, 0],
    [1232, 80, 92],
    [800, 28, 40],
    [480, 0, 0],
  ])(
    "at %i px a %i px label takes %i px label columns, and the columns tile the drawing",
    (crossPx, longestPx, labelPx) => {
      const columns = tanglegramColumns(crossPx, longestPx, RULES);

      expect({
        edges: [
          columns.left.start,
          columns.left.end === columns.leftLabels.start,
          columns.leftLabels.end === columns.links.start,
          columns.links.end === columns.rightLabels.start,
          columns.rightLabels.end === columns.right.start,
          columns.right.end,
        ],
        labelsFit: [columns.leftLabels, columns.rightLabels].every(
          (column) => Math.abs(width(column) - labelPx) < 1e-9,
        ),
        treesMatch: Math.abs(width(columns.left) - width(columns.right)) < 1e-9,
        linkShare: width(columns.links) / (crossPx - 2 * RULES.marginPx) >= RULES.linkZoneMinShare,
        mirrored: [columns.left.mirrored, columns.right.mirrored],
      }).toStrictEqual({
        edges: [RULES.marginPx, true, true, true, true, crossPx - RULES.marginPx],
        labelsFit: true,
        treesMatch: true,
        linkShare: true,
        mirrored: [false, true],
      });
    },
  );

  test.each([
    ["no labels: links take 20%", 0, { tree: 480, labels: 0, links: 240 }],
    [
      "short labels take their width from the link zone, so the trees keep theirs",
      8,
      { tree: 480, labels: 20, links: 200 },
    ],
    ["labels reach the 15% minimum of the link zone", 18, { tree: 480, labels: 30, links: 180 }],
    ["wider labels narrow the trees once the link zone is at 15%", 88, { tree: 410, labels: 100, links: 180 }],
    ["a label column takes at most a quarter of half the inner width", 500, { tree: 360, labels: 150, links: 180 }],
  ] as const)("at 1232 px, %s", (_, longestPx, expected) => {
    const columns = tanglegramColumns(1232, longestPx, RULES);

    expect({ tree: width(columns.left), labels: width(columns.leftLabels), links: width(columns.links) }).toStrictEqual(
      expected,
    );
  });
});
