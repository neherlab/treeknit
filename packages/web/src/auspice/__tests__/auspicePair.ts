import type { AuspiceDataset, AuspiceNode, AuspicePair } from "@neherlab/treeknit-wasm";

const MCC_SCALE: [string, string][] = [
  ["MCC 1", "#2f4b9a"],
  ["MCC 2", "#93771c"],
];

export const AUSPICE_LABELS = ["a", "b"] as const;

export const AUSPICE_PAIR: AuspicePair = {
  scale: "div",
  axis_title: "Divergence",
  mcc_roots: [
    { left: "X", right: "X" },
    { left: "NODE_1", right: "NODE_1" },
  ],
  left: dataset({
    name: "NODE_1",
    node_attrs: { div: 0, mcc: { value: "MCC 2" } },
    branch_attrs: { labels: { MCC: "2" } },
    children: [
      {
        name: "NODE_2",
        node_attrs: { div: 1, mcc: { value: "MCC 2" } },
        branch_attrs: {},
        children: [leaf("A", 2, "2"), leaf("B", 2, "2")],
      },
      {
        name: "NODE_3",
        node_attrs: { div: 1, mcc: { value: "MCC 2" } },
        branch_attrs: {},
        children: [
          leaf("C", 2, "2"),
          {
            name: "NODE_4",
            node_attrs: { div: 2 },
            branch_attrs: {},
            children: [leaf("D", 3, "2"), { ...leaf("X", 3, "1"), branch_attrs: { labels: { MCC: "1" } } }],
          },
        ],
      },
    ],
  }),
  right: dataset({
    name: "NODE_1",
    node_attrs: { div: 0, mcc: { value: "MCC 2" } },
    branch_attrs: { labels: { MCC: "2" } },
    children: [
      {
        name: "NODE_2",
        node_attrs: { div: 1, mcc: { value: "MCC 2" } },
        branch_attrs: {},
        children: [
          leaf("A", 2, "2"),
          {
            name: "NODE_3",
            node_attrs: { div: 2 },
            branch_attrs: {},
            children: [leaf("B", 3, "2"), { ...leaf("X", 3, "1"), branch_attrs: { labels: { MCC: "1" } } }],
          },
        ],
      },
      {
        name: "NODE_4",
        node_attrs: { div: 1, mcc: { value: "MCC 2" } },
        branch_attrs: {},
        children: [leaf("C", 2, "2"), leaf("D", 2, "2")],
      },
    ],
  }),
};

function dataset(tree: AuspiceNode): AuspiceDataset {
  return {
    version: "v2",
    meta: {
      title: "TreeKnit: a and b",
      panels: ["tree"],
      colorings: [
        { key: "mcc", title: "MCC (a and b)", type: "categorical", scale: MCC_SCALE },
        { key: "largest_mcc", title: "Largest MCCs", type: "categorical", scale: MCC_SCALE },
      ],
      filters: ["mcc", "largest_mcc"],
      display_defaults: { color_by: "mcc", branch_label: "MCC" },
      sharing: { entropy: false },
    },
    tree,
  };
}

function leaf(name: string, div: number, mcc: string): AuspiceNode {
  return { name, node_attrs: { div, mcc: { value: `MCC ${mcc}` } }, branch_attrs: {} };
}
