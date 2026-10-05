import type { AuspiceThunk } from "auspice/src/state";

export declare function updateVisibleTipsAndBranchThicknesses(params?: {
  root?: [number | undefined, number | undefined];
}): AuspiceThunk;

export declare function applyFilter(
  mode: "add" | "inactivate" | "remove" | "set" | "focus",
  trait: string | symbol,
  values: string[],
): AuspiceThunk;
