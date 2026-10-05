const branchLength = new Intl.NumberFormat(undefined, { maximumSignificantDigits: 4 });

const count = new Intl.NumberFormat();

export function mccNumber(index: number): number {
  return index + 1;
}

export function mccTitle(index: number): string {
  return `MCC ${String(mccNumber(index))}`;
}

export function leafCount(size: number): string {
  return `${count.format(size)} ${size === 1 ? "leaf" : "leaves"}`;
}

export function mccSummary(index: number, size: number): string {
  return `${mccTitle(index)}, ${leafCount(size)}`;
}

export function formatBranchLength(length: number | null): string {
  return length === null ? "none" : branchLength.format(length);
}
