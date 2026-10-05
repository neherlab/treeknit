const KILO = 1000;

const MEGA = KILO * KILO;

const ONE_DECIMAL_ROUNDING = 20;

const SMALLEST_ROUNDED_MEGABYTE = MEGA - KILO / ONE_DECIMAL_ROUNDING;

const LOCALE = "en";

const BYTES = new Intl.NumberFormat(LOCALE, { style: "unit", unit: "byte", unitDisplay: "long" });

const KILOBYTES = new Intl.NumberFormat(LOCALE, {
  style: "unit",
  unit: "kilobyte",
  unitDisplay: "short",
  maximumFractionDigits: 1,
});

const MEGABYTES = new Intl.NumberFormat(LOCALE, {
  style: "unit",
  unit: "megabyte",
  unitDisplay: "short",
  maximumFractionDigits: 1,
});

export function formatBytes(size: number): string {
  const bytes = Number.isFinite(size) && size > 0 ? Math.round(size) : 0;

  if (bytes < KILO) {
    return BYTES.format(bytes);
  }

  if (bytes < SMALLEST_ROUNDED_MEGABYTE) {
    return KILOBYTES.format(bytes / KILO);
  }

  return MEGABYTES.format(bytes / MEGA);
}
