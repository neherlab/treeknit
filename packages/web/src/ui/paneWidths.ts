export const PANE_WIDTHS = {
  266: "w-[266px]",
  320: "w-[320px]",
  336: "w-[336px]",
} as const;

export type PaneWidthPx = keyof typeof PANE_WIDTHS;
