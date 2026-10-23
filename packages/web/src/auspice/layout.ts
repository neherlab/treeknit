import { calcPanelDims } from "auspice/src/components/main/utils";
import { controlsHiddenWidth } from "auspice/src/util/globals";

import type { PaneWidthPx } from "../ui/paneWidths";

export const CARD_TITLE_HEIGHT_PX = 26;

export const CARD_TITLE_HEIGHT_TWO_TREES_PX = 46;

export const HEADER_ROW_HEIGHT_PX = 26;

export const HEADER_GAP_PX = 16;

export const HEADER_MIN_SUMMARY_WIDTH_PX = 120;

export const HEADER_COMPACT_MAX_WIDTH_PX = 600;

export const MIN_TREE_WIDTH_PX = 320;

export const SIDEBAR_WIDTH_PX: PaneWidthPx = 266;

export interface TreeSize {
  width: number;
  height: number;
}

export function treeSize(area: TreeSize, headerStacked: boolean): TreeSize | null {
  const panel = calcPanelDims(["tree"], false, area.width, area.height).full;
  const width = Math.floor(panel.width);
  const height = Math.floor(panel.height) - (headerStacked ? HEADER_ROW_HEIGHT_PX : 0);

  return width >= MIN_TREE_WIDTH_PX && height > 0 ? { width, height } : null;
}

export function sidebarOverlays(viewWidth: number): boolean {
  return viewWidth <= controlsHiddenWidth;
}

export function cardTitleHeight(twoTrees: boolean): number {
  return twoTrees ? CARD_TITLE_HEIGHT_TWO_TREES_PX : CARD_TITLE_HEIGHT_PX;
}

export interface HeaderWidths {
  tree: number;
  zoom: number;
  downloads: number;
}

export interface HeaderLayout {
  stacked: boolean;
  compactDownloads: boolean;
}

export function headerLayout({ tree, zoom, downloads }: HeaderWidths): HeaderLayout {
  return {
    stacked: tree - zoom - HEADER_GAP_PX < downloads + HEADER_GAP_PX + HEADER_MIN_SUMMARY_WIDTH_PX,
    compactDownloads: tree < HEADER_COMPACT_MAX_WIDTH_PX,
  };
}
