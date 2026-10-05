import { describe, expect, test } from "vitest";

import { exampleConstellation } from "../../drawing/__tests__/fixtures";
import { cellLabel, constellationColumns, constellationRows, LEAF_COLUMN, pairColumnId } from "../constellation";

const TABLE = exampleConstellation();

describe("constellationRows", () => {
  test("keeps the leaf order of the table and an empty cell where a pair lacks the leaf", () => {
    expect(constellationRows(TABLE).at(-1)).toStrictEqual({
      leaf: "Z",
      cells: [null, null, { mcc: 0, size: 1, slot: 1 }],
    });
  });
});

describe("constellationColumns", () => {
  test("starts with the leaf and has one column per pair", () => {
    expect(constellationColumns(TABLE).map((definition) => [definition.id, definition.header])).toStrictEqual([
      [LEAF_COLUMN, "Leaf"],
      [pairColumnId(0), "ha and na"],
      [pairColumnId(1), "ha and mp"],
      [pairColumnId(2), "na and mp"],
    ]);
  });
});

describe("cellLabel", () => {
  test("names the MCC, its size, and the navigation target", () => {
    expect(cellLabel({ mcc: 1, size: 2, slot: 2 }, "A", "ha and mp")).toBe("MCC 2 of 2 leaves: show A in ha and mp");
  });
});
