import { APP_LOCALE } from "./locale";

const COUNT = new Intl.NumberFormat(APP_LOCALE);

export function formatCount(count: number): string {
  if (!Number.isFinite(count)) {
    throw new RangeError(`The count ${String(count)} is not a finite number`);
  }

  return COUNT.format(count);
}

export function counted(count: number, one: string, many: string): string {
  return `${formatCount(count)} ${count === 1 ? one : many}`;
}
