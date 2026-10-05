import type { ConstellationCell, ConstellationTable } from "@neherlab/treeknit-wasm";
import { createColumnHelper, tableFeatures } from "@tanstack/react-table";

import { leafCount, mccTitle } from "../drawing/format";

export const CONSTELLATION_INFO =
  "Each column is a pair of trees. Colors are not comparable across columns; the number is the size of the leaf's MCC in that pair.";

export const NOT_IN_PAIR = "Not in this pair";

export const LEAF_COLUMN = "leaf";

export interface ConstellationRow {
  leaf: string;
  cells: readonly (ConstellationCell | null)[];
}

export const constellationFeatures = tableFeatures({});

const column = createColumnHelper<typeof constellationFeatures, ConstellationRow>();

export function constellationRows(table: ConstellationTable): ConstellationRow[] {
  return table.leaves.map((leaf, index) => ({ leaf, cells: table.cells[index] ?? [] }));
}

export function constellationHeaders(table: ConstellationTable): { id: string; title: string }[] {
  return [
    { id: LEAF_COLUMN, title: "Leaf" },
    ...table.pairs.map(([a, b], pair) => ({ id: pairColumnId(pair), title: pairTitle(a, b) })),
  ];
}

export function constellationColumns(table: ConstellationTable) {
  const [leaf, ...pairs] = constellationHeaders(table);

  return column.columns([
    column.accessor((row) => row.leaf, { id: LEAF_COLUMN, header: leaf?.title ?? LEAF_COLUMN }),
    ...pairs.map(({ id, title }, pair) => column.accessor((row) => row.cells[pair] ?? null, { id, header: title })),
  ]);
}

export function pairColumnId(pair: number): string {
  return `pair-${String(pair)}`;
}

export function pairTitle(a: string, b: string): string {
  return `${a} and ${b}`;
}

export function cellLabel(cell: ConstellationCell, leaf: string, pair: string): string {
  return `${mccTitle(cell.mcc)} of ${leafCount(cell.size)}: show ${leaf} in ${pair}`;
}
