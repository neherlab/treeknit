import type { Color, Position } from "@deck.gl/core";
import { describe, expect, test } from "vitest";

import { fillLayer } from "../layers/fillLayer";
import { type LabelAnchor, labelLayer } from "../layers/labelLayer";
import { markLayer } from "../layers/markLayer";
import { pathLayer } from "../layers/pathLayer";

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

  test("builds its glyphs from the data, because strain names hold letters outside ASCII", () => {
    expect(labelLayer({ ...LABELS, fontReady: true })?.props.characterSet).toBe("auto");
  });
});

describe("selection and hover change colors without rebuilding geometry", () => {
  const TRIGGERS = ["selected MCC 3", "hovered leaf 7"];

  const SQUARE: [number, number][] = [
    [0, 0],
    [1, 0],
    [1, 1],
  ];

  test.each([
    ["labels", labelLayer({ ...LABELS, fontReady: true, colorTriggers: TRIGGERS })?.props.updateTriggers, ["getColor"]],
    [
      "paths",
      pathLayer({
        id: "p",
        data: [SQUARE],
        getPath: (path) => path,
        getColor: INK,
        widthPx: 1,
        colorTriggers: TRIGGERS,
      }).props.updateTriggers,
      ["getColor"],
    ],
    [
      "fills",
      fillLayer({ id: "f", data: [SQUARE], getPolygon: (path) => path, getFillColor: INK, colorTriggers: TRIGGERS })
        .props.updateTriggers,
      ["getFillColor"],
    ],
    [
      "marks",
      markLayer({
        id: "m",
        data: [[0, 0]],
        getPosition: (point: [number, number]) => point,
        getLineColor: INK,
        radiusPx: 3,
        lineWidthPx: 1,
        colorTriggers: TRIGGERS,
      }).props.updateTriggers,
      ["getFillColor", "getLineColor"],
    ],
  ])("ties only the color accessors of %s to the color triggers", (_kind, triggers, colorAccessors) => {
    expect(triggers).toStrictEqual(Object.fromEntries(colorAccessors.map((accessor) => [accessor, TRIGGERS])));
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
    const layer = pathLayer({ ...PATHS, dashPx: [4, 3] });

    expect(layer.props.extensions).toHaveLength(1);
    expect(layer.props.getDashArray).toStrictEqual([4, 3]);
    expect(layer.props.dashUnits).toBe("pixels");
  });
});
