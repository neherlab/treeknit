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
import { cellStyle, columnStyle, nativeRowStyle, tableHeaderStyle, tableStyle } from "../ui/Table";
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
  const { items, before, after, measureRow, headerRef } = useVirtualRows(rows.length, scrollRef);
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
        <table aria-label="Constellation" aria-rowcount={rows.length + 1} className={cn(tableStyle, "w-auto")}>
          <thead ref={headerRef} className={cn(tableHeaderStyle, "z-[2]")}>
            <tr aria-rowindex={1}>
              {headers.map(({ id, title }) => (
                <th key={id} scope="col" className={cn(columnStyle("start"), id === LEAF_COLUMN && STICKY_COLUMN)}>
                  {title}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            <VirtualGap heightPx={before} />
            {items.map(({ index }) => {
              const row = shown[index]?.original;

              return row === undefined ? null : (
                <LeafRow key={row.leaf} row={row} index={index} titles={titles} measure={measureRow} onOpen={open} />
              );
            })}
            <VirtualGap heightPx={after} />
          </tbody>
        </table>
      </div>
    </div>
  );
}

function LeafRow({ row, index, titles, measure, onOpen }: LeafRowProps) {
  return (
    <tr ref={measure} data-index={index} aria-rowindex={index + 2} className={nativeRowStyle}>
      <th scope="row" className={cn(cellStyle("start"), STICKY_COLUMN, "max-w-[32ch] truncate font-normal")}>
        {row.leaf}
      </th>
      {titles.map((title, pair) => (
        <td key={title} className={cellStyle("start")}>
          <PairCell cell={row.cells[pair] ?? null} leaf={row.leaf} pair={pair} title={title} onOpen={onOpen} />
        </td>
      ))}
    </tr>
  );
}

interface LeafRowProps {
  row: ConstellationRow;
  index: number;
  titles: readonly string[];
  measure: (row: HTMLTableRowElement | null) => void;
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
