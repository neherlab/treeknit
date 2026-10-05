import { describe, expect, test } from "vitest";

import { pickTarget } from "../picking";

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
