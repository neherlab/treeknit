import { describe, expect, test } from "vitest";

import { focusRows, mccRows } from "../focus";
import { drawingLeafAxis, NARROW_CANVAS_PX } from "../narrow";
import { examplePairView } from "./fixtures";

const VIEW = examplePairView();

describe("focusRows", () => {
  test("centers a leaf on its row in the left tree", () => {
    expect(focusRows(VIEW, { kind: "leaf", name: "C" })).toStrictEqual({ first: 2, last: 2 });
  });

  test("fits the rows of an MCC's leaves in the left tree", () => {
    expect(mccRows(VIEW, 0)).toStrictEqual({ first: 0, last: 3 });
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
