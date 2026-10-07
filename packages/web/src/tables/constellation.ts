import type { ConstellationCell, ConstellationTable } from "@neherlab/treeknit-wasm";

import { leafCount } from "../drawing/format";
import { itemAt } from "../drawing/lookup";

export const CONSTELLATION_INFO =
  "Each column is a pair of trees. Colors are not comparable across columns; the number is the size of the leaf's MCC in that pair.";

export const NOT_IN_PAIR = "Not in this pair";

export const LEAF_HEADER = "Leaf";

export interface ConstellationRow {
  leaf: string;
  cells: readonly (ConstellationCell | null)[];
}

export interface PairColumn {
  id: string;
  title: string;
  pair: number;
}

export function constellationRows(table: ConstellationTable): ConstellationRow[] {
  return table.leaves.map((leaf, index) => ({ leaf, cells: itemAt(table.cells, index, "constellation row") }));
}

export function pairColumns(table: ConstellationTable): PairColumn[] {
  return table.pairTitles.map((title, pair) => ({ id: pairColumnId(pair), title, pair }));
}

function pairColumnId(pair: number): string {
  return `pair-${String(pair)}`;
}

export function cellLabel(cell: ConstellationCell, mccNames: readonly string[], leaf: string, pair: string): string {
  return `${itemAt(mccNames, cell.mcc, "MCC name")} of ${leafCount(cell.size)}: show ${leaf} in ${pair}`;
}
