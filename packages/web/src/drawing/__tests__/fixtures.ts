import type {
  ArgView,
  Bezier,
  Block,
  ConstellationTable,
  DrawNode,
  DrawTree,
  Elbow,
  Leader,
  Mark,
  PairView,
  Point,
} from "@neherlab/treeknit-wasm";

interface NodeSpec {
  name: string;
  parent: number | null;
  x: number;
  y: number;
  mcc?: number;
  mccBreak?: boolean;
  added?: boolean;
  imputed?: boolean;
  branchLength?: number;
}

const MCC_ABCD = 0;

const MCC_X = 1;

const SLOT_ABCD = 0;

const SLOT_X = 3;

export const EXAMPLE_LEFT: NodeSpec[] = [
  { name: "NODE_1", parent: null, x: 0, y: 2 },
  { name: "NODE_2", parent: 0, x: 0.5, y: 0.5, mcc: MCC_ABCD, mccBreak: true, branchLength: 1 },
  { name: "A", parent: 1, x: 1, y: 0, mcc: MCC_ABCD, branchLength: 1 },
  { name: "B", parent: 1, x: 1, y: 1, mcc: MCC_ABCD, branchLength: 1 },
  { name: "NODE_3", parent: 0, x: 0.25, y: 3, branchLength: 0.5 },
  { name: "C", parent: 4, x: 1, y: 2, mcc: MCC_ABCD, mccBreak: true, branchLength: 1.5 },
  { name: "NODE_4", parent: 4, x: 0.5, y: 3.5, branchLength: 0.5 },
  { name: "D", parent: 6, x: 1, y: 3, mcc: MCC_ABCD, mccBreak: true, branchLength: 1 },
  { name: "X", parent: 6, x: 1, y: 4, mcc: MCC_X, mccBreak: true, branchLength: 1 },
];

export const EXAMPLE_RIGHT: NodeSpec[] = [
  { name: "NODE_1", parent: null, x: 0, y: 2 },
  { name: "NODE_2", parent: 0, x: 0.5, y: 0.75, branchLength: 1 },
  { name: "A", parent: 1, x: 1, y: 0, mcc: MCC_ABCD, mccBreak: true, branchLength: 1 },
  { name: "RESOLVED_1", parent: 1, x: 0.75, y: 1.5, added: true, branchLength: 0 },
  { name: "B", parent: 3, x: 1, y: 1, mcc: MCC_ABCD, mccBreak: true, branchLength: 1 },
  { name: "X", parent: 3, x: 1, y: 2, mcc: MCC_X, mccBreak: true, imputed: true, branchLength: 1 },
  { name: "NODE_3", parent: 0, x: 0.5, y: 3.5, mcc: MCC_ABCD, mccBreak: true, branchLength: 1 },
  { name: "C", parent: 6, x: 1, y: 3, mcc: MCC_ABCD, branchLength: 1 },
  { name: "D", parent: 6, x: 1, y: 4, mcc: MCC_ABCD, branchLength: 1 },
];

export function examplePairView(): PairView {
  const left = drawTree("ha", EXAMPLE_LEFT);
  const right = drawTree("na", EXAMPLE_RIGHT);

  const links = ["A", "B", "C", "D", "X"].map((name) => {
    const leftIndex = left.nodes.findIndex((node) => node.name === name);
    const rightIndex = right.nodes.findIndex((node) => node.name === name);

    return { left: leftIndex, right: rightIndex, mcc: name === "X" ? MCC_X : MCC_ABCD };
  });

  const blocks: Block[] = [
    { mcc: MCC_ABCD, left: [0, 1], right: [0, 1] },
    { mcc: MCC_ABCD, left: [2, 3], right: [3, 4] },
    { mcc: MCC_X, left: [4, 4], right: [2, 2] },
  ];

  const slotOf = (mcc: number) => (mcc === MCC_X ? SLOT_X : SLOT_ABCD);

  return {
    left,
    right,
    links,
    blocks,
    mccs: [
      {
        index: MCC_ABCD,
        size: 4,
        leaves: ["A", "B", "C", "D"],
        imputedLeaves: [],
        ambiguousLeaves: [],
        slot: SLOT_ABCD,
      },
      {
        index: MCC_X,
        size: 1,
        leaves: ["X"],
        imputedLeaves: ["X"],
        ambiguousLeaves: ["X"],
        slot: SLOT_X,
      },
    ],
    // oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- field name of the generated PairView type
    shapes: {
      left: treeDrawing(left, slotOf),
      right: treeDrawing(right, slotOf),
      links: links.map((link, index) => ({
        link: index,
        mcc: link.mcc,
        slot: slotOf(link.mcc),
        curve: sCurve([0, left.nodes[link.left]?.y ?? 0], [1, right.nodes[link.right]?.y ?? 0]),
      })),
      ribbons: blocks.map((block, index) => ({
        block: index,
        mcc: block.mcc,
        slot: slotOf(block.mcc),
        outline: ribbonOutline(block.left, block.right),
      })),
    },
  };
}

export function sCurve(from: Point, to: Point): Bezier {
  return { from, c1: [0.5, from[1]], c2: [0.5, to[1]], to };
}

export function exampleArgView(): ArgView {
  const nodes = [
    argNode("ROOT", [null, null], [1, 4], [0, 1], { x: 0, y: 1.5 }),
    argNode("N1", [0, 0], [2, 3], [0, 1], { x: 0.5, y: 0.5 }),
    argNode("A", [1, 1], [], [0, 1], { x: 1, y: 0, leaf: true }),
    argNode("B", [1, 1], [], [0, 1], { x: 1, y: 1, leaf: true }),
    argNode("H", [0, 1], [5], [0, 1], { x: 0.75, y: 2.5, hybrid: true }),
    argNode("C", [4, 4], [], [0, 1], { x: 1, y: 2, leaf: true }),
  ];

  const edges = [
    { parent: 0, child: 1, segments: [0, 1], reticulation: false },
    { parent: 1, child: 2, segments: [0, 1], reticulation: false },
    { parent: 1, child: 3, segments: [0, 1], reticulation: false },
    { parent: 0, child: 4, segments: [0], reticulation: false },
    { parent: 1, child: 4, segments: [1], reticulation: true },
    { parent: 4, child: 5, segments: [0, 1], reticulation: false },
  ];

  const point = (index: number): Point => {
    const node = nodes[index];

    return [node?.xDiv ?? 0, node?.y ?? 0];
  };

  return {
    nodes,
    edges,
    root: 0,
    rootCase: "shared",
    // oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- field name of the generated ArgView type
    shapes: {
      edges: edges.map(({ parent, child, segments, reticulation }, edge) => ({
        edge,
        segments,
        reticulation,
        path: reticulation
          ? { kind: "curve", curve: sCurve(point(parent), point(child)) }
          : { kind: "elbow", points: [point(parent), [point(parent)[0], point(child)[1]], point(child)] },
      })),
      marks: [{ kind: "hybrid", node: 4, mcc: null, slot: null, at: point(4) }],
      leaders: nodes.flatMap((node, index): Leader[] =>
        node.leaf ? [{ node: index, from: point(index), to: [1, node.y] }] : [],
      ),
    },
  };
}

export function exampleConstellation(): ConstellationTable {
  return {
    leaves: ["A", "B", "C", "Z"],
    pairs: [
      ["ha", "na"],
      ["ha", "mp"],
      ["na", "mp"],
    ],
    cells: [
      [{ mcc: 0, size: 3, slot: 0 }, { mcc: 1, size: 2, slot: 2 }, null],
      [{ mcc: 0, size: 3, slot: 0 }, { mcc: 1, size: 2, slot: 2 }, null],
      [{ mcc: 0, size: 3, slot: 0 }, null, null],
      [null, null, { mcc: 0, size: 1, slot: 1 }],
    ],
  };
}

function drawTree(label: string, specs: readonly NodeSpec[]): DrawTree {
  return {
    label,
    nodes: specs.map((spec, index): DrawNode => ({
      name: spec.name,
      shortName: spec.name,
      parent: spec.parent,
      children: specs.flatMap((child, childIndex) => (child.parent === index ? [childIndex] : [])),
      branchLength: spec.branchLength ?? null,
      xDiv: spec.x,
      xDepth: spec.x,
      y: spec.y,
      leaf: specs.every((child) => child.parent !== index),
      cladeSize: leavesBelow(specs, index),
      added: spec.added ?? false,
      imputed: spec.imputed ?? false,
      mcc: spec.mcc ?? null,
      mccBreak: spec.mccBreak ?? false,
    })),
  };
}

function leavesBelow(specs: readonly NodeSpec[], index: number): number {
  const children = specs.flatMap((child, childIndex) => (child.parent === index ? [childIndex] : []));

  return children.length === 0 ? 1 : children.reduce((sum, child) => sum + leavesBelow(specs, child), 0);
}

function treeDrawing(tree: DrawTree, slotOf: (mcc: number) => number) {
  const elbows = tree.nodes.flatMap((node, index): Elbow[] => {
    const parent = node.parent === null ? undefined : tree.nodes[node.parent];

    return parent === undefined
      ? []
      : [
          {
            node: index,
            points: [
              [parent.xDiv, parent.y],
              [parent.xDiv, node.y],
              [node.xDiv, node.y],
            ],
            mcc: node.mcc,
            slot: node.mcc === null ? null : slotOf(node.mcc),
            mccBreak: node.mccBreak,
            added: node.added,
          },
        ];
  });

  const marks = tree.nodes.flatMap((node, index): Mark[] => {
    const parent = node.parent === null ? undefined : tree.nodes[node.parent];
    const color = { mcc: node.mcc, slot: node.mcc === null ? null : slotOf(node.mcc) };

    const reassortment: Mark[] =
      node.mccBreak && parent !== undefined
        ? [{ kind: "reassortment", node: index, ...color, at: [(parent.xDiv + node.xDiv) / 2, node.y] }]
        : [];

    const imputed: Mark[] = node.imputed ? [{ kind: "imputed", node: index, ...color, at: [node.xDiv, node.y] }] : [];

    return [...reassortment, ...imputed];
  });

  const leaders = tree.nodes.flatMap((node, index): Leader[] =>
    node.leaf ? [{ node: index, from: [node.xDiv, node.y], to: [1, node.y] }] : [],
  );

  return { elbows, marks, leaders };
}

function ribbonOutline([leftFirst, leftLast]: [number, number], [rightFirst, rightLast]: [number, number]): Bezier[] {
  const half = 0.5;
  const leftTop = Math.min(leftFirst, leftLast) - half;
  const leftBottom = Math.max(leftFirst, leftLast) + half;
  const rightTop = Math.min(rightFirst, rightLast) - half;
  const rightBottom = Math.max(rightFirst, rightLast) + half;

  return [
    sCurve([0, leftTop], [1, rightTop]),
    line([1, rightTop], [1, rightBottom]),
    sCurve([1, rightBottom], [0, leftBottom]),
    line([0, leftBottom], [0, leftTop]),
  ];
}

function line(from: Point, to: Point): Bezier {
  return { from, c1: from, c2: to, to };
}

function argNode(
  label: string,
  parents: [number | null, number | null],
  children: number[],
  segments: number[],
  { x, y, leaf = false, hybrid = false }: { x: number; y: number; leaf?: boolean; hybrid?: boolean },
): ArgView["nodes"][number] {
  return {
    label,
    shortLabel: label,
    parents,
    children,
    tau: [0.1, 0.2],
    hybrid,
    leaf,
    segments,
    xDiv: x,
    xDepth: x,
    y,
  };
}
