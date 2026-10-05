import type { Rgba } from "../canvas/color";
import { DASH_PX } from "../canvas/layers/pathLayer";
import { BRANCH_WIDTH_PX, MARK_LINE_PX, MARK_RADIUS_PX, REASSORTMENT_WIDTH_PX } from "../canvas/layers/treeLayers";
import type { LegendEntry } from "../drawing/Legend";
import {
  branchColor,
  type ColorStyle,
  linkColor,
  LINK_WIDTH_PX,
  markColor,
  type PairStyle,
  ribbonColor,
} from "./layers";

const EXAMPLE_MCC = { mcc: 0, slot: 0 };

const STRAIGHT = "M0 6 H24";

const TO_TIP = "M0 6 H16";

const LINK_CURVE = "M0 2 C12 2 12 10 24 10";

const RIBBON_OUTLINE = "M0 1 C12 1 12 5 24 5 V11 C12 11 12 7 0 7 Z";

export function tanglegramLegend({ colors, colorByMcc, ribbons }: LegendStyle): LegendEntry[] {
  const plain: ColorStyle = { colors, colorByMcc, emphasis: { mcc: undefined } };

  return [
    {
      label: "Reassortment branch",
      marks: [
        {
          kind: "line",
          path: STRAIGHT,
          color: branchColor(EXAMPLE_MCC, "reassortment", plain),
          widthPx: REASSORTMENT_WIDTH_PX,
        },
        ring([12, 6], markColor(EXAMPLE_MCC, "reassortment", plain), plain),
      ],
    },
    {
      label: "Node added by resolution",
      marks: [
        {
          kind: "line",
          path: STRAIGHT,
          color: branchColor(EXAMPLE_MCC, "added", plain),
          widthPx: BRANCH_WIDTH_PX,
          dashPx: DASH_PX,
        },
      ],
    },
    {
      label: "Imputed leaf",
      marks: [
        { kind: "line", path: TO_TIP, color: branchColor(EXAMPLE_MCC, "plain", plain), widthPx: BRANCH_WIDTH_PX },
        ring([18, 6], markColor(EXAMPLE_MCC, "imputed", plain), plain),
      ],
    },
    ribbons
      ? {
          label: "Leaves of one MCC",
          marks: [{ kind: "area", path: RIBBON_OUTLINE, color: ribbonColor(EXAMPLE_MCC, plain) }],
        }
      : {
          label: "Leaf of one MCC in both trees",
          marks: [{ kind: "line", path: LINK_CURVE, color: linkColor(EXAMPLE_MCC, plain), widthPx: LINK_WIDTH_PX }],
        },
  ];
}

export type LegendStyle = Pick<PairStyle, "colors" | "colorByMcc" | "ribbons">;

function ring(at: readonly [number, number], color: Rgba, style: ColorStyle) {
  return {
    kind: "ring",
    at,
    color,
    fill: style.colors.ground,
    radiusPx: MARK_RADIUS_PX,
    lineWidthPx: MARK_LINE_PX,
  } as const;
}
