import { describe, expect, test } from "vitest";

import { examplePairView } from "../../drawing/__tests__/fixtures";
import type { PairTarget } from "../../drawing/selection";
import { pairTargetRows } from "../picking";

const VIEW = examplePairView();

describe("pairTargetRows", () => {
  test.each([
    ["a ribbon spans its block in both trees", { kind: "ribbon", block: 1 }, { first: 2, last: 4 }],
    ["a link spans both copies of its leaf", { kind: "link", link: 4 }, { first: 2, last: 4 }],
    ["a node spans the leaves under it in its tree", { kind: "node", side: "right", node: 6 }, { first: 3, last: 4 }],
    ["a missing ribbon has no rows", { kind: "ribbon", block: 9 }, null],
  ] satisfies [string, PairTarget, unknown][])("%s", (_, target, expected) => {
    expect(pairTargetRows(VIEW, target)).toStrictEqual(expected);
  });
});
