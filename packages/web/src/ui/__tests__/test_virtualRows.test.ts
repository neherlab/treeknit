import { describe, expect, test } from "vitest";

import { virtualGaps } from "../useVirtualRows";

describe("virtualGaps", () => {
  test("pads the rows above and below the rendered window below a 32 px header", () => {
    expect(
      virtualGaps(
        [
          { start: 352, end: 384 },
          { start: 384, end: 416 },
        ],
        3200,
        32,
      ),
    ).toStrictEqual({
      before: 320,
      after: 2816,
    });
  });

  test("needs no padding without rows", () => {
    expect(virtualGaps([], 0, 32)).toStrictEqual({ before: 0, after: 0 });
  });
});
