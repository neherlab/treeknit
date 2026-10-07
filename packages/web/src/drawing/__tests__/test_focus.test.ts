import { describe, expect, test } from "vitest";

import type { RowRange } from "../../canvas/viewState";
import { focusApplies, focusRows, revealLeafRows } from "../focus";
import { examplePairView } from "./fixtures";

const VIEW = examplePairView();

describe("focusRows", () => {
  test("spans a leaf's rows in both trees", () => {
    expect(focusRows(VIEW, { kind: "leaf", name: "C" })).toStrictEqual({ first: 2, last: 3 });
  });

  test("fits the rows of an MCC's leaves in both trees", () => {
    expect({
      abcd: focusRows(VIEW, { kind: "mcc", mcc: 0 }),
      x: focusRows(VIEW, { kind: "mcc", mcc: 1 }),
    }).toStrictEqual({
      abcd: { first: 0, last: 4 },
      x: { first: 2, last: 4 },
    });
  });

  test("gives no rows for an unknown leaf or MCC", () => {
    expect({
      leaf: focusRows(VIEW, { kind: "leaf", name: "nope" }),
      mcc: focusRows(VIEW, { kind: "mcc", mcc: 7 }),
    }).toStrictEqual({ leaf: null, mcc: null });
  });
});

describe("revealLeafRows", () => {
  function recorded(range: RowRange) {
    const calls: [string, number | RowRange][] = [];

    revealLeafRows(
      {
        panTo: (leaf) => {
          calls.push(["panTo", leaf]);
        },
        fitRows: (rows) => {
          calls.push(["fitRows", rows]);
        },
      },
      range,
    );

    return calls;
  }

  test("pans to a leaf with one row, keeping the zoom", () => {
    expect(recorded({ first: 7, last: 7 })).toStrictEqual([["panTo", 7]]);
  });

  test("fits both copies of a leaf in rows 10 and 5,000, so neither copy is left off screen", () => {
    expect(recorded({ first: 10, last: 5000 })).toStrictEqual([["fitRows", { first: 10, last: 5000 }]]);
  });
});

describe("focusApplies", () => {
  test("applies a request for any pair at once, and a request for one pair only once that pair is shown", () => {
    expect([focusApplies({ pair: null }, 0), focusApplies({ pair: 2 }, 0), focusApplies({ pair: 2 }, 2)]).toStrictEqual(
      [true, false, true],
    );
  });
});
