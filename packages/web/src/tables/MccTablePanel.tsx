import type { MccInfo, PairView } from "@neherlab/treeknit-wasm";
import { type Cell, type Header, useTable } from "@tanstack/react-table";
import { cn } from "cn";
import { type ReactNode, useCallback, useMemo, useRef, useState } from "react";
import { Button, useFilter } from "react-aria-components";
import ShowIcon from "~icons/lucide/git-compare-arrows";

import { usePairView } from "../analysis/queries";
import { requestFocus } from "../drawing/focus";
import { leafCount, mccTitle } from "../drawing/format";
import { mccInTanglegram } from "../drawing/navigation";
import { withSelection } from "../drawing/selection";
import { counted, formatCount } from "../format/count";
import { NONE, yesNo } from "../format/words";
import { IconButton } from "../ui/IconButton";
import { InfoButton } from "../ui/InfoButton";
import { MccSwatch } from "../ui/MccSwatch";
import { QueryState } from "../ui/QueryState";
import { focusRing } from "../ui/styles";
import {
  type CellAlign,
  cellStyle,
  columnStyle,
  nativeRowStyle,
  SortIndicator,
  tableHeaderStyle,
  tableStyle,
} from "../ui/Table";
import { TextField } from "../ui/TextField";
import { useVirtualRows } from "../ui/useVirtualRows";
import { VirtualGap } from "../ui/VirtualGap";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import {
  hasAmbiguousAttachment,
  isMccColumn,
  leavesPreview,
  MCC_COLUMN,
  MCC_HEADERS,
  type MccColumnId,
  mccColumns,
  mccTableFeatures,
  sortDirection,
} from "./mccTable";

const ALIGN_END: ReadonlySet<string> = new Set([MCC_COLUMN.size, MCC_COLUMN.imputed]);

type MccHeader = Header<typeof mccTableFeatures, MccInfo>;

type MccCell = Cell<typeof mccTableFeatures, MccInfo>;

export function MccTablePanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <MccTableQuery result={result} />;
}

function MccTableQuery({ result }: { result: RunResult }) {
  const { search } = useWorkspaceSearch();
  const query = usePairView(result.sessionId, search.pair, search.version, search.x);
  const labels = result.summary.pairs[search.pair]?.labels;

  return (
    <QueryState query={query} loading="Loading the MCCs" errorTitle="The MCCs could not be loaded">
      {(data) => (
        <MccTable data={data} title={labels === undefined ? "MCCs" : `MCCs of ${labels[0]} and ${labels[1]}`} />
      )}
    </QueryState>
  );
}

function MccTable({ data, title }: { data: PairView; title: string }) {
  const { search, update } = useWorkspaceSearch();
  const { contains } = useFilter({ sensitivity: "base" });
  const columns = useMemo(() => mccColumns(contains), [contains]);
  const table = useTable({ features: mccTableFeatures, columns, data: data.mccs });
  const [query, setQuery] = useState("");
  const rows = table.getRowModel().rows;
  const scrollRef = useRef<HTMLDivElement>(null);
  const { items, before, after, measureRow, headerRef } = useVirtualRows(rows.length, scrollRef);
  const headers = table.getHeaderGroups()[0]?.headers ?? [];

  const filter = useCallback(
    (text: string) => {
      setQuery(text);
      table.setColumnFilters(text.trim() === "" ? [] : [{ id: MCC_COLUMN.leaves, value: text }]);
    },
    [table],
  );

  const select = useCallback(
    (mcc: number) => {
      update((written) => withSelection(written, { mcc }));
    },
    [update],
  );

  const show = useCallback(
    (mcc: number) => {
      update((written) => mccInTanglegram(written, mcc));
      requestFocus({ kind: "mcc", mcc });
    },
    [update],
  );

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-rule flex shrink-0 flex-wrap items-center gap-3 border-b px-3 py-1.5">
        <h2 className="text-ink flex items-center gap-1 text-sm font-semibold">
          {title}
          <InfoButton topic="the MCC table">
            <p>
              Each row is a maximally compatible clade: leaves whose subtrees have the same topology in both trees of
              the pair. Choose a row to select its MCC, and use the arrow button to see it in the tanglegram.
            </p>
          </InfoButton>
        </h2>
        <span className="text-ink-muted text-sm">{`${formatCount(rows.length)} of ${counted(data.mccs.length, "MCC", "MCCs")}`}</span>
        <TextField
          label="Filter by leaf name"
          labelHidden
          placeholder="Filter by leaf name"
          value={query}
          onChange={filter}
          className="ml-auto w-56"
        />
      </div>
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto">
        <table aria-label={title} aria-rowcount={rows.length + 1} className={tableStyle}>
          <thead ref={headerRef} className={tableHeaderStyle}>
            <tr aria-rowindex={1}>
              {headers.map((header) => (
                <SortHeader key={header.id} header={header} />
              ))}
              <th scope="col" className={columnStyle("start")}>
                <span className="sr-only">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <VirtualGap heightPx={before} />
            {items.map(({ index }) => {
              const row = rows[index];

              return row === undefined ? null : (
                <MccRow
                  key={row.id}
                  cells={row.getAllCells()}
                  mcc={row.original}
                  index={index}
                  selected={search.mcc === row.original.index}
                  measure={measureRow}
                  onSelect={select}
                  onShow={show}
                />
              );
            })}
            <VirtualGap heightPx={after} />
          </tbody>
        </table>
        {rows.length === 0 ? (
          <p className="text-ink-muted px-3 py-4 text-sm">No MCC has a leaf with this name.</p>
        ) : null}
      </div>
    </div>
  );
}

function SortHeader({ header }: { header: MccHeader }) {
  const { column } = header;
  const direction = sortDirection(column.getIsSorted());
  const align = alignOf(column.id);

  const toggle = useCallback(() => {
    column.toggleSorting();
  }, [column]);

  return (
    <th scope="col" aria-sort={direction ?? "none"} className={columnStyle(align)}>
      <Button
        onPress={toggle}
        className={cn(
          "rounded-inner data-hovered:text-ink inline-flex items-center gap-1",
          align === "end" && "flex-row-reverse",
          focusRing,
        )}
      >
        {isMccColumn(column.id) ? MCC_HEADERS[column.id] : column.id}
        <SortIndicator direction={direction} />
      </Button>
    </th>
  );
}

function MccRow({ cells, mcc, index, selected, measure, onSelect, onShow }: MccRowProps) {
  const choose = useCallback(() => {
    onSelect(mcc.index);
  }, [mcc.index, onSelect]);

  const show = useCallback(() => {
    onShow(mcc.index);
  }, [mcc.index, onShow]);

  const content: Record<MccColumnId, ReactNode> = {
    [MCC_COLUMN.mcc]: (
      <Button
        aria-pressed={selected}
        onPress={choose}
        className={cn("rounded-inner inline-flex items-center gap-2 font-sans", selected && "font-semibold", focusRing)}
      >
        <MccSwatch slot={mcc.slot} />
        {mccTitle(mcc.index)}
      </Button>
    ),
    [MCC_COLUMN.size]: leafCount(mcc.size),
    [MCC_COLUMN.leaves]: <span className="block max-w-[48ch] truncate">{leavesPreview(mcc.leaves)}</span>,
    [MCC_COLUMN.imputed]: mcc.imputedLeaves.length === 0 ? NONE : formatCount(mcc.imputedLeaves.length),
    [MCC_COLUMN.ambiguous]: yesNo(hasAmbiguousAttachment(mcc)),
  };

  return (
    <tr
      ref={measure}
      data-index={index}
      aria-rowindex={index + 2}
      data-selected={selected || undefined}
      className={nativeRowStyle}
    >
      {cells.map((cell) => (
        <td key={cell.id} className={cellStyle(alignOf(cell.column.id))}>
          {isMccColumn(cell.column.id) ? content[cell.column.id] : null}
        </td>
      ))}
      <td className={cn(cellStyle("start"), "w-10")}>
        <IconButton label={`Show ${mccTitle(mcc.index)} in the tanglegram`} icon={ShowIcon} size="xs" onPress={show} />
      </td>
    </tr>
  );
}

interface MccRowProps {
  cells: readonly MccCell[];
  mcc: MccInfo;
  index: number;
  selected: boolean;
  measure: (row: HTMLTableRowElement | null) => void;
  onSelect: (mcc: number) => void;
  onShow: (mcc: number) => void;
}

function alignOf(id: string): CellAlign {
  return ALIGN_END.has(id) ? "end" : "start";
}
