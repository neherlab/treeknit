import type { Rgba } from "../canvas/color";
import type { LegendEntry } from "../drawing/Legend";
import {
  bandPath,
  horizontalPath,
  SYMBOL_HEIGHT_PX,
  SYMBOL_MIDDLE_PX,
  SYMBOL_WIDTH_PX,
  sCurvePath,
} from "../drawing/legendSymbols";
import { branchColor, type ColorStyle, linkColor, markColor, type PairStyle, ribbonColor } from "./layers";

const EXAMPLE_MCC = { mcc: 0, slot: 0 };

const STRAIGHT = horizontalPath();

const TO_TIP = horizontalPath(16);

const LINK_CURVE = sCurvePath(2, SYMBOL_HEIGHT_PX - 2);

const RIBBON_OUTLINE = bandPath(1, 5, SYMBOL_MIDDLE_PX);

export function tanglegramLegend({ colors, rules, colorByMcc, ribbons }: LegendStyle): LegendEntry[] {
  const plain: ColorStyle = { colors, rules, colorByMcc, emphasis: { mcc: undefined } };

  return [
    {
      label: "Reassortment branch",
      marks: [
        {
          kind: "line",
          path: STRAIGHT,
          color: branchColor(EXAMPLE_MCC, "reassortment", plain),
          widthPx: rules.reassortmentWidthPx,
        },
        ring([SYMBOL_WIDTH_PX / 2, SYMBOL_MIDDLE_PX], markColor(EXAMPLE_MCC, "reassortment", plain), plain),
      ],
    },
    {
      label: "Node added by resolution",
      marks: [
        {
          kind: "line",
          path: STRAIGHT,
          color: branchColor(EXAMPLE_MCC, "added", plain),
          widthPx: rules.branchWidthPx,
          dashPx: rules.dashPx,
        },
      ],
    },
    {
      label: "Imputed leaf",
      marks: [
        { kind: "line", path: TO_TIP, color: branchColor(EXAMPLE_MCC, "plain", plain), widthPx: rules.branchWidthPx },
        ring([18, SYMBOL_MIDDLE_PX], markColor(EXAMPLE_MCC, "imputed", plain), plain),
      ],
    },
    ribbons
      ? {
          label: "Leaves of one MCC",
          marks: [{ kind: "area", path: RIBBON_OUTLINE, color: ribbonColor(EXAMPLE_MCC, plain) }],
        }
      : {
          label: "Leaf of one MCC in both trees",
          marks: [{ kind: "line", path: LINK_CURVE, color: linkColor(EXAMPLE_MCC, plain), widthPx: rules.linkWidthPx }],
        },
  ];
}

export type LegendStyle = Pick<PairStyle, "colors" | "rules" | "colorByMcc" | "ribbons">;

function ring(at: readonly [number, number], color: Rgba, style: ColorStyle) {
  return {
    kind: "ring",
    at,
    color,
    fill: style.colors.ground,
    radiusPx: style.rules.markRadiusPx,
    lineWidthPx: style.rules.markLinePx,
  } as const;
}
