import { useMemo } from "react";

import { LABEL_FONT, labelCharacters, shortenLabel, useLabelFontReady } from "../canvas/labels";
import { LABEL_GAP_PX } from "./spacing";

export const LABEL_COLUMN_MAX_SHARE = 0.25;

export interface LeafLabels {
  fontReady: boolean;
  longestPx: number;
}

export function longestLabelPx(names: readonly string[], measure: (text: string) => number, maxChars: number): number {
  return names.reduce((longest, name) => Math.max(longest, measure(shortenLabel(name, maxChars))), 0);
}

export function labelColumnPx(capBasisPx: number, longestPx: number): number {
  return longestPx <= 0 ? 0 : Math.min(longestPx + 2 * LABEL_GAP_PX, LABEL_COLUMN_MAX_SHARE * capBasisPx);
}

export function useLeafLabels(names: readonly string[], reserved: boolean, maxChars: number): LeafLabels {
  const text = useMemo(() => labelCharacters(names), [names]);
  const fontReady = useLabelFontReady(text);

  const longestPx = useMemo(
    () => (reserved && fontReady ? longestLabelPx(names, canvasTextMeasure(), maxChars) : 0),
    [names, reserved, fontReady, maxChars],
  );

  return { fontReady, longestPx };
}

function canvasTextMeasure(): (text: string) => number {
  const context = document.createElement("canvas").getContext("2d");

  if (context === null) {
    return () => 0;
  }

  context.font = LABEL_FONT;

  return (text) => context.measureText(text).width;
}
