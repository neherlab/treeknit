export interface PanelSize {
  width: number;
  height: number;
}

export declare function calcPanelDims(
  panels: readonly string[],
  narrativeIsDisplayed: boolean,
  availableWidth: number,
  availableHeight: number,
): { full: PanelSize; grid: PanelSize; chartFrequencies: PanelSize; chartEntropy: PanelSize };
