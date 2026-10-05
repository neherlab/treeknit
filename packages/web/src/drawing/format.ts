import { counted } from "../format/count";
import { APP_LOCALE } from "../format/locale";
import { NONE } from "../format/words";

const BRANCH_LENGTH = new Intl.NumberFormat(APP_LOCALE, { maximumSignificantDigits: 4 });

function mccNumber(index: number): number {
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
  return length === null ? NONE : BRANCH_LENGTH.format(length);
}
