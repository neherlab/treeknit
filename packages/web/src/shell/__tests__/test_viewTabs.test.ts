import { describe, expect, test } from "vitest";

import { NO_WORKSPACE, type WorkspaceAvailability } from "../../workspace/search";
import { viewTabs } from "../viewTabs";

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
    expect(tabsOf({ hasResult: true, treeCount: 2, resultTreeCount: 2, pairCount: 1 })).toStrictEqual([
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
    const views = tabsOf({ hasResult: true, treeCount: 3, resultTreeCount: 3, pairCount: 3 }).map(({ view }) => view);

    expect(views).toStrictEqual(["overview", "tanglegram", "auspice", "mccs", "constellation", "files", "diagnostics"]);
  });

  test("follows the trees of the result while the workspace has more trees", () => {
    const views = tabsOf({ hasResult: true, treeCount: 3, resultTreeCount: 2, pairCount: 1 }).map(({ view }) => view);

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
