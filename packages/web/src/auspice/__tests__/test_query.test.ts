import { describe, expect, test } from "vitest";

import type { WrittenSearch } from "../../workspace/search";
import { auspiceQuery, withAuspiceQuery, withoutFilterValue } from "../query";
import type { AuspiceControlsState } from "../state";

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

describe("auspiceQuery", () => {
  test.each([
    [
      "scatterX, a scatter x axis other than the branch metric",
      { layout: "scatter", scatterVariables: { x: "mcc", y: "mcc" } },
      "l=scatter&scatterX=mcc",
    ],
    [
      "no scatterX for the branch metric on the x axis",
      { layout: "scatter", scatterVariables: { x: "div", y: "mcc" } },
      "l=scatter",
    ],
    [
      "scatterY, a scatter y axis other than the coloring",
      { layout: "scatter", scatterVariables: { x: "div", y: "largest_mcc" } },
      "l=scatter&scatterY=largest_mcc",
    ],
    [
      "branches=hide, hidden branches of a scatter plot",
      { layout: "scatter", scatterVariables: { x: "div", y: "mcc", showBranches: false } },
      "l=scatter&branches=hide",
    ],
    [
      "regression=show, a regression of a scatter plot",
      { layout: "scatter", scatterVariables: { x: "div", y: "mcc", showRegression: true } },
      "l=scatter&regression=show",
    ],
    [
      "regression=hide, a clock without its regression",
      { layout: "clock", scatterVariables: { showRegression: false } },
      "l=clock&regression=hide",
    ],
    [
      "no scatter keys outside a scatter plot",
      { layout: "rect", scatterVariables: { x: "mcc", y: "largest_mcc", showBranches: false, showRegression: true } },
      "",
    ],
    ["streamLabel while stream trees show", { showStreamTrees: true, streamTreeBranchLabel: "MCC" }, "streamLabel=MCC"],
    ["no streamLabel while stream trees are off", { showStreamTrees: false, streamTreeBranchLabel: "MCC" }, ""],
  ] satisfies [string, Partial<AuspiceControlsState>, string][])("writes %s", (_case, changed, expected) => {
    expect(auspiceQuery({ ...CONTROLS, ...changed })).toBe(expected);
  });
});

const CONTROLS: AuspiceControlsState = {
  defaults: { colorBy: "mcc", layout: "rect", tipLabelKey: "name", selectedBranchLabel: "MCC" },
  colorBy: "mcc",
  layout: "rect",
  tipLabelKey: "name",
  selectedBranchLabel: "MCC",
  showAllBranchLabels: false,
  distanceMeasure: "div",
  focus: null,
  branchLengthsToDisplay: "divOnly",
  panelLayout: "full",
  showTreeToo: false,
  showTangle: false,
  showStreamTrees: false,
  streamTreeBranchLabel: null,
  scatterVariables: {},
  panelsToDisplay: ["tree"],
  selectedNode: null,
  filters: {},
  performanceFlags: new Map(),
};
