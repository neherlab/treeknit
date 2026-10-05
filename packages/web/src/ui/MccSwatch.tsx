import { cn } from "cn";

import { MCC_SLOT_COUNT } from "../canvas/drawingColors";

const SLOT_CLASSES = [
  "bg-mcc-0",
  "bg-mcc-1",
  "bg-mcc-2",
  "bg-mcc-3",
  "bg-mcc-4",
  "bg-mcc-5",
  "bg-mcc-6",
  "bg-mcc-7",
] as const satisfies { length: typeof MCC_SLOT_COUNT };

export function MccSwatch({ slot, className }: { slot: number | null; className?: string }) {
  return (
    <span aria-hidden className={cn("rounded-inner inline-block size-3 shrink-0", swatchClass(slot), className)} />
  );
}

export function swatchClass(slot: number | null): string {
  if (slot === null) {
    return "bg-mcc-none";
  }

  const name = SLOT_CLASSES[slot];

  if (name === undefined) {
    throw new RangeError(`MCC color slot ${String(slot)} is outside the ${String(MCC_SLOT_COUNT)} palette slots`);
  }

  return name;
}
