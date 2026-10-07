import type { AuspiceNode, AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";

const SHOWN_ROOTS: Record<AuspiceTrees, (datasets: AuspicePair) => AuspiceNode[]> = {
  both: ({ left, right }) => [left.tree, right.tree],
  left: ({ left }) => [left.tree],
  right: ({ right }) => [right.tree],
};

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
