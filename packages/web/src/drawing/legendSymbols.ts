export const SYMBOL_HEIGHT_PX = 12;

export const SYMBOL_MIDDLE_PX = SYMBOL_HEIGHT_PX / 2;

export function horizontalPath(lengthPx: number): string {
  return `M0 ${String(SYMBOL_MIDDLE_PX)} H${String(lengthPx)}`;
}

export function sCurvePath(fromY: number, toY: number, endX: number, controlX = endX / 2): string {
  const [x, c] = [String(endX), String(controlX)];

  return `M0 ${String(fromY)} C${c} ${String(fromY)} ${c} ${String(toY)} ${x} ${String(toY)}`;
}

export function bandPath(fromY: number, toY: number, thicknessPx: number, widthPx: number): string {
  const c = String(widthPx / 2);
  const bottom = String(toY + thicknessPx);
  const start = String(fromY + thicknessPx);

  return `${sCurvePath(fromY, toY, widthPx)} V${bottom} C${c} ${bottom} ${c} ${start} 0 ${start} Z`;
}
