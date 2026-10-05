import { cn } from "cn";
import { useCallback, useState } from "react";
import HideIcon from "~icons/lucide/chevron-up";

import { cssColor, type Rgba } from "../canvas/color";
import { Button } from "../ui/Button";
import { IconButton } from "../ui/IconButton";

export const SYMBOL_WIDTH_PX = 24;

export const SYMBOL_HEIGHT_PX = 12;

export type SymbolMark =
  | { kind: "line"; path: string; color: Rgba; widthPx: number; dashPx?: readonly [number, number] }
  | { kind: "ring"; at: readonly [number, number]; color: Rgba; fill: Rgba; radiusPx: number; lineWidthPx: number }
  | { kind: "area"; path: string; color: Rgba };

export interface LegendEntry {
  label: string;
  marks: readonly SymbolMark[];
}

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
        {entries.map(({ label, marks }) => (
          <li key={label} className="text-ink flex items-center gap-2">
            <svg
              aria-hidden
              width={SYMBOL_WIDTH_PX}
              height={SYMBOL_HEIGHT_PX}
              viewBox={`0 0 ${String(SYMBOL_WIDTH_PX)} ${String(SYMBOL_HEIGHT_PX)}`}
              className="shrink-0 overflow-visible"
            >
              {marks.map((mark, index) => (
                // oxlint-disable-next-line react/no-array-index-key -- the marks of a symbol are a fixed list drawn in order
                <Mark key={index} mark={mark} />
              ))}
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

function Mark({ mark }: { mark: SymbolMark }) {
  if (mark.kind === "ring") {
    return (
      <circle
        cx={mark.at[0]}
        cy={mark.at[1]}
        r={mark.radiusPx}
        fill={cssColor(mark.fill)}
        stroke={cssColor(mark.color)}
        strokeWidth={mark.lineWidthPx}
      />
    );
  }

  if (mark.kind === "area") {
    return <path d={mark.path} fill={cssColor(mark.color)} />;
  }

  return (
    <path
      d={mark.path}
      fill="none"
      stroke={cssColor(mark.color)}
      strokeWidth={mark.widthPx}
      strokeDasharray={mark.dashPx?.join(" ")}
    />
  );
}
