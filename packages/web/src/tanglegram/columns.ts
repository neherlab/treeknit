import type { Column } from "../canvas/projection";
import { DRAWING_MARGIN_PX, innerWidthPx } from "../drawing/spacing";

export const LINK_ZONE_MIN_SHARE = 0.15;

export const LINK_ZONE_SHARE = 0.2;

export interface TanglegramColumns {
  left: Column;
  leftLabels: Column;
  links: Column;
  rightLabels: Column;
  right: Column;
}

export function tanglegramColumns(crossPx: number, labelPx: number): TanglegramColumns {
  const inner = innerWidthPx(crossPx);
  const links = Math.max(LINK_ZONE_MIN_SHARE * inner, LINK_ZONE_SHARE * inner - 2 * labelPx);
  const tree = Math.max((inner - links - 2 * labelPx) / 2, 0);
  const leftStart = DRAWING_MARGIN_PX;
  const leftEnd = leftStart + tree;
  const linksStart = leftEnd + labelPx;
  const linksEnd = linksStart + links;
  const rightLabelsEnd = linksEnd + labelPx;
  const rightEnd = rightLabelsEnd + tree;

  return {
    left: { start: leftStart, end: leftEnd, mirrored: false },
    leftLabels: { start: leftEnd, end: linksStart, mirrored: false },
    links: { start: linksStart, end: linksEnd, mirrored: false },
    rightLabels: { start: linksEnd, end: rightLabelsEnd, mirrored: false },
    right: { start: rightLabelsEnd, end: rightEnd, mirrored: true },
  };
}
