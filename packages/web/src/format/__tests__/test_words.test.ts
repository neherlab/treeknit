import { describe, expect, test } from "vitest";

import { NONE, yesNo } from "../words";

describe("value words", () => {
  test("writes yes, no, and none in sentence case, the same in every table and fact", () => {
    expect([yesNo(true), yesNo(false), NONE]).toStrictEqual(["Yes", "No", "None"]);
  });
});
