import { cn } from "cn";
import { type ReactNode, useCallback, useState } from "react";
import HideIcon from "~icons/lucide/chevron-up";

import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";

export function Legend({ entries, className }: LegendProps) {
  const [open, setOpen] = useState(true);

  const show = useCallback(() => {
    setOpen(true);
  }, []);

  const hide = useCallback(() => {
    setOpen(false);
  }, []);

  if (!open) {
    return (
      <div className={cn("absolute top-2 left-2", className)}>
        <Button variant="secondary" size="xs" onPress={show}>
          Show legend
        </Button>
      </div>
    );
  }

  return (
    <section
      aria-label="Legend"
      className={cn(
        "rounded-control border-rule bg-ground/90 absolute top-2 left-2 flex items-start gap-2 border py-1.5 pr-1 pl-2.5 text-xs shadow-sm",
        className,
      )}
    >
      <ul className="flex flex-col gap-1">
        {entries.map(({ label, symbol }) => (
          <li key={label} className="text-ink flex items-center gap-2">
            <svg aria-hidden width={24} height={12} viewBox="0 0 24 12" className="shrink-0 overflow-visible">
              {symbol}
            </svg>
            {label}
          </li>
        ))}
      </ul>
      <IconButton label="Hide legend" icon={HideIcon} size="xs" onPress={hide} />
    </section>
  );
}

export interface LegendProps {
  entries: readonly LegendEntry[];
  className?: string;
}

export interface LegendEntry {
  label: string;
  symbol: ReactNode;
}

export const REASSORTMENT_SYMBOL = (
  <>
    <path d="M0 6 H24" className="stroke-signal" strokeWidth={2} />
    <circle cx={12} cy={6} r={3.5} className="fill-ground stroke-signal" strokeWidth={1.5} />
  </>
);

export const ADDED_SYMBOL = <path d="M0 6 H24" className="stroke-ink-muted" strokeWidth={1.5} strokeDasharray="4 3" />;

export const IMPUTED_SYMBOL = (
  <>
    <path d="M0 6 H16" className="stroke-ink-muted" strokeWidth={1.5} />
    <circle cx={18} cy={6} r={3.5} className="fill-ground stroke-ink-muted" strokeWidth={1.5} />
  </>
);

export const RIBBON_SYMBOL = <path d="M0 1 C12 1 12 5 24 5 V11 C12 11 12 7 0 7 Z" className="fill-mcc-0/55" />;

export const HYBRID_SYMBOL = (
  <>
    <path d="M0 6 H24" className="stroke-ink" strokeWidth={1.5} />
    <circle cx={12} cy={6} r={3.5} className="fill-ground stroke-signal" strokeWidth={1.5} />
  </>
);

export function lineSymbol(className: string, dashed = false) {
  return <path d="M0 6 H24" className={className} strokeWidth={1.5} strokeDasharray={dashed ? "4 3" : undefined} />;
}
