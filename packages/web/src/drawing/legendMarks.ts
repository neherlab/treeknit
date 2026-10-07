import type { DrawingRules, LegendItem, LegendMark } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";

import { withOpacity } from "../canvas/color";
import { type DrawingColors, mccColor, roleColor, roleColorByMcc } from "../canvas/drawingColors";
import type { LegendEntry, SymbolMark } from "./Legend";
import { bandPath, horizontalPath, SYMBOL_HEIGHT_PX, SYMBOL_MIDDLE_PX, sCurvePath } from "./legendSymbols";

const LEGEND_SLOT = 0;

export function legendEntries(items: readonly LegendItem[], style: LegendStyle): LegendEntry[] {
  return items.map((item) => ({
    label: item.label,
    marks: item.marks.flatMap((mark) => symbolMark(mark, style)),
  }));
}

export interface LegendStyle {
  colors: DrawingColors;
  rules: DrawingRules;
  colorByMcc: boolean;
  ribbons: boolean;
}

function symbolMark(mark: LegendMark, { colors, rules, colorByMcc, ribbons }: LegendStyle): SymbolMark[] {
  const width = rules.legendSymbolPx;

  return match(mark)
    .returnType<SymbolMark[]>()
    .with({ kind: "branch" }, ({ color, stroke }) => [
      {
        kind: "line",
        path: horizontalPath(width),
        color: roleColorByMcc(colors, color, LEGEND_SLOT, colorByMcc),
        widthPx: stroke === "reassortment" ? rules.reassortmentWidthPx : rules.branchWidthPx,
      },
    ])
    .with({ kind: "linkRibbon" }, () =>
      ribbons
        ? [
            {
              kind: "area",
              path: bandPath(1, 5, SYMBOL_MIDDLE_PX, width),
              color: withOpacity(mccColor(colors, LEGEND_SLOT), rules.ribbonOpacity),
            },
          ]
        : [],
    )
    .with({ kind: "linkCurve" }, ({ color }) =>
      ribbons
        ? []
        : [
            {
              kind: "line",
              path: sCurvePath(2, SYMBOL_HEIGHT_PX - 2, width),
              color: roleColor(colors, color, LEGEND_SLOT),
              widthPx: rules.linkWidthPx,
            },
          ],
    )
    .with({ kind: "reticulation" }, ({ color }) => [
      {
        kind: "line",
        path: sCurvePath(1, SYMBOL_MIDDLE_PX, width),
        color: roleColor(colors, color, null),
        widthPx: rules.branchWidthPx,
        dashPx: rules.dashPx,
      },
    ])
    .with({ kind: "ring" }, ({ color, at }) => [
      {
        kind: "ring",
        at: [at * width, SYMBOL_MIDDLE_PX],
        color: roleColorByMcc(colors, color, LEGEND_SLOT, colorByMcc),
        fill: colors.ground,
        radiusPx: rules.markRadiusPx,
        lineWidthPx: rules.markLinePx,
      },
    ])
    .exhaustive();
}
