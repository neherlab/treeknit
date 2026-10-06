import type { AuspiceNode, AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";

const SHOWN_ROOTS: Record<AuspiceTrees, (datasets: AuspicePair) => AuspiceNode[]> = {
  both: ({ left, right }) => [left.tree, right.tree],
  left: ({ left }) => [left.tree],
  right: ({ right }) => [right.tree],
};

export function shownTreeLabels(labels: readonly [string, string] | undefined, trees: AuspiceTrees): string[] {
  if (labels === undefined) {
    return [];
  }

  const [left, right] = labels;

  if (trees === "left") {
    return [left];
  }

  return trees === "right" ? [right] : [left, right];
}

export function leafNames(datasets: AuspicePair, trees: AuspiceTrees): string[] {
  const names = new Set<string>();

  for (const root of SHOWN_ROOTS[trees](datasets)) {
    const stack: AuspiceNode[] = [root];

    for (let node = stack.pop(); node !== undefined; node = stack.pop()) {
      if (node.children === undefined) {
        names.add(node.name);
      } else {
        stack.push(...node.children);
      }
    }
  }

  return [...names].toSorted();
}
