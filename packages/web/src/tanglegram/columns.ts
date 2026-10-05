import type { Column } from "../canvas/projection";
import { DRAWING_MARGIN_PX, LABEL_GAP_PX } from "../drawing/spacing";

export const LABEL_COLUMN_MAX_SHARE = 0.25;

export const LINK_ZONE_MIN_SHARE = 0.15;

export const LINK_ZONE_SHARE = 0.2;

export interface TanglegramColumns {
  left: Column;
  leftLabels: Column;
  links: Column;
  rightLabels: Column;
  right: Column;
}

export function labelColumnPx(crossPx: number, longestLabelPx: number): number {
  if (longestLabelPx <= 0) {
    return 0;
  }

  const half = Math.max(crossPx - 2 * DRAWING_MARGIN_PX, 0) / 2;

  return Math.min(longestLabelPx + 2 * LABEL_GAP_PX, LABEL_COLUMN_MAX_SHARE * half);
}

export function tanglegramColumns(crossPx: number, labelPx: number): TanglegramColumns {
  const inner = Math.max(crossPx - 2 * DRAWING_MARGIN_PX, 0);
  const links = Math.max(LINK_ZONE_SHARE, LINK_ZONE_MIN_SHARE) * inner;
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
