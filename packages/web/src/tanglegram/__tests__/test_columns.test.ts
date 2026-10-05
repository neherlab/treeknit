import { describe, expect, test } from "vitest";

import { DRAWING_MARGIN_PX, LABEL_GAP_PX } from "../../drawing/spacing";
import { LABEL_COLUMN_MAX_SHARE, labelColumnPx, LINK_ZONE_MIN_SHARE, tanglegramColumns } from "../columns";

function width({ start, end }: { start: number; end: number }) {
  return end - start;
}

describe("labelColumnPx", () => {
  test("fits the longest label with a gap on both sides", () => {
    expect(labelColumnPx(1232, 80)).toBe(80 + 2 * LABEL_GAP_PX);
  });

  test("takes at most a quarter of half the drawing", () => {
    expect(labelColumnPx(1232, 900)).toBe(LABEL_COLUMN_MAX_SHARE * ((1232 - 2 * DRAWING_MARGIN_PX) / 2));
  });

  test("takes no space without labels", () => {
    expect(labelColumnPx(1232, 0)).toBe(0);
  });
});

describe("tanglegramColumns", () => {
  test.each([
    [1232, 0],
    [1232, 92],
    [800, 40],
    [480, 0],
  ])("at %i px with %i px label columns the columns tile the drawing", (crossPx, labelPx) => {
    const columns = tanglegramColumns(crossPx, labelPx);

    expect({
      edges: [
        columns.left.start,
        columns.left.end === columns.leftLabels.start,
        columns.leftLabels.end === columns.links.start,
        columns.links.end === columns.rightLabels.start,
        columns.rightLabels.end === columns.right.start,
        columns.right.end,
      ],
      labelsFit: [columns.leftLabels, columns.rightLabels].every((column) => Math.abs(width(column) - labelPx) < 1e-9),
      treesMatch: Math.abs(width(columns.left) - width(columns.right)) < 1e-9,
      linkShare: width(columns.links) / (crossPx - 2 * DRAWING_MARGIN_PX) >= LINK_ZONE_MIN_SHARE,
      mirrored: [columns.left.mirrored, columns.right.mirrored],
    }).toStrictEqual({
      edges: [DRAWING_MARGIN_PX, true, true, true, true, crossPx - DRAWING_MARGIN_PX],
      labelsFit: true,
      treesMatch: true,
      linkShare: true,
      mirrored: [false, true],
    });
  });

  test.each([
    ["no labels: links take 20%", 0, { tree: 480, links: 240 }],
    ["short labels take their width from the link zone, so the trees keep theirs", 20, { tree: 480, links: 200 }],
    ["labels reach the 15% minimum of the link zone", 30, { tree: 480, links: 180 }],
    ["wider labels narrow the trees once the link zone is at 15%", 100, { tree: 410, links: 180 }],
  ] as const)("at 1232 px, %s", (_, labelPx, expected) => {
    const columns = tanglegramColumns(1232, labelPx);

    expect({ tree: width(columns.left), links: width(columns.links) }).toStrictEqual(expected);
  });
});
