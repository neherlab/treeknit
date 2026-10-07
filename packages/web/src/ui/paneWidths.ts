export const PANE_WIDTHS = {
  260: "w-[260px]",
  320: "w-[320px]",
  336: "w-[336px]",
} as const;

export type PaneWidthPx = keyof typeof PANE_WIDTHS;
