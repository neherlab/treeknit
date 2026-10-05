import type { ConstellationCell, ConstellationTable } from "@neherlab/treeknit-wasm";

import { leafCount, mccTitle } from "../drawing/format";
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
  return table.pairs.map(([a, b], pair) => ({ id: pairColumnId(pair), title: pairTitle(a, b), pair }));
}

function pairColumnId(pair: number): string {
  return `pair-${String(pair)}`;
}

function pairTitle(a: string, b: string): string {
  return `${a} and ${b}`;
}

export function cellLabel(cell: ConstellationCell, leaf: string, pair: string): string {
  return `${mccTitle(cell.mcc)} of ${leafCount(cell.size)}: show ${leaf} in ${pair}`;
}
