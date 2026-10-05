import type { ArgOutcome, Overlap, PairSummary, Summary } from "@neherlab/treeknit-wasm";

export interface PairRow {
  index: number;
  labels: readonly [string, string];
  mccCount: number;
  imputedCount: number;
  ambiguousCount: number;
}

export interface MatrixCell<T> {
  row: number;
  column: number;
  value: T | null;
}

export interface Matrix<T> {
  labels: readonly string[];
  rows: readonly (readonly MatrixCell<T>[])[];
}

export interface PairCell {
  pair: number;
  mccCount: number;
}

export interface OverlapCell {
  shared: number;
  blocked: boolean;
}

export interface ResultOverview {
  rows: readonly PairRow[];
  arg: ArgOutcome | null;
  noReassortment: boolean;
  matrix: Matrix<PairCell> | null;
}

export function resultOverview(summary: Summary, labels: readonly string[]): ResultOverview {
  return {
    rows: summary.pairs.map(pairRow),
    arg: labels.length === 2 ? summary.arg : null,
    noReassortment: summary.noReassortment,
    matrix: labels.length > 2 ? pairMatrix(summary.pairs, labels) : null,
  };
}

function pairMatrix(pairs: readonly PairSummary[], labels: readonly string[]): Matrix<PairCell> {
  const cells = new Map<string, PairCell>(
    pairs.map(({ index, trees: [i, j], mccCount }) => [cellKey(i, j), { pair: index, mccCount }]),
  );

  return symmetricMatrix(labels, cells);
}

export function overlapMatrix(overlap: Overlap, labels: readonly string[]): Matrix<OverlapCell> {
  const cells = new Map<string, OverlapCell>(
    overlap.pairs.map(({ i, j, shared, blocked }) => [cellKey(i, j), { shared, blocked }]),
  );

  return symmetricMatrix(labels, cells);
}

function pairRow({ index, labels, mccCount, imputedCount, ambiguousCount }: PairSummary): PairRow {
  return { index, labels, mccCount, imputedCount, ambiguousCount };
}

function symmetricMatrix<T>(labels: readonly string[], cells: ReadonlyMap<string, T>): Matrix<T> {
  return {
    labels,
    rows: labels.map((_, row) =>
      labels.map((__, column) => ({
        row,
        column,
        value: row === column ? null : (cells.get(cellKey(Math.min(row, column), Math.max(row, column))) ?? null),
      })),
    ),
  };
}

function cellKey(i: number, j: number): string {
  return `${String(i)}:${String(j)}`;
}
