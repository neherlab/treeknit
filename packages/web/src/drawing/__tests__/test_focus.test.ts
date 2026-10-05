import { describe, expect, test } from "vitest";

import { focusRows, mccRows } from "../focus";
import { drawingLeafAxis, NARROW_CANVAS_PX } from "../narrow";
import { examplePairView } from "./fixtures";

const VIEW = examplePairView();

describe("focusRows", () => {
  test("spans a leaf's rows in both trees", () => {
    expect(focusRows(VIEW, { kind: "leaf", name: "C" })).toStrictEqual({ first: 2, last: 3 });
  });

  test("fits the rows of an MCC's leaves in both trees", () => {
    expect({ abcd: mccRows(VIEW, 0), x: mccRows(VIEW, 1) }).toStrictEqual({
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

describe("drawingLeafAxis", () => {
  test.each([
    [undefined, "y"],
    [NARROW_CANVAS_PX - 1, "x"],
    [NARROW_CANVAS_PX, "y"],
  ] as const)("a canvas %s px wide draws its leaves along %s", (width, axis) => {
    expect(drawingLeafAxis(width)).toBe(axis);
  });
});
