import type { Color, Position } from "@deck.gl/core";
import { describe, expect, test } from "vitest";

import { type LabelAnchor, labelLayer } from "../layers/labelLayer";
import { DASH_PX, pathLayer } from "../layers/pathLayer";

interface Leaf {
  name: string;
  row: number;
}

const LEAVES: Leaf[] = [
  { name: "A/Hong Kong/1/1968", row: 0 },
  { name: "Ünïcödé/Ω", row: 1 },
];

const INK: Color = [31, 43, 48, 255];

const START: LabelAnchor = "start";

const LABEL_OFFSET: [number, number] = [4, 0];

const LABELS = {
  id: "labels",
  data: LEAVES,
  visible: true,
  getPosition: (leaf: Leaf): Position => [100, leaf.row],
  getText: (leaf: Leaf) => leaf.name,
  getColor: INK,
  anchor: START,
  offsetPx: LABEL_OFFSET,
};

describe("labelLayer", () => {
  test("waits for the label font before building a text layer", () => {
    expect(labelLayer({ ...LABELS, fontReady: false })).toBeNull();
  });
});

describe("pathLayer", () => {
  const BRANCHES: { path: [number, number][] }[] = [
    {
      path: [
        [0, 0],
        [10, 0],
      ],
    },
  ];

  const PATHS = {
    id: "branches",
    data: BRANCHES,
    getPath: (item: { path: [number, number][] }) => item.path,
    getColor: INK,
    widthPx: 2,
  };

  test("draws solid lines without the dash extension", () => {
    expect(pathLayer(PATHS).props.extensions).toStrictEqual([]);
  });

  test("draws dashed lines with a dash pattern in pixels", () => {
    const layer = pathLayer({ ...PATHS, dashed: true });

    expect(layer.props.extensions).toHaveLength(1);
    expect(layer.props.getDashArray).toStrictEqual(DASH_PX);
    expect(layer.props.dashUnits).toBe("pixels");
  });
});
