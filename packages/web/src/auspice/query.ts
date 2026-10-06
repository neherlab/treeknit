import { strainSymbolUrlString } from "auspice/src/middleware/changeURL";
import { strainSymbol } from "auspice/src/util/globals";
import { omit } from "remeda";

import type { WrittenSearch } from "../workspace/search";
import type { AuspiceControlsState, AuspiceFilterValue } from "./state";

export const AUSPICE_KEYS = {
  colorBy: "c",
  layout: "l",
  tipLabel: "tl",
  branchLabel: "branchLabel",
  showBranchLabels: "showBranchLabels",
  legend: "legend",
  focus: "focus",
  strain: "s",
} as const;

export const AUSPICE_FILTER_PREFIX = "f_";

const AUSPICE_KEY_NAMES: ReadonlySet<string> = new Set(Object.values(AUSPICE_KEYS));

export function isAuspiceKey(key: string): boolean {
  return (
    AUSPICE_KEY_NAMES.has(key) || (key.startsWith(AUSPICE_FILTER_PREFIX) && key.length > AUSPICE_FILTER_PREFIX.length)
  );
}

export function auspiceQuery(controls: AuspiceControlsState): string {
  const { defaults, selectedNode } = controls;

  const markedLeaf =
    selectedNode !== null && !selectedNode.isBranch && (selectedNode.existingFilterState ?? null) === null
      ? selectedNode.name
      : undefined;

  const query = new URLSearchParams();

  if (controls.colorBy !== defaults.colorBy) {
    query.set(AUSPICE_KEYS.colorBy, controls.colorBy);
  }

  if (controls.layout !== defaults.layout) {
    query.set(AUSPICE_KEYS.layout, controls.layout);
  }

  if (controls.tipLabelKey !== defaults.tipLabelKey) {
    query.set(
      AUSPICE_KEYS.tipLabel,
      controls.tipLabelKey === strainSymbol ? strainSymbolUrlString : String(controls.tipLabelKey),
    );
  }

  if (controls.selectedBranchLabel !== defaults.selectedBranchLabel) {
    query.set(AUSPICE_KEYS.branchLabel, controls.selectedBranchLabel);
  }

  if (controls.showAllBranchLabels) {
    query.set(AUSPICE_KEYS.showBranchLabels, "all");
  }

  if (controls.legendOpen === true) {
    query.set(AUSPICE_KEYS.legend, "open");
  }

  if (controls.focus === "selected") {
    query.set(AUSPICE_KEYS.focus, "selected");
  }

  for (const key of Object.keys(controls.filters).toSorted()) {
    setFilter(query, `${AUSPICE_FILTER_PREFIX}${key}`, controls.filters[key], undefined);
  }

  setFilter(query, AUSPICE_KEYS.strain, controls.filters[strainSymbol], markedLeaf);

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

export function withAuspiceQuery(search: WrittenSearch, text: string): WrittenSearch {
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
