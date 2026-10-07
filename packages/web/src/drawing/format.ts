import { counted } from "../format/count";
import { APP_LOCALE } from "../format/locale";
import { NONE } from "../format/words";

const BRANCH_LENGTH = new Intl.NumberFormat(APP_LOCALE, { maximumSignificantDigits: 4 });

export function leafCount(size: number): string {
  return counted(size, "leaf", "leaves");
}

export function mccSummary(name: string, size: number): string {
  return `${name}, ${leafCount(size)}`;
}

export function formatBranchLength(length: number | null): string {
  return length === null ? NONE : BRANCH_LENGTH.format(length);
}
