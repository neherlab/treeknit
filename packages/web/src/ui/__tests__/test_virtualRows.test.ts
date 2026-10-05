import { describe, expect, test } from "vitest";

import { virtualGaps } from "../useVirtualRows";

describe("virtualGaps", () => {
  test("pads the rows above and below the rendered window", () => {
    expect(
      virtualGaps(
        [
          { start: 320, end: 352 },
          { start: 352, end: 384 },
        ],
        3200,
      ),
    ).toStrictEqual({
      before: 320,
      after: 2816,
    });
  });

  test("needs no padding without rows", () => {
    expect(virtualGaps([], 0)).toStrictEqual({ before: 0, after: 0 });
  });
});
