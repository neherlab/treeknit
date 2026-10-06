import { describe, expect, test } from "vitest";

import { type PairRef, WORKSPACE_SEARCH_DEFAULTS, type WrittenSearch } from "../../workspace/search";
import { leafInPair, mccInTanglegram } from "../navigation";

const PAIR_LABELS: readonly PairRef[] = [
  ["ha", "na"],
  ["ha", "pb1"],
  ["na", "pb1"],
];

const SEARCH: WrittenSearch = {
  ...WORKSPACE_SEARCH_DEFAULTS,
  view: "constellation",
  version: "imputed",
  mcc: 2,
  node: { side: "left", name: "NODE_4" },
};

describe("leafInPair", () => {
  test("leaves the first pair unwritten", () => {
    expect(leafInPair(SEARCH, PAIR_LABELS, 0, "A").pair).toBeUndefined();
  });

  test("opens the tanglegram of the pair with only the leaf selected", () => {
    expect(leafInPair(SEARCH, PAIR_LABELS, 2, "A/New York/392/2004")).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      view: "tanglegram",
      version: "imputed",
      pair: ["na", "pb1"],
      leaf: "A/New York/392/2004",
    });
  });
});

describe("mccInTanglegram", () => {
  test("opens the tanglegram of the same pair with only the MCC selected", () => {
    expect(mccInTanglegram({ ...SEARCH, view: "mccs", pair: ["ha", "pb1"], leaf: "A" }, 4)).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      view: "tanglegram",
      version: "imputed",
      pair: ["ha", "pb1"],
      mcc: 4,
    });
  });
});
