const LOCALE = "en";

const BRANCH_LENGTH = new Intl.NumberFormat(LOCALE, { maximumSignificantDigits: 4 });

const COUNT = new Intl.NumberFormat(LOCALE);

export function formatCount(count: number): string {
  return COUNT.format(count);
}

export function counted(count: number, one: string, many: string): string {
  return `${formatCount(count)} ${count === 1 ? one : many}`;
}

export function mccNumber(index: number): number {
  return index + 1;
}

export function mccTitle(index: number): string {
  return `MCC ${String(mccNumber(index))}`;
}

export function leafCount(size: number): string {
  return counted(size, "leaf", "leaves");
}

export function mccSummary(index: number, size: number): string {
  return `${mccTitle(index)}, ${leafCount(size)}`;
}

export function formatBranchLength(length: number | null): string {
  return length === null ? "none" : BRANCH_LENGTH.format(length);
}
