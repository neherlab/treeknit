import { cn } from "cn";

const SLOT_CLASSES = [
  "bg-mcc-0",
  "bg-mcc-1",
  "bg-mcc-2",
  "bg-mcc-3",
  "bg-mcc-4",
  "bg-mcc-5",
  "bg-mcc-6",
  "bg-mcc-7",
] as const;

export function MccSwatch({ slot, className }: { slot: number | null; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        "rounded-inner inline-block size-3 shrink-0",
        slot === null ? "bg-mcc-none" : (SLOT_CLASSES[slot] ?? "bg-mcc-none"),
        className,
      )}
    />
  );
}
