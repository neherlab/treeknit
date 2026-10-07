import type { PaneWidthPx } from "../ui/paneWidths";

export const CARD_TITLE_HEIGHT_PX = 26;

export const CHART_BOTTOM_GAP_PX = 16;

export const MIN_TREE_WIDTH_PX = 320;

export const SIDEBAR_WIDTH_PX: PaneWidthPx = 260;

export const OVERLAY_MAX_WIDTH_PX = 900;

export interface TreeSize {
  width: number;
  height: number;
}

export function treeSize(area: TreeSize): TreeSize | null {
  const width = Math.floor(area.width);
  const height = Math.floor(area.height) - CARD_TITLE_HEIGHT_PX - CHART_BOTTOM_GAP_PX;

  return width >= MIN_TREE_WIDTH_PX && height > 0 ? { width, height } : null;
}

export function sidebarOverlays(viewWidth: number): boolean {
  return viewWidth < OVERLAY_MAX_WIDTH_PX;
}
