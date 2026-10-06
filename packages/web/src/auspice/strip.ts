import type { AuspiceTrees, PairSummary } from "@neherlab/treeknit-wasm";

export interface ShownTrees {
  pair: number;
  trees: AuspiceTrees;
}

export function shownTreeIndices(pair: PairSummary, trees: AuspiceTrees): number[] {
  const [left, right] = pair.trees;

  if (trees === "left") {
    return [left];
  }

  return trees === "right" ? [right] : [left, right];
}

export function shownFromStrip(
  selected: readonly number[],
  pairs: readonly PairSummary[],
  current: ShownTrees,
): ShownTrees | null {
  if (selected.length === 2) {
    const [a = 0, b = 0] = selected;
    const wanted = [Math.min(a, b), Math.max(a, b)];
    const pair = pairs.find(({ trees }) => trees[0] === wanted[0] && trees[1] === wanted[1]);

    return pair === undefined ? null : { pair: pair.index, trees: "both" };
  }

  const [only] = selected;
  const pair = pairs[current.pair];

  if (selected.length !== 1 || only === undefined || pair === undefined || !pair.trees.includes(only)) {
    return null;
  }

  return { pair: current.pair, trees: only === pair.trees[0] ? "left" : "right" };
}
