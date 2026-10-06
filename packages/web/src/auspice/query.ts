import { strainSymbolUrlString } from "auspice/src/middleware/changeURL";
import { strainSymbol } from "auspice/src/util/globals";
import { omit } from "remeda";

import type { WorkspaceSearch } from "../workspace/search";
import type { AuspiceControlsState, AuspiceFilterValue } from "./state";

export function auspiceQuery(controls: AuspiceControlsState): string {
  const { defaults, selectedNode } = controls;

  const markedLeaf =
    selectedNode !== null && !selectedNode.isBranch && (selectedNode.existingFilterState ?? null) === null
      ? selectedNode.name
      : undefined;

  const query = new URLSearchParams();

  if (controls.colorBy !== defaults.colorBy) {
    query.set("c", controls.colorBy);
  }

  if (controls.layout !== defaults.layout) {
    query.set("l", controls.layout);
  }

  if (controls.tipLabelKey !== defaults.tipLabelKey) {
    query.set("tl", controls.tipLabelKey === strainSymbol ? strainSymbolUrlString : String(controls.tipLabelKey));
  }

  if (controls.selectedBranchLabel !== defaults.selectedBranchLabel) {
    query.set("branchLabel", controls.selectedBranchLabel);
  }

  if (controls.showAllBranchLabels) {
    query.set("showBranchLabels", "all");
  }

  if (controls.legendOpen === true) {
    query.set("legend", "open");
  }

  if (controls.focus === "selected") {
    query.set("focus", "selected");
  }

  for (const key of Object.keys(controls.filters).toSorted()) {
    setFilter(query, `f_${key}`, controls.filters[key], undefined);
  }

  setFilter(query, "s", controls.filters[strainSymbol], markedLeaf);

  return query.toString();
}

export function parseAuspiceQuery(text: string | undefined): Record<string, string> {
  return Object.fromEntries(new URLSearchParams(text ?? ""));
}

function setFilter(
  query: URLSearchParams,
  key: string,
  values: readonly AuspiceFilterValue[] | undefined,
  omitted: string | undefined,
): void {
  const active = (values ?? []).filter(({ active: on, value }) => on && value !== omitted).map(({ value }) => value);

  if (active.length > 0) {
    query.set(key, active.join(","));
  }
}

export function withAuspiceQuery(search: WorkspaceSearch, text: string): WorkspaceSearch {
  return text === "" ? omit(search, ["auspice"]) : { ...search, auspice: text };
}

export function withoutFilterValue(text: string, key: string, value: string | null): string {
  const query = new URLSearchParams(text);
  const values = query.get(key);

  if (value === null || values === null) {
    return text;
  }

  const kept = values.split(",").filter((item) => item !== value);

  if (kept.length === 0) {
    query.delete(key);
  } else {
    query.set(key, kept.join(","));
  }

  return query.toString();
}
