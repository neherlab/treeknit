import { describe, expect, test } from "vitest";

import { pickTarget, reportingTo } from "../picking";

const PICKS = {
  first: (items: readonly string[], index: number) => items[index],
  last: (items: readonly string[], index: number) => items.at(-1 - index),
};

describe("pickTarget", () => {
  test("asks the pick function of the layer", () => {
    expect([pickTarget(PICKS, ["a", "b"], "first", 0), pickTarget(PICKS, ["a", "b"], "last", 0)]).toStrictEqual([
      "a",
      "b",
    ]);
  });

  test("picks nothing on a layer without a pick function", () => {
    expect(pickTarget(PICKS, ["a"], "leaders", 0)).toBeUndefined();
  });
});

describe("reportingTo", () => {
  test("passes the result of a handler through and reports nothing", () => {
    const reported: unknown[] = [];

    const guarded = reportingTo((error) => {
      reported.push(error);
    });

    expect([guarded((x: number) => x * 2, 0)(21), reported]).toStrictEqual([42, []]);
  });

  test("reports a throw of a picking handler to the drawing's error boundary and returns the fallback", () => {
    const reported: unknown[] = [];
    const failure = new RangeError("The display data refers to MCC 9, but it has 2 of them");

    const guarded = reportingTo((error) => {
      reported.push(error);
    });

    const tooltip = guarded((): string[] | null => {
      throw failure;
    }, null);

    expect([tooltip(), reported]).toStrictEqual([null, [failure]]);
  });
});
