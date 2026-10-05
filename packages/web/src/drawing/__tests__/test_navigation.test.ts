import { describe, expect, test } from "vitest";

import { WORKSPACE_SEARCH_DEFAULTS, type WorkspaceSearch } from "../../workspace/search";
import { leafInPair, mccInTanglegram } from "../navigation";

const SEARCH: WorkspaceSearch = {
  ...WORKSPACE_SEARCH_DEFAULTS,
  view: "constellation",
  version: "imputed",
  mcc: 2,
  node: { side: "left", name: "NODE_4" },
};

describe("leafInPair", () => {
  test("opens the tanglegram of the pair with only the leaf selected", () => {
    expect(leafInPair(SEARCH, 2, "A/New York/392/2004")).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      view: "tanglegram",
      version: "imputed",
      pair: 2,
      leaf: "A/New York/392/2004",
    });
  });
});

describe("mccInTanglegram", () => {
  test("opens the tanglegram of the same pair with only the MCC selected", () => {
    expect(mccInTanglegram({ ...SEARCH, view: "mccs", pair: 1, leaf: "A" }, 4)).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      view: "tanglegram",
      version: "imputed",
      pair: 1,
      mcc: 4,
    });
  });
});
