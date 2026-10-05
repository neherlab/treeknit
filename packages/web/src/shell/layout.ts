export const RAIL_PANE_MIN_WIDTH_PX = 1024;

export const INSPECTOR_PANE_MIN_WIDTH_PX = 1320;

export type PanePlacement = "pane" | "sheet";

export interface ShellLayout {
  rail: PanePlacement;
  inspector: PanePlacement;
}

const WIDE: ShellLayout = { rail: "pane", inspector: "pane" };

const MEDIUM: ShellLayout = { rail: "pane", inspector: "sheet" };

const NARROW: ShellLayout = { rail: "sheet", inspector: "sheet" };

export function shellLayout(viewportWidthPx: number): ShellLayout {
  if (viewportWidthPx >= INSPECTOR_PANE_MIN_WIDTH_PX) {
    return WIDE;
  }

  if (viewportWidthPx >= RAIL_PANE_MIN_WIDTH_PX) {
    return MEDIUM;
  }

  return NARROW;
}

export function keepSheetOpen(open: boolean, placement: PanePlacement): boolean {
  return open && placement === "sheet";
}

export const SHELL_MEDIA_QUERIES = [RAIL_PANE_MIN_WIDTH_PX, INSPECTOR_PANE_MIN_WIDTH_PX].map(
  (widthPx) => `(min-width: ${widthPx}px)`,
);

export const RAIL_TITLE = "Trees and settings";

export const INSPECTOR_TITLE = "Details";
