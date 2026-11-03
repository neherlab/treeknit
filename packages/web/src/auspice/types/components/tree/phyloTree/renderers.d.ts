export declare function setClipMask(this: ClipMaskTree): void;

export interface ClipMaskTree {
  id: string;
  xScale: { range(): number[] };
  yScale: { range(): number[] };
  margins: { branchLabels?: number };
  params: { orientation: readonly number[] };
  svg: ClipMaskSelection;
  groups: { clipPath?: ClipMaskSelection };
}

export interface ClipMaskSelection {
  append(name: string): ClipMaskSelection;
  select(selector: string): ClipMaskSelection;
  attr(name: string, value: number | string): ClipMaskSelection;
}
