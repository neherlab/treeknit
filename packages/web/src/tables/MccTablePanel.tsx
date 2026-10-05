import type { MccInfo, PairView } from "@neherlab/treeknit-wasm";
import { type Header, useTable } from "@tanstack/react-table";
import { cn } from "cn";
import { type ReactNode, useCallback, useMemo, useRef, useState } from "react";
import { Button, useFilter } from "react-aria-components";
import { getErrorMessage } from "react-error-boundary";
import SortDescIcon from "~icons/lucide/arrow-down";
import SortAscIcon from "~icons/lucide/arrow-up";
import ShowIcon from "~icons/lucide/git-compare-arrows";

import { usePairView } from "../analysis/queries";
import { requestFocus } from "../drawing/focus";
import { leafCount, mccTitle } from "../drawing/format";
import { MccSwatch } from "../drawing/MccSwatch";
import { mccInTanglegram } from "../drawing/navigation";
import { withSelection } from "../drawing/selection";
import { IconButton } from "../ui/IconButton";
import { InfoButton } from "../ui/InfoButton";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { focusRing } from "../ui/styles";
import { TextField } from "../ui/TextField";
import { useVirtualRows } from "../ui/useVirtualRows";
import { VirtualGap } from "../ui/VirtualGap";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import {
  isMccColumn,
  leavesPreview,
  MCC_COLUMN,
  MCC_HEADERS,
  type MccColumnId,
  mccColumns,
  mccTableFeatures,
} from "./mccTable";
import { ARIA_SORT, cellStyle, headerStyle, tableStyle } from "./styles";

const ALIGN_END: ReadonlySet<string> = new Set([MCC_COLUMN.size, MCC_COLUMN.imputed]);

export function MccTablePanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <MccTableQuery result={result} />;
}

function MccTableQuery({ result }: { result: RunResult }) {
  const { search } = useWorkspaceSearch();
  const query = usePairView(result.sessionId, search.pair, search.version, search.x);
  const labels = result.summary.pairs[search.pair]?.labels;

  if (query.isError) {
    return (
      <div className="p-3">
        <InlineNotice tone="danger" title="The MCCs could not be loaded">
          {getErrorMessage(query.error) ?? String(query.error)}
        </InlineNotice>
      </div>
    );
  }

  if (query.isPending) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <ProgressBar label="Loading the MCCs" isIndeterminate className="w-64" />
      </div>
    );
  }

  return <MccTable data={query.data} title={labels === undefined ? "MCCs" : `MCCs of ${labels[0]} and ${labels[1]}`} />;
}

function MccTable({ data, title }: { data: PairView; title: string }) {
  const { search, update } = useWorkspaceSearch();
  const { contains } = useFilter({ sensitivity: "base" });
  const columns = useMemo(() => mccColumns(contains), [contains]);
  const table = useTable({ features: mccTableFeatures, columns, data: data.mccs });
  const [query, setQuery] = useState("");
  const rows = table.getRowModel().rows;
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtual = useVirtualRows(rows.length, scrollRef);
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
        <span className="text-ink-muted text-sm">{`${String(rows.length)} of ${String(data.mccs.length)} MCCs`}</span>
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
        <table className={tableStyle} aria-rowcount={rows.length + 1}>
          <thead className="bg-ground sticky top-0 z-10">
            <tr aria-rowindex={1}>
              {headers.map((header) => (
                <SortHeader key={header.id} header={header} />
              ))}
              <th className={headerStyle}>
                <span className="sr-only">Actions</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <VirtualGap heightPx={virtual.before} />
            {virtual.items.map(({ index }) => {
              const row = rows[index];

              return row === undefined ? null : (
                <MccRow
                  key={row.id}
                  mcc={row.original}
                  rowIndex={index + 2}
                  selected={search.mcc === row.original.index}
                  onSelect={select}
                  onShow={show}
                />
              );
            })}
            <VirtualGap heightPx={virtual.after} />
          </tbody>
        </table>
        {rows.length === 0 ? (
          <p className="text-ink-muted px-3 py-4 text-sm">No MCC has a leaf with this name.</p>
        ) : null}
      </div>
    </div>
  );
}

function SortHeader({ header }: { header: Header<typeof mccTableFeatures, MccInfo> }) {
  const { column } = header;
  const sorted = column.getIsSorted();
  const label = isMccColumn(column.id) ? MCC_HEADERS[column.id] : column.id;
  const end = ALIGN_END.has(column.id);

  const toggle = useCallback(() => {
    column.toggleSorting();
  }, [column]);

  return (
    <th aria-sort={sorted === false ? "none" : ARIA_SORT[sorted]} className={cn(headerStyle, end && "text-right")}>
      <Button
        onPress={toggle}
        className={cn(
          "rounded-inner data-hovered:text-ink inline-flex items-center gap-1",
          end && "flex-row-reverse",
          focusRing,
        )}
      >
        {label}
        {sorted === "asc" ? <SortAscIcon aria-hidden className="size-3.5" /> : null}
        {sorted === "desc" ? <SortDescIcon aria-hidden className="size-3.5" /> : null}
      </Button>
    </th>
  );
}

function MccRow({ mcc, rowIndex, selected, onSelect, onShow }: MccRowProps) {
  const choose = useCallback(() => {
    onSelect(mcc.index);
  }, [mcc.index, onSelect]);

  const show = useCallback(() => {
    onShow(mcc.index);
  }, [mcc.index, onShow]);

  const cells: Record<MccColumnId, ReactNode> = {
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
    [MCC_COLUMN.imputed]: mcc.imputedLeaves.length === 0 ? "none" : String(mcc.imputedLeaves.length),
    [MCC_COLUMN.ambiguous]: mcc.ambiguous ? "yes" : "no",
  };

  return (
    <tr aria-rowindex={rowIndex} className={cn("h-8", selected ? "bg-pane" : "hover:bg-ink/4")}>
      {Object.values(MCC_COLUMN).map((id) => (
        <td key={id} className={cn(cellStyle, ALIGN_END.has(id) && "text-right")}>
          {cells[id]}
        </td>
      ))}
      <td className={cn(cellStyle, "w-10")}>
        <IconButton label={`Show ${mccTitle(mcc.index)} in the tanglegram`} icon={ShowIcon} size="xs" onPress={show} />
      </td>
    </tr>
  );
}

interface MccRowProps {
  mcc: MccInfo;
  rowIndex: number;
  selected: boolean;
  onSelect: (mcc: number) => void;
  onShow: (mcc: number) => void;
}
