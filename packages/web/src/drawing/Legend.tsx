import type { Rgba } from "@neherlab/treeknit-wasm";

import { cssColor } from "../canvas/color";
import { SYMBOL_HEIGHT_PX, SYMBOL_WIDTH_PX } from "./legendSymbols";

export type SymbolMark =
  | { kind: "line"; path: string; color: Rgba; widthPx: number; dashPx?: readonly [number, number] }
  | { kind: "ring"; at: readonly [number, number]; color: Rgba; fill: Rgba; radiusPx: number; lineWidthPx: number }
  | { kind: "area"; path: string; color: Rgba };

export interface LegendEntry {
  label: string;
  marks: readonly SymbolMark[];
}

export function Legend({ entries }: LegendProps) {
  return (
    <section aria-label="Legend" className="border-rule shrink-0 border-b px-3 py-1 text-xs">
      <ul className="flex flex-wrap items-center gap-x-5 gap-y-1">
        {entries.map(({ label, marks }) => (
          <li key={label} className="text-ink-muted flex items-center gap-1.5">
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
    </section>
  );
}

export interface LegendProps {
  entries: readonly LegendEntry[];
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
