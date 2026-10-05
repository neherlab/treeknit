import { LABEL_FONT, shortenLabel } from "../canvas/labels";

export function longestLabelPx(names: readonly string[], measure: (text: string) => number, maxChars: number): number {
  return names.reduce((longest, name) => Math.max(longest, measure(shortenLabel(name, maxChars))), 0);
}

export function canvasTextMeasure(): (text: string) => number {
  const context = document.createElement("canvas").getContext("2d");

  if (context === null) {
    return () => 0;
  }

  context.font = LABEL_FONT;

  return (text) => context.measureText(text).width;
}
