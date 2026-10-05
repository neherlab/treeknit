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
    noReassortment: noReassortmentFound(summary, labels.length),
    matrix: labels.length > 2 ? pairMatrix(summary.pairs, labels) : null,
  };
}

export function noReassortmentFound(summary: Summary, treeCount: number): boolean {
  if (treeCount === 2) {
    return summary.arg?.status === "built" && summary.arg.reassortments === 0;
  }

  return summary.pairs.length > 0 && summary.pairs.every(({ mccCount }) => mccCount === 1);
}

export function pipelinePairs(treeCount: number): readonly (readonly [number, number])[] {
  return Array.from({ length: treeCount }, (_, i) =>
    Array.from({ length: treeCount - i - 1 }, (__, offset) => [i, i + offset + 1] as const),
  ).flat();
}

export function pairMatrix(pairs: readonly PairSummary[], labels: readonly string[]): Matrix<PairCell> {
  const cells = new Map<string, PairCell>();

  pipelinePairs(labels.length).forEach(([i, j], index) => {
    const pair = pairs[index];

    if (pair !== undefined) {
      cells.set(cellKey(i, j), { pair: pair.index, mccCount: pair.mccCount });
    }
  });

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
