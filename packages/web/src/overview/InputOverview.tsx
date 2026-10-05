import { useCallback, useMemo } from "react";
import BlockedIcon from "~icons/lucide/circle-alert";

import { useInspectTrees, useOverlap } from "../analysis/queries";
import { formatCount } from "../drawing/format";
import { useDelayedIndicator } from "../indicator/useDelayedIndicator";
import { BRANCH_LENGTH_VALUES } from "../inputs/treeStatus";
import { InfoButton } from "../ui/InfoButton";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../ui/Table";
import { useCurrentRequest } from "../workspace/context";
import { Matrix } from "./Matrix";
import { type MatrixCell, type OverlapCell, overlapMatrix } from "./overviewModel";

export const BLOCKED_PAIR = "shares fewer than two leaves";

const COUNTING_SHARED_LEAVES = "Counting shared leaves";

export function InputOverview() {
  const request = useCurrentRequest();
  const inspections = useInspectTrees(request.trees);
  const { data: overlap, isPending: overlapPending } = useOverlap(request.trees);
  const labels = useMemo(() => request.trees.map(({ label }) => label), [request.trees]);
  const counting = useDelayedIndicator(overlapPending && labels.length >= 2);
  const matrix = useMemo(() => (overlap === undefined ? null : overlapMatrix(overlap, labels)), [labels, overlap]);
  const anyBlocked = overlap?.pairs.some(({ blocked }) => blocked) ?? false;

  const leaves = useMemo(
    () => new Map(overlap?.trees.map(({ index, leaves: count }) => [index, count]) ?? []),
    [overlap],
  );

  const missing = useMemo(
    () => new Map(overlap?.trees.map(({ index, missing: count }) => [index, count]) ?? []),
    [overlap],
  );

  const renderOverlap = useCallback(
    ({ row, column, value }: MatrixCell<OverlapCell>) => {
      if (row === column) {
        return <span className="text-ink-muted">{format(leaves.get(row))}</span>;
      }

      if (value === null) {
        return null;
      }

      return value.blocked ? (
        <span className="text-danger inline-flex items-center gap-1 font-medium">
          <BlockedIcon aria-hidden />
          {format(value.shared)}
          <span className="sr-only">, {BLOCKED_PAIR}</span>
        </span>
      ) : (
        format(value.shared)
      );
    },
    [leaves],
  );

  return (
    <section aria-labelledby="overview-trees" className="flex flex-col gap-6">
      <h2 id="overview-trees" className="text-base font-semibold">
        Trees
      </h2>
      <div className="max-w-full overflow-x-auto">
        <Table aria-label="Trees">
          <TableHeader>
            <Column isRowHeader>Label</Column>
            <Column align="end">Leaves</Column>
            <Column align="end">Missing leaves</Column>
            <Column align="end">Polytomies</Column>
            <Column>Branch lengths</Column>
          </TableHeader>
          <TableBody>
            {request.trees.map((tree, index) => {
              const inspection = inspections[index];
              const failed = inspection?.error !== null && inspection?.error !== undefined;

              return (
                <Row key={`tree-${String(index)}`} id={`tree-${String(index)}`}>
                  <Cell className="font-medium">{tree.label}</Cell>
                  <Cell align="end">
                    {failed ? (
                      <span className="text-danger inline-flex items-center gap-1">
                        <BlockedIcon aria-hidden />
                        Does not parse
                      </span>
                    ) : (
                      format(inspection?.leaves)
                    )}
                  </Cell>
                  <Cell align="end">{failed ? null : format(missing.get(index))}</Cell>
                  <Cell align="end">{failed ? null : format(inspection?.polytomies)}</Cell>
                  <Cell>
                    {failed || inspection === undefined ? null : BRANCH_LENGTH_VALUES[inspection.branchLengths]}
                  </Cell>
                </Row>
              );
            })}
          </TableBody>
        </Table>
      </div>
      {matrix === null && counting ? (
        <output className="text-ink-muted text-sm">{COUNTING_SHARED_LEAVES}</output>
      ) : null}
      {matrix === null || labels.length < 2 ? null : (
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-1">
            <h3 className="text-sm font-semibold">Shared leaves</h3>
            <InfoButton topic="shared leaves">
              <p>
                Each cell is the number of leaves that the two trees share, and the diagonal is the number of leaves of
                each tree. TreeKnit compares every pair of trees on the leaves they share.
              </p>
            </InfoButton>
          </div>
          <Matrix label="Shared leaves" matrix={matrix} renderCell={renderOverlap} />
          {anyBlocked ? (
            <p className="text-danger flex items-center gap-1 text-xs">
              <BlockedIcon aria-hidden />
              Pair {BLOCKED_PAIR}
            </p>
          ) : null}
        </div>
      )}
    </section>
  );
}

function format(value: number | undefined): string {
  return value === undefined ? "" : formatCount(value);
}
