import type { ConstellationCell, ConstellationTable } from "@neherlab/treeknit-wasm";
import { useTable } from "@tanstack/react-table";
import { cn } from "cn";
import { useCallback, useMemo, useRef } from "react";
import { Button } from "react-aria-components";

import { useConstellation } from "../analysis/queries";
import { requestFocus } from "../drawing/focus";
import { counted, formatCount } from "../drawing/format";
import { leafInPair } from "../drawing/navigation";
import { InfoButton } from "../ui/InfoButton";
import { MccSwatch } from "../ui/MccSwatch";
import { QueryState } from "../ui/QueryState";
import { focusRing } from "../ui/styles";
import { useVirtualRows } from "../ui/useVirtualRows";
import { VirtualGap } from "../ui/VirtualGap";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import {
  cellLabel,
  CONSTELLATION_INFO,
  type ConstellationRow,
  constellationColumns,
  constellationFeatures,
  constellationHeaders,
  constellationRows,
  LEAF_COLUMN,
  NOT_IN_PAIR,
  pairTitle,
} from "./constellation";
import { cellStyle, headerStyle, tableStyle } from "./styles";

const STICKY_COLUMN = "bg-ground sticky left-0 z-[1]";

export function ConstellationPanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <ConstellationQuery result={result} />;
}

function ConstellationQuery({ result }: { result: RunResult }) {
  const query = useConstellation(result.sessionId);

  return (
    <QueryState query={query} loading="Loading the constellation" errorTitle="The constellation could not be loaded">
      {(data) => <Constellation data={data} />}
    </QueryState>
  );
}

function Constellation({ data }: { data: ConstellationTable }) {
  const { update } = useWorkspaceSearch();
  const rows = useMemo(() => constellationRows(data), [data]);
  const columns = useMemo(() => constellationColumns(data), [data]);
  const table = useTable({ features: constellationFeatures, columns, data: rows });
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtual = useVirtualRows(rows.length, scrollRef);
  const shown = table.getRowModel().rows;
  const headers = useMemo(() => constellationHeaders(data), [data]);
  const titles = useMemo(() => data.pairs.map(([a, b]) => pairTitle(a, b)), [data]);

  const open = useCallback(
    (pair: number, leaf: string) => {
      update((written) => leafInPair(written, pair, leaf));
      requestFocus({ kind: "leaf", name: leaf });
    },
    [update],
  );

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-rule flex shrink-0 items-center gap-3 border-b px-3 py-1.5">
        <h2 className="text-ink flex items-center gap-1 text-sm font-semibold">
          Constellation
          <InfoButton topic="the constellation">
            <p>{CONSTELLATION_INFO}</p>
          </InfoButton>
        </h2>
        <span className="text-ink-muted text-sm">{`${counted(rows.length, "leaf", "leaves")}, ${counted(data.pairs.length, "pair", "pairs")}`}</span>
      </div>
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto">
        <table className={cn(tableStyle, "w-auto")} aria-rowcount={rows.length + 1}>
          <thead className="bg-ground sticky top-0 z-[2]">
            <tr aria-rowindex={1}>
              {headers.map(({ id, title }) => (
                <th key={id} scope="col" className={cn(headerStyle, id === LEAF_COLUMN && STICKY_COLUMN)}>
                  {title}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            <VirtualGap heightPx={virtual.before} />
            {virtual.items.map(({ index }) => {
              const row = shown[index]?.original;

              return row === undefined ? null : (
                <LeafRow key={row.leaf} row={row} rowIndex={index + 2} titles={titles} onOpen={open} />
              );
            })}
            <VirtualGap heightPx={virtual.after} />
          </tbody>
        </table>
      </div>
    </div>
  );
}

function LeafRow({ row, rowIndex, titles, onOpen }: LeafRowProps) {
  return (
    <tr aria-rowindex={rowIndex} className="hover:bg-ink/4 h-8">
      <th scope="row" className={cn(cellStyle, STICKY_COLUMN, "max-w-[32ch] truncate text-left font-normal")}>
        {row.leaf}
      </th>
      {titles.map((title, pair) => (
        <td key={title} className={cellStyle}>
          <PairCell cell={row.cells[pair] ?? null} leaf={row.leaf} pair={pair} title={title} onOpen={onOpen} />
        </td>
      ))}
    </tr>
  );
}

interface LeafRowProps {
  row: ConstellationRow;
  rowIndex: number;
  titles: readonly string[];
  onOpen: (pair: number, leaf: string) => void;
}

function PairCell({ cell, leaf, pair, title, onOpen }: PairCellProps) {
  const open = useCallback(() => {
    onOpen(pair, leaf);
  }, [leaf, onOpen, pair]);

  if (cell === null) {
    return (
      <span title={NOT_IN_PAIR} className="text-ink-muted">
        <span aria-hidden>-</span>
        <span className="sr-only">{NOT_IN_PAIR}</span>
      </span>
    );
  }

  return (
    <Button
      aria-label={cellLabel(cell, leaf, title)}
      onPress={open}
      className={cn("rounded-inner data-hovered:text-ink inline-flex items-center gap-1.5 tabular-nums", focusRing)}
    >
      <MccSwatch slot={cell.slot} />
      {formatCount(cell.size)}
    </Button>
  );
}

interface PairCellProps {
  cell: ConstellationCell | null;
  leaf: string;
  pair: number;
  title: string;
  onOpen: (pair: number, leaf: string) => void;
}
