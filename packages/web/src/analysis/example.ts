import type { TreeText } from "@neherlab/treeknit-wasm";

export function exampleTrees(): TreeText[] {
  return [
    { label: "ha", newick: "((A,B),(C,(D,X)));" },
    { label: "na", newick: "((A,(B,X)),(C,D));" },
  ];
}
