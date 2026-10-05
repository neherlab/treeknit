import { describe, expect, test } from "vitest";

import { exampleConstellation } from "../../drawing/__tests__/fixtures";
import { cellLabel, constellationRows, pairColumns } from "../constellation";

const TABLE = exampleConstellation();

describe("constellationRows", () => {
  test("keeps the leaf order of the table and an empty cell where a pair lacks the leaf", () => {
    expect(constellationRows(TABLE).at(-1)).toStrictEqual({
      leaf: "Z",
      cells: [null, null, { mcc: 0, size: 1, slot: 1 }],
    });
  });
});

describe("pairColumns", () => {
  test("has one column per pair, keyed by the pair index", () => {
    expect(pairColumns(TABLE)).toStrictEqual([
      { id: "pair-0", title: "ha and na", pair: 0 },
      { id: "pair-1", title: "ha and mp", pair: 1 },
      { id: "pair-2", title: "na and mp", pair: 2 },
    ]);
  });
});

describe("constellationRows of a table without a row of cells for a leaf", () => {
  test("rejects the table instead of showing the leaf in no pair", () => {
    expect(() => constellationRows({ ...TABLE, cells: TABLE.cells.slice(0, 1) })).toThrow(RangeError);
  });
});

describe("cellLabel", () => {
  test("names the MCC, its size, and the navigation target", () => {
    expect(cellLabel({ mcc: 1, size: 2, slot: 2 }, "A", "ha and mp")).toBe("MCC 2 of 2 leaves: show A in ha and mp");
  });
});
