import { describe, expect, test } from "vitest";

import { swatchClass } from "../MccSwatch";

describe("swatchClass", () => {
  test("maps a slot to its MCC color token and no slot to the no-MCC token", () => {
    expect([swatchClass(0), swatchClass(7), swatchClass(null)]).toStrictEqual(["bg-mcc-0", "bg-mcc-7", "bg-mcc-none"]);
  });

  test("rejects a slot outside the palette instead of drawing the no-MCC color", () => {
    expect(() => swatchClass(8)).toThrow(new RangeError("MCC color slot 8 is outside the 8 palette slots"));
  });
});
