import type { MccInfo } from "@neherlab/treeknit-wasm";
import { type ColumnFiltersState, type SortingState, useTable } from "@tanstack/react-table";
import { renderToString } from "react-dom/server";
import { describe, expect, test } from "vitest";

import { leavesPreview, MCC_COLUMN, mccColumns, mccsBySize, mccTableFeatures, sortDirection } from "../mccTable";

const contains = (text: string, substring: string) => text.toLowerCase().includes(substring.toLowerCase());

const COLUMNS = mccColumns(contains);

const MCCS: MccInfo[] = [
  mcc(0, ["A/Texas/1/2004", "A/Ohio/2/2004", "A/Iowa/3/2004", "A/Utah/4/2004"], ["A/Utah/4/2004"], true),
  mcc(1, ["B/Lee/1940"], [], false),
  mcc(2, ["A/Hong Kong/1/1968", "A/Hanoi/5/2005"], [], false),
];

describe("mccsBySize", () => {
  test("lists the largest MCC first and breaks size ties by MCC number", () => {
    const tied = [...MCCS, mcc(3, ["C/1", "C/2"], [], false)];

    expect(mccsBySize(tied).map(({ index }) => index)).toStrictEqual([0, 2, 3, 1]);
  });
});

describe("mccColumns", () => {
  test("lists the MCCs in their order", () => {
    expect(shownRows([], [])).toStrictEqual([0, 1, 2]);
  });

  test.each([
    ["size, largest first", MCC_COLUMN.size, true, [0, 2, 1]],
    ["size, smallest first", MCC_COLUMN.size, false, [1, 2, 0]],
    ["first leaf name", MCC_COLUMN.leaves, false, [2, 0, 1]],
    ["imputed members, most first", MCC_COLUMN.imputed, true, [0, 1, 2]],
    ["ambiguous first", MCC_COLUMN.ambiguous, true, [0, 1, 2]],
  ] as const)("sorts by %s", (_, id, desc, order) => {
    expect(shownRows([{ id, desc }], [])).toStrictEqual(order);
  });

  test.each([
    ["a part of a leaf name, in any case", "hong kong", [2]],
    ["a part shared by the names of one MCC", "/2004", [0]],
    ["a leaf that no MCC has", "Yamagata", []],
  ] as const)("filters by %s", (_, query, rows) => {
    expect(shownRows([], [{ id: MCC_COLUMN.leaves, value: query }])).toStrictEqual(rows);
  });
});

describe("sortDirection", () => {
  test("names the TanStack sort order as the aria-sort direction of the header", () => {
    expect([sortDirection("asc"), sortDirection("desc"), sortDirection(false)]).toStrictEqual([
      "ascending",
      "descending",
      undefined,
    ]);
  });
});

describe("leavesPreview", () => {
  test.each([
    [["A"], "A"],
    [["A", "B", "C"], "A, B, C"],
    [["A", "B", "C", "D", "E"], "A, B, C and 2 more"],
  ])("shows %o as %s", (leaves, text) => {
    expect(leavesPreview(leaves)).toBe(text);
  });
});

function shownRows(sorting: SortingState, columnFilters: ColumnFiltersState): number[] {
  const text = renderToString(<Probe sorting={sorting} columnFilters={columnFilters} />);

  return text === "" ? [] : text.split(",").map(Number);
}

function Probe({ sorting, columnFilters }: { sorting: SortingState; columnFilters: ColumnFiltersState }) {
  const table = useTable({
    features: mccTableFeatures,
    columns: COLUMNS,
    data: MCCS,
    initialState: { sorting, columnFilters },
  });

  return table
    .getRowModel()
    .rows.map((row) => row.original.index)
    .join(",");
}

function mcc(index: number, leaves: string[], imputedLeaves: string[], ambiguous: boolean): MccInfo {
  return {
    index,
    size: leaves.length,
    leaves,
    imputedLeaves,
    ambiguousLeaves: ambiguous ? imputedLeaves : [],
    slot: index,
  };
}
