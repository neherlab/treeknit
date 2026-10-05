import { describe, expect, test } from "vitest";

import { stepTooltip } from "../stepTooltip";

describe("stepTooltip", () => {
  test("shows the localized label of a step button, and no tooltip for a missing or empty label", () => {
    expect([stepTooltip("Increase γ"), stepTooltip(undefined), stepTooltip("")]).toStrictEqual([
      "Increase γ",
      null,
      null,
    ]);
  });
});
