import { Duration, type DurationUnit } from "luxon";

import { APP_LOCALE } from "./locale";

const SPAN_UNITS = [
  { unit: "weeks", intlUnit: "week" },
  { unit: "days", intlUnit: "day" },
  { unit: "hours", intlUnit: "hour" },
  { unit: "minutes", intlUnit: "minute" },
  { unit: "seconds", intlUnit: "second" },
] as const satisfies readonly { unit: DurationUnit; intlUnit: string }[];

const SECOND_MS = 1000;

export function formatElapsed(milliseconds: number): string {
  const span = Number.isFinite(milliseconds) && milliseconds > 0 ? milliseconds : 0;

  return Duration.fromMillis(span).toFormat("m:ss");
}

export function formatSpan(milliseconds: number, locale: string = APP_LOCALE): string {
  const wholeSeconds = Number.isFinite(milliseconds) && milliseconds > 0 ? Math.floor(milliseconds / SECOND_MS) : 0;

  if (wholeSeconds === 0) {
    return new Intl.RelativeTimeFormat(locale, { numeric: "auto" }).format(0, "second");
  }

  const duration = Duration.fromObject({ seconds: wholeSeconds }).shiftTo(...SPAN_UNITS.map(({ unit }) => unit));
  const largest = SPAN_UNITS.findIndex(({ unit }) => duration.get(unit) > 0);

  const parts = SPAN_UNITS.slice(largest, largest + 2).flatMap(({ unit, intlUnit }) => {
    const count = duration.get(unit);

    return count > 0
      ? [new Intl.NumberFormat(locale, { style: "unit", unit: intlUnit, unitDisplay: "narrow" }).format(count)]
      : [];
  });

  return new Intl.ListFormat(locale, { type: "unit", style: "narrow" }).format(parts);
}
