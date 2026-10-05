import { useMemo } from "react";

import { LABEL_FONT, labelCharacters, useLabelFontReady } from "../canvas/labels";
import { LABEL_GAP_PX } from "./spacing";

export const LABEL_COLUMN_MAX_SHARE = 0.25;

export interface LeafLabels {
  fontReady: boolean;
  longestPx: number;
}

export function longestLabelPx(texts: readonly string[], measure: (text: string) => number): number {
  return texts.reduce((longest, text) => Math.max(longest, measure(text)), 0);
}

export function labelColumnPx(capBasisPx: number, longestPx: number): number {
  return longestPx <= 0 ? 0 : Math.min(longestPx + 2 * LABEL_GAP_PX, LABEL_COLUMN_MAX_SHARE * capBasisPx);
}

export function useLeafLabels(texts: readonly string[], reserved: boolean): LeafLabels {
  const characters = useMemo(() => labelCharacters(texts), [texts]);
  const fontReady = useLabelFontReady(characters);

  const longestPx = useMemo(
    () => (reserved && fontReady ? longestLabelPx(texts, canvasTextMeasure()) : 0),
    [texts, reserved, fontReady],
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
