import { describe, expect, test } from "vitest";

import {
  DRAWING_MARGIN_PX,
  LABEL_COLUMN_MAX_SHARE,
  LABEL_GAP_PX,
  labelColumnPx,
  LINK_ZONE_MIN_SHARE,
  tanglegramColumns,
} from "../columns";

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

  test("gives the trees the width the labels and links leave at 1232 px", () => {
    const columns = tanglegramColumns(1232, 100);

    expect([width(columns.left), width(columns.links)]).toStrictEqual([380, 240]);
  });
});
