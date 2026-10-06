import { describe, expect, test } from "vitest";

import { NO_WORKSPACE, type WorkspaceAvailability } from "../../workspace/search";
import { UNAVAILABLE_VIEW_TOOLTIP, viewTabs } from "../viewTabs";

function tabsOf(availability: Partial<WorkspaceAvailability>) {
  return viewTabs({ ...NO_WORKSPACE, ...availability }).map(({ view, isDisabled }) => ({ view, isDisabled }));
}

describe("viewTabs", () => {
  test("enables only the overview before a run, with Auspice after the tanglegram", () => {
    expect(tabsOf({ treeCount: 1 })).toStrictEqual([
      { view: "overview", isDisabled: false },
      { view: "tanglegram", isDisabled: true },
      { view: "auspice", isDisabled: true },
      { view: "mccs", isDisabled: true },
      { view: "files", isDisabled: true },
      { view: "diagnostics", isDisabled: true },
    ]);
  });

  test("shows the ARG tab, disabled, for two trees before a run", () => {
    expect(tabsOf({ treeCount: 2 })).toContainEqual({ view: "arg", isDisabled: true });
  });

  test("enables every tab of a two-tree result and hides the constellation", () => {
    expect(tabsOf({ hasResult: true, treeCount: 2, resultTreeCount: 2, pairLabels: [["ha", "na"]] })).toStrictEqual([
      { view: "overview", isDisabled: false },
      { view: "tanglegram", isDisabled: false },
      { view: "auspice", isDisabled: false },
      { view: "arg", isDisabled: false },
      { view: "mccs", isDisabled: false },
      { view: "files", isDisabled: false },
      { view: "diagnostics", isDisabled: false },
    ]);
  });

  test("shows the constellation and hides the ARG for a three-tree result", () => {
    const views = tabsOf({
      hasResult: true,
      treeCount: 3,
      resultTreeCount: 3,
      pairLabels: [
        ["a", "b"],
        ["a", "c"],
        ["b", "c"],
      ],
    }).map(({ view }) => view);

    expect(views).toStrictEqual(["overview", "tanglegram", "auspice", "mccs", "constellation", "files", "diagnostics"]);
  });

  test("follows the trees of the result while the workspace has more trees", () => {
    const views = tabsOf({ hasResult: true, treeCount: 3, resultTreeCount: 2, pairLabels: [["ha", "na"]] }).map(
      ({ view }) => view,
    );

    expect(views).toContain("arg");
  });

  test("labels the tabs", () => {
    expect(viewTabs({ ...NO_WORKSPACE, treeCount: 2 }).map(({ label }) => label)).toStrictEqual([
      "Overview",
      "Tanglegram",
      "Auspice",
      "ARG",
      "MCCs",
      "Files",
      "Diagnostics",
    ]);
  });
});

describe("viewTabs tooltips", () => {
  test("describe each enabled view and give the reason for each disabled one", () => {
    const tabs = viewTabs({ ...NO_WORKSPACE, hasResult: false, treeCount: 2 });

    expect(tabs.map(({ view, tooltip }) => ({ view, tooltip }))).toStrictEqual([
      { view: "overview", tooltip: "The trees, their leaves, and the result of the run" },
      { view: "tanglegram", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
      { view: "auspice", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
      { view: "arg", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
      { view: "mccs", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
      { view: "files", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
      { view: "diagnostics", tooltip: UNAVAILABLE_VIEW_TOOLTIP },
    ]);
  });

  test("describe the tanglegram once a run has a result", () => {
    const tabs = viewTabs({
      ...NO_WORKSPACE,
      hasResult: true,
      treeCount: 2,
      resultTreeCount: 2,
      pairLabels: [["ha", "na"]],
    });

    expect(tabs.find(({ view }) => view === "tanglegram")?.tooltip).toBe("Both trees side by side, linked by MCC");
  });
});
