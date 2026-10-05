import type { MccInfo } from "@neherlab/treeknit-wasm";
import {
  columnFilteringFeature,
  createColumnHelper,
  createFilteredRowModel,
  createSortedRowModel,
  rowSortingFeature,
  sortFn_basic,
  sortFn_text,
  tableFeatures,
} from "@tanstack/react-table";

export const MCC_LEAVES_SHOWN = 3;

export const MCC_COLUMN = {
  mcc: "mcc",
  size: "size",
  leaves: "leaves",
  imputed: "imputed",
  ambiguous: "ambiguous",
} as const;

export type MccColumnId = (typeof MCC_COLUMN)[keyof typeof MCC_COLUMN];

export const MCC_HEADERS: Record<MccColumnId, string> = {
  mcc: "MCC",
  size: "Size",
  leaves: "Leaves",
  imputed: "Imputed members",
  ambiguous: "Ambiguous",
};

export function isMccColumn(id: string): id is MccColumnId {
  return Object.values<string>(MCC_COLUMN).includes(id);
}

export const mccTableFeatures = tableFeatures({
  rowSortingFeature,
  columnFilteringFeature,
  sortedRowModel: createSortedRowModel(),
  filteredRowModel: createFilteredRowModel(),
  sortFns: { basic: sortFn_basic, text: sortFn_text },
  filterFns: {},
});

const column = createColumnHelper<typeof mccTableFeatures, MccInfo>();

export function mccColumns(contains: (text: string, substring: string) => boolean) {
  return column.columns([
    column.accessor((mcc) => mcc.index, { id: MCC_COLUMN.mcc, header: MCC_HEADERS.mcc, sortFn: "basic" }),
    column.accessor((mcc) => mcc.size, { id: MCC_COLUMN.size, header: MCC_HEADERS.size, sortFn: "basic" }),
    column.accessor((mcc) => mcc.leaves[0] ?? "", {
      id: MCC_COLUMN.leaves,
      header: MCC_HEADERS.leaves,
      sortFn: "text",
      filterFn: (row, _columnId, query: string) => row.original.leaves.some((name) => contains(name, query.trim())),
    }),
    column.accessor((mcc) => mcc.imputedLeaves.length, {
      id: MCC_COLUMN.imputed,
      header: MCC_HEADERS.imputed,
      sortFn: "basic",
    }),
    column.accessor((mcc) => (mcc.ambiguousLeaves.length > 0 ? 1 : 0), {
      id: MCC_COLUMN.ambiguous,
      header: MCC_HEADERS.ambiguous,
      sortFn: "basic",
    }),
  ]);
}

export type SortDirection = "ascending" | "descending";

const SORT_DIRECTIONS = { asc: "ascending", desc: "descending" } as const satisfies Record<string, SortDirection>;

export function sortDirection(sorted: false | keyof typeof SORT_DIRECTIONS): SortDirection | undefined {
  return sorted === false ? undefined : SORT_DIRECTIONS[sorted];
}

export function mccsBySize(mccs: readonly MccInfo[]): MccInfo[] {
  return mccs.toSorted((a, b) => b.size - a.size || a.index - b.index);
}

export function leavesPreview(leaves: readonly string[]): string {
  const shown = leaves.slice(0, MCC_LEAVES_SHOWN).join(", ");
  const more = leaves.length - MCC_LEAVES_SHOWN;

  return more > 0 ? `${shown} and ${String(more)} more` : shown;
}
