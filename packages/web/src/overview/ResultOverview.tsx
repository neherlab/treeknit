import { type ReactNode, useCallback, useMemo } from "react";
import { match } from "ts-pattern";

import { counted, formatCount } from "../format/count";
import { runTime } from "../run/runControl";
import { InfoButton } from "../ui/InfoButton";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { Link } from "../ui/Link";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "../ui/Table";
import { selectPair, type WrittenSearch } from "../workspace/search";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { Matrix } from "./Matrix";
import { type MatrixCell, type PairCell, resultOverview } from "./overviewModel";

export const NO_REASSORTMENT_FOUND = "No reassortment found.";

export const ARG_NOT_BUILT = "ARG not built";

export function ResultOverview({ result }: ResultOverviewProps) {
  const labels = useMemo(() => result.request.trees.map(({ label }) => label), [result.request.trees]);
  const overview = useMemo(() => resultOverview(result.summary, labels), [labels, result.summary]);

  const renderPairCell = useCallback(
    ({ value }: MatrixCell<PairCell>) =>
      value === null ? null : <PairLink pair={value.pair}>{formatCount(value.mccCount)}</PairLink>,
    [],
  );

  return (
    <section aria-labelledby="overview-results" className="flex flex-col gap-6">
      <h2 id="overview-results" className="text-base font-semibold">
        Results
      </h2>
      {overview.arg === null ? null : (
        <dl className="flex flex-wrap gap-x-10 gap-y-4">
          {match(overview.arg)
            .with({ status: "built" }, ({ reassortments }) => (
              <Figure label="Reassortments" value={formatCount(reassortments)} />
            ))
            .with({ status: "failed" }, () => null)
            .exhaustive()}
          {overview.rows.map((row) => (
            <Figure key={row.index} label="MCCs" value={formatCount(row.mccCount)} />
          ))}
        </dl>
      )}
      <NoticeRegion>
        {overview.noReassortment ? <InlineNotice tone="info" title={NO_REASSORTMENT_FOUND} /> : null}
      </NoticeRegion>
      {overview.arg?.status === "failed" ? (
        <InlineNotice tone="warning" title={ARG_NOT_BUILT}>
          {overview.arg.message}
        </InlineNotice>
      ) : null}
      <div className="max-w-full overflow-x-auto">
        <Table aria-label="Pairs">
          <TableHeader>
            <Column isRowHeader>Pair</Column>
            <Column align="end">MCCs</Column>
            <Column align="end">Imputed leaves</Column>
          </TableHeader>
          <TableBody>
            {overview.rows.map((row) => (
              <Row key={row.index} id={row.index}>
                <Cell>
                  <PairLink pair={row.index}>
                    {row.labels[0]} and {row.labels[1]}
                  </PairLink>
                </Cell>
                <Cell align="end">{formatCount(row.mccCount)}</Cell>
                <Cell align="end">
                  {formatCount(row.imputedCount)}
                  {row.ambiguousCount === 0 ? null : (
                    <span className="text-ink-muted"> ({formatCount(row.ambiguousCount)} ambiguous)</span>
                  )}
                </Cell>
              </Row>
            ))}
          </TableBody>
        </Table>
      </div>
      {overview.matrix === null ? null : (
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-1">
            <h3 className="text-sm font-semibold">MCCs per pair</h3>
            <InfoButton topic="MCCs per pair">
              <p>
                Each cell is the number of MCCs of a pair of trees. One MCC means that the two trees share their
                topology. Choose a number to open the tanglegram of that pair.
              </p>
            </InfoButton>
          </div>
          <Matrix label="MCCs per pair" matrix={overview.matrix} renderCell={renderPairCell} />
        </div>
      )}
      <p className="text-ink-muted text-xs">
        {counted(labels.length, "tree", "trees")}, {counted(overview.rows.length, "pair", "pairs")}, finished in{" "}
        {runTime(result.durationMs)}
      </p>
    </section>
  );
}

export interface ResultOverviewProps {
  result: RunResult;
}

function Figure({ label, value }: FigureProps) {
  return (
    <div className="flex flex-col">
      <dt className="text-ink-muted text-xs">{label}</dt>
      <dd className="text-lg font-semibold tabular-nums">{value}</dd>
    </div>
  );
}

interface FigureProps {
  label: string;
  value: string;
}

function PairLink({ pair, children }: PairLinkProps) {
  const { pairLabels } = useWorkspaceSearch();

  const target = useCallback(
    (search: WrittenSearch): WrittenSearch => ({ ...selectPair(search, pairLabels, pair), view: "tanglegram" }),
    [pairLabels, pair],
  );

  return (
    <Link from="/" to="/" search={target}>
      {children}
    </Link>
  );
}

interface PairLinkProps {
  pair: number;
  children: ReactNode;
}
