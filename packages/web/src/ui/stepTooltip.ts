export function stepTooltip(label: string | undefined): string | null {
  return label === undefined || label === "" ? null : label;
}
