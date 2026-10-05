import { describe, expect, test } from "vitest";

import { longestLabelPx } from "../labelWidth";
import { LEAF_SEARCH_LIMIT, matchingLeaves } from "../leafSearch";

const contains = (text: string, substring: string) => text.toLowerCase().includes(substring.toLowerCase());

const NAMES = ["A/New York/392/2004", "A/Hong Kong/1/1968", "B/Yamagata/16/1988", "A/new york/55/2005"];

describe("matchingLeaves", () => {
  test("finds names containing the query, in their order", () => {
    expect(matchingLeaves(NAMES, "new york", contains)).toStrictEqual({
      shown: ["A/New York/392/2004", "A/new york/55/2005"],
      total: 2,
    });
  });

  test("shows nothing for an empty query", () => {
    expect(matchingLeaves(NAMES, "  ", contains)).toStrictEqual({ shown: [], total: 0 });
  });

  test("shows at most the limit and counts every match", () => {
    const many = Array.from({ length: LEAF_SEARCH_LIMIT + 7 }, (_, index) => `leaf ${String(index)}`);
    const matches = matchingLeaves(many, "leaf", contains);

    expect({ shown: matches.shown.length, total: matches.total }).toStrictEqual({
      shown: LEAF_SEARCH_LIMIT,
      total: LEAF_SEARCH_LIMIT + 7,
    });
  });
});

describe("longestLabelPx", () => {
  test("measures the shortened form of each label", () => {
    const long = "x".repeat(60);

    expect(longestLabelPx(["ab", long, "abcd"], (text) => text.length, 40)).toBe(40);
  });

  test("is zero without labels", () => {
    expect(longestLabelPx([], (text) => text.length, 40)).toBe(0);
  });
});
