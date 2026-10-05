import { Layer, type LayersList } from "@deck.gl/core";
import { PathLayer } from "@deck.gl/layers";
import type { DrawingRules } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { withOpacity } from "../../canvas/color";
import type { DrawingColors } from "../../canvas/drawingColors";
import { LEADER_OPACITY } from "../../canvas/layers/leaderLayer";
import { DOT_PX, type StyledPathLayer } from "../../canvas/layers/pathLayer";
import { examplePairView } from "../../drawing/__tests__/fixtures";
import { pairEmphasis, UNSELECTED_OPACITY } from "../../drawing/selection";
import { tanglegramColumns } from "../columns";
import { PAIR_LAYER } from "../geometry";
import {
  branchColor,
  labelColor,
  leaderColor,
  linkColor,
  type PairStyle,
  RIBBON_OPACITY,
  ribbonColor,
  ribbonsShown,
  selectionPositions,
  tanglegramLayers,
} from "../layers";
import { tanglegramGeometry } from "./geometry";

const COLORS: DrawingColors = {
  ground: [243, 245, 244, 255],
  ink: [31, 43, 48, 255],
  inkMuted: [85, 101, 107, 255],
  signal: [176, 38, 94, 255],
  focus: [36, 87, 197, 255],
  segmentA: [62, 106, 138, 255],
  segmentB: [138, 106, 62, 255],
  mcc: [
    [47, 75, 154, 255],
    [147, 119, 28, 255],
    [44, 140, 131, 255],
    [124, 79, 172, 255],
    [122, 85, 55, 255],
    [103, 118, 48, 255],
    [206, 121, 30, 255],
    [91, 124, 153, 255],
  ],
  mccNone: [131, 144, 141, 255],
};

const VIEW = examplePairView();

const GEOMETRY = tanglegramGeometry(VIEW, tanglegramColumns(1232, 100), "y", 64);

const STYLE: PairStyle = {
  colors: COLORS,
  colorByMcc: true,
  emphasis: pairEmphasis(VIEW, {}),
  ribbons: false,
  labels: true,
  fontReady: true,
  fade: 1,
};

const SLOT_0 = COLORS.mcc[0] ?? COLORS.mccNone;

const SLOT_3 = COLORS.mcc[3] ?? COLORS.mccNone;

const RULES: DrawingRules = {
  labelAutoMinRowPx: 10,
  linkMinRowPx: 6,
  labelMaxChars: 40,
  marginPx: 16,
  labelGapPx: 6,
  linkZoneShare: 0.2,
  linkZoneMinShare: 0.15,
  tanglegramLabelColumnMaxShare: 0.25,
  argLabelColumnMaxShare: 0.25,
  branchWidthPx: 1.5,
  reassortmentWidthPx: 2,
  linkWidthPx: 1,
  leaderWidthPx: 1,
  leaderOpacity: 0.5,
  markRadiusPx: 3.5,
  markLinePx: 1.5,
  ribbonOpacity: 0.55,
  dashPx: [4, 3],
  dotPx: [1, 3],
};

describe("ribbonsShown", () => {
  test.each([
    [0.1, true],
    [5.99, true],
    [6, false],
    [30, false],
  ] as const)("at %f px per row draws ribbons: %s", (rowPx, ribbons) => {
    expect(ribbonsShown(rowPx, RULES)).toBe(ribbons);
  });
});

describe("branch colors", () => {
  test.each([
    ["a plain branch takes its MCC slot", { mcc: 0, slot: 0 }, "plain", STYLE, SLOT_0],
    ["a branch without an MCC takes the no-MCC color", { mcc: null, slot: null }, "plain", STYLE, COLORS.mccNone],
    [
      "a branch with coloring off is ink-muted",
      { mcc: 0, slot: 0 },
      "plain",
      { ...STYLE, colorByMcc: false },
      COLORS.inkMuted,
    ],
    ["an added branch is ink-muted", { mcc: 0, slot: 0 }, "added", STYLE, COLORS.inkMuted],
    ["a reassortment branch is signal", { mcc: 1, slot: 3 }, "reassortment", STYLE, COLORS.signal],
  ] as const)("%s", (_, item, kind, style, color) => {
    expect(branchColor(item, kind, style)).toStrictEqual(color);
  });
});

describe("selection opacity", () => {
  const selected: PairStyle = { ...STYLE, emphasis: pairEmphasis(VIEW, { mcc: 1 }) };

  test("keeps the selected MCC and fades the others to a quarter", () => {
    expect({
      selectedLink: linkColor({ mcc: 1, slot: 3 }, selected),
      otherLink: linkColor({ mcc: 0, slot: 0 }, selected),
      otherBranch: branchColor({ mcc: 0, slot: 0 }, "plain", selected),
      otherLabel: labelColor(0, selected),
      unassignedLabel: labelColor(null, selected),
    }).toStrictEqual({
      selectedLink: SLOT_3,
      otherLink: withOpacity(SLOT_0, UNSELECTED_OPACITY),
      otherBranch: withOpacity(SLOT_0, UNSELECTED_OPACITY),
      otherLabel: withOpacity(COLORS.ink, UNSELECTED_OPACITY),
      unassignedLabel: withOpacity(COLORS.ink, UNSELECTED_OPACITY),
    });
  });

  test("fills ribbons at 55 percent and fades unselected ones further", () => {
    expect([ribbonColor({ mcc: 1, slot: 3 }, selected), ribbonColor({ mcc: 0, slot: 0 }, selected)]).toStrictEqual([
      withOpacity(SLOT_3, RIBBON_OPACITY),
      withOpacity(SLOT_0, RIBBON_OPACITY * UNSELECTED_OPACITY),
    ]);
  });
});

describe("tanglegramLayers", () => {
  function layerProps(style: PairStyle) {
    return Object.fromEntries(
      deckLayers(tanglegramLayers(GEOMETRY, style)).map((layer) => [
        layer.id,
        { visible: layer.props.visible, opacity: layer.props.opacity },
      ]),
    );
  }

  test("draws links from 6 px per row and ribbons below", () => {
    expect({
      links: layerProps(STYLE)[PAIR_LAYER.links],
      ribbons: layerProps(STYLE)[PAIR_LAYER.ribbons],
      ribbonsBelow: layerProps({ ...STYLE, ribbons: true })[PAIR_LAYER.ribbons],
      linksBelow: layerProps({ ...STYLE, ribbons: true })[PAIR_LAYER.links],
    }).toStrictEqual({
      links: { visible: true, opacity: 1 },
      ribbons: { visible: false, opacity: 1 },
      ribbonsBelow: { visible: true, opacity: 1 },
      linksBelow: { visible: false, opacity: 1 },
    });
  });

  test("fades in links and ribbons, not branches", () => {
    const props = layerProps({ ...STYLE, fade: 0.4 });

    expect([
      props[PAIR_LAYER.links]?.opacity,
      props[PAIR_LAYER.ribbons]?.opacity,
      props[PAIR_LAYER.plainBranches]?.opacity,
    ]).toStrictEqual([0.4, 0.4, 1]);
  });

  test("hides labels and their leaders when the label rule says so or the label font is not ready", () => {
    const hidden = layerProps({ ...STYLE, labels: false });
    const waiting = layerProps({ ...STYLE, fontReady: false });

    expect([
      hidden[PAIR_LAYER.leftLabels]?.visible,
      hidden[PAIR_LAYER.leaders]?.visible,
      waiting[PAIR_LAYER.leaders]?.visible,
      layerProps(STYLE)[PAIR_LAYER.leaders]?.visible,
    ]).toStrictEqual([false, false, false, true]);
  });

  test("draws leaders dotted and lets clicks pass through them", () => {
    const leaders = deckLayers(tanglegramLayers(GEOMETRY, STYLE)).find(
      (layer): layer is StyledPathLayer<unknown> => layer instanceof PathLayer && layer.id === PAIR_LAYER.leaders,
    );

    expect([leaders?.props.pickable, leaders?.props.getDashArray]).toStrictEqual([false, DOT_PX]);
  });

  test("draws leaders in muted ink and dims them with the labels of other MCCs", () => {
    const selected = { ...STYLE, emphasis: pairEmphasis(VIEW, { mcc: 0 }) };

    expect([leaderColor(0, STYLE), leaderColor(0, selected), leaderColor(1, selected)]).toStrictEqual([
      withOpacity(COLORS.inkMuted, LEADER_OPACITY),
      withOpacity(COLORS.inkMuted, LEADER_OPACITY),
      withOpacity(COLORS.inkMuted, LEADER_OPACITY * UNSELECTED_OPACITY),
    ]);
  });

  test("draws the selected leaf's link above the branches and labels", () => {
    const ids = deckLayers(tanglegramLayers(GEOMETRY, { ...STYLE, emphasis: pairEmphasis(VIEW, { leaf: "X" }) })).map(
      (layer) => layer.id,
    );

    expect(ids.indexOf("selected-link")).toBeGreaterThan(ids.indexOf(PAIR_LAYER.rightLabels));
  });

  test("lets clicks on the selected link reach the link below it", () => {
    const selected = deckLayers(
      tanglegramLayers(GEOMETRY, { ...STYLE, emphasis: pairEmphasis(VIEW, { leaf: "X" }) }),
    ).find((layer) => layer.id === "selected-link");

    expect(selected?.props.pickable).toBe(false);
  });

  test("rings a selected leaf in both trees", () => {
    expect(selectionPositions(GEOMETRY, pairEmphasis(VIEW, { leaf: "X" }))).toStrictEqual([
      GEOMETRY.nodes.left[8],
      GEOMETRY.nodes.right[5],
    ]);
  });

  test("rings a selected node at its position", () => {
    expect(
      selectionPositions(GEOMETRY, pairEmphasis(VIEW, { node: { side: "right", name: "RESOLVED_1" } })),
    ).toStrictEqual([GEOMETRY.nodes.right[3]]);
  });
});

function deckLayers(list: LayersList): Layer[] {
  return list.flat().filter((layer) => layer instanceof Layer);
}
