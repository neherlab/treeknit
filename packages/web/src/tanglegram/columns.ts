import type { DrawingRules } from "@neherlab/treeknit-wasm";

import type { Column } from "../canvas/projection";
import { innerWidthPx, labelColumnPx } from "../drawing/spacing";

export interface TanglegramColumns {
  left: Column;
  leftLabels: Column;
  links: Column;
  rightLabels: Column;
  right: Column;
}

export function tanglegramColumns(crossPx: number, longestLabelPx: number, rules: DrawingRules): TanglegramColumns {
  const inner = innerWidthPx(crossPx, rules);
  const labelPx = labelColumnPx((rules.tanglegramLabelColumnMaxShare * inner) / 2, longestLabelPx, rules);
  const links = Math.max(rules.linkZoneMinShare * inner, rules.linkZoneShare * inner - 2 * labelPx);
  const tree = Math.max((inner - links - 2 * labelPx) / 2, 0);
  const leftStart = rules.marginPx;
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
