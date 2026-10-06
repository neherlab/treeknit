import { describe, expect, test } from "vitest";

import type { WrittenSearch } from "../../workspace/search";
import { withAuspiceQuery, withoutFilterValue } from "../query";

const SEARCH: WrittenSearch = {
  view: "auspice",
  version: "resolved",
  scale: "div",
  labels: "auto",
};

describe("withoutFilterValue", () => {
  test.each([
    ["the only value of the filter", "c=x&f_mcc=MCC+2", "MCC 2", "c=x"],
    ["one of several values", "f_mcc=MCC+1%2CMCC+2", "MCC 2", "f_mcc=MCC+1"],
    ["a value the filter lacks", "f_mcc=MCC+1", "MCC 2", "f_mcc=MCC+1"],
    ["nothing when no value is marked", "f_mcc=MCC+2", null, "f_mcc=MCC+2"],
    ["nothing when the query has no such filter", "c=x", "MCC 2", "c=x"],
  ])("removes %s", (_case, text, value, expected) => {
    expect(withoutFilterValue(text, "f_mcc", value)).toBe(expected);
  });
});

describe("withAuspiceQuery", () => {
  test("drops the parameter for an empty query and keeps a query otherwise", () => {
    const search = { ...SEARCH, auspice: "c=x" };

    expect([withAuspiceQuery(search, "").auspice, withAuspiceQuery(search, "l=radial").auspice]).toStrictEqual([
      undefined,
      "l=radial",
    ]);
  });
});
