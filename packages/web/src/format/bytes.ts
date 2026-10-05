const KILO = 1000;

const MEGA = KILO * KILO;

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
  if (size < KILO) {
    return BYTES.format(size);
  }

  if (size < MEGA) {
    return KILOBYTES.format(size / KILO);
  }

  return MEGABYTES.format(size / MEGA);
}
