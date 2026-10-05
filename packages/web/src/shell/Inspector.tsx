import type { ArgNodeView, DrawNode, MccInfo } from "@neherlab/treeknit-wasm";
import { type ReactNode, useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";
import { match } from "ts-pattern";
import ZoomIcon from "~icons/lucide/scan-search";

import { currentData, useArgView, useConstellation, usePairView } from "../analysis/queries";
import { requestFocus } from "../drawing/focus";
import { formatBranchLength, leafCount, mccSummary, mccTitle } from "../drawing/format";
import { leafInPair, mccInTanglegram } from "../drawing/navigation";
import { selectionOf, type Selection, withSelection } from "../drawing/selection";
import { segmentLabels, segmentList, segmentName, type SegmentLabels } from "../drawing/tooltip";
import { type InspectorSubject, inspectorSubject, type LeafPair } from "../inspector/subject";
import { mccsBySize } from "../tables/mccTable";
import { Button } from "../ui/Button";
import { EmptyState } from "../ui/EmptyState";
import { Link } from "../ui/Link";
import { MccSwatch } from "../ui/MccSwatch";
import { VirtualList } from "../ui/VirtualList";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";

const NOTHING_SELECTED = "Select a leaf, a branch, or an MCC to see details.";

export function Inspector() {
  const result = useWorkspace((state) => state.result);

  return (
    <div className="flex flex-col gap-4 px-4 py-4">
      {result === null ? (
        <EmptyState title={NOTHING_SELECTED} className="text-ink-muted" />
      ) : (
        <ResultInspector result={result} />
      )}
    </div>
  );
}

function ResultInspector({ result }: { result: RunResult }) {
  const { search, update } = useWorkspaceSearch();
  const { sessionId } = result;
  const selection = useMemo(() => selectionOf(search), [search]);
  const pair = currentData(usePairView(sessionId, search.pair, search.version, search.x));
  const arg = currentData(useArgView(selection.node?.side === "arg" ? sessionId : null, search.x));
  const constellation = useConstellation(selection.leaf === undefined ? null : sessionId).data;

  const subject = useMemo(
    () => inspectorSubject(selection, { pair, arg, constellation }),
    [selection, pair, arg, constellation],
  );

  const segments = useMemo(() => segmentLabels(result.request.trees), [result.request.trees]);

  const select = useCallback(
    (next: Selection) => {
      update((written) => withSelection(written, next));
    },
    [update],
  );

  return match(subject)
    .with({ kind: "none" }, ({ mccs }) => <NothingSelected mccs={mccs} onSelect={select} />)
    .with({ kind: "mcc" }, ({ mcc }) => <MccDetails mcc={mcc} onSelect={select} />)
    .with({ kind: "leaf" }, (leaf) => <LeafDetails subject={leaf} />)
    .with({ kind: "node" }, (node) => <NodeDetails subject={node} />)
    .with({ kind: "argNode" }, ({ node }) => <ArgNodeDetails node={node} segments={segments} />)
    .exhaustive();
}

function NothingSelected({ mccs, onSelect }: { mccs: readonly MccInfo[]; onSelect: (selection: Selection) => void }) {
  const items = useMemo(
    () => mccsBySize(mccs).map((mcc) => ({ id: mcc.index, text: mccTitle(mcc.index), mcc })),
    [mccs],
  );

  const choose = useCallback(
    (key: Key) => {
      onSelect({ mcc: Number(key) });
    },
    [onSelect],
  );

  return (
    <>
      <EmptyState title={NOTHING_SELECTED} className="text-ink-muted" />
      {items.length === 0 ? null : (
        <section aria-label="MCCs of this pair" className="flex flex-col gap-2">
          <h3 className="text-ink text-sm font-semibold">MCCs of this pair</h3>
          <VirtualList label="MCCs of this pair" items={items} onAction={choose}>
            {({ mcc }) => (
              <>
                <MccSwatch slot={mcc.slot} />
                <span className="w-16 shrink-0">{mccTitle(mcc.index)}</span>
                <span className="text-ink-muted w-20 shrink-0 text-right tabular-nums">{leafCount(mcc.size)}</span>
                <span className="font-condensed text-ink-muted min-w-0 truncate">{mcc.leaves[0] ?? ""}</span>
              </>
            )}
          </VirtualList>
        </section>
      )}
    </>
  );
}

function MccDetails({ mcc, onSelect }: { mcc: MccInfo; onSelect: (selection: Selection) => void }) {
  const { update } = useWorkspaceSearch();
  const leaves = useMemo(() => mcc.leaves.map((name) => ({ id: name, text: name })), [mcc]);

  const zoom = useCallback(() => {
    update((written) => mccInTanglegram(written, mcc.index));
    requestFocus({ kind: "mcc", mcc: mcc.index });
  }, [mcc.index, update]);

  const chooseLeaf = useCallback(
    (key: Key) => {
      onSelect({ leaf: String(key) });
    },
    [onSelect],
  );

  return (
    <Section title={mccTitle(mcc.index)} swatch={mcc.slot}>
      <Facts>
        <Fact term="Size">{leafCount(mcc.size)}</Fact>
        <Fact term="Imputed members">
          {mcc.imputedLeaves.length === 0 ? "none" : <NameList names={mcc.imputedLeaves} />}
        </Fact>
        <Fact term="Attachment">{mcc.ambiguousLeaves.length > 0 ? "ambiguous for some members" : "unambiguous"}</Fact>
      </Facts>
      <Button size="sm" icon={ZoomIcon} onPress={zoom} className="self-start">
        Zoom to MCC
      </Button>
      <VirtualList label={`Leaves of ${mccTitle(mcc.index)}`} items={leaves} onAction={chooseLeaf}>
        {({ text }) => <span className="font-condensed min-w-0 truncate">{text}</span>}
      </VirtualList>
    </Section>
  );
}

function LeafDetails({ subject }: { subject: Extract<InspectorSubject, { kind: "leaf" }> }) {
  const { name, copies, mcc, ambiguous, pairs } = subject;

  return (
    <Section title={name} condensed>
      <Facts>
        {copies.map(({ side, tree, node }) => (
          <Fact key={side} term={`Branch length in ${tree}`}>
            {formatBranchLength(node.branchLength)}
          </Fact>
        ))}
        {copies.map(({ side, tree, node }) => (
          <Fact key={`imputed-${side}`} term={`Imputed in ${tree}`}>
            {node.imputed ? "yes" : "no"}
          </Fact>
        ))}
        <Fact term="MCC">
          {mcc === undefined ? "none" : <MccValue mcc={mcc.index} size={mcc.size} slot={mcc.slot} />}
        </Fact>
        <Fact term="Attachment">{ambiguous ? "ambiguous" : "unambiguous"}</Fact>
      </Facts>
      {pairs.length === 0 ? null : <LeafPairs name={name} pairs={pairs} />}
    </Section>
  );
}

function LeafPairs({ name, pairs }: { name: string; pairs: readonly LeafPair[] }) {
  const { search } = useWorkspaceSearch();

  const focusLeaf = useCallback(() => {
    requestFocus({ kind: "leaf", name });
  }, [name]);

  return (
    <section aria-label="MCC in each pair" className="flex flex-col gap-1.5">
      <h3 className="text-ink text-sm font-semibold">MCC in each pair</h3>
      <ul className="flex flex-col gap-1 text-sm">
        {pairs.map(({ pair, labels: [a, b], cell }) => (
          <li key={pair} className="flex items-center justify-between gap-2">
            {cell === null ? (
              <span className="text-ink-muted">{`${a} and ${b}`}</span>
            ) : (
              <Link to="/" search={leafInPair(search, pair, name)} onPress={focusLeaf}>{`${a} and ${b}`}</Link>
            )}
            {cell === null ? (
              <span className="text-ink-muted">Not in this pair</span>
            ) : (
              <MccValue mcc={cell.mcc} size={cell.size} slot={cell.slot} />
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

function NodeDetails({ subject }: { subject: Extract<InspectorSubject, { kind: "node" }> }) {
  const { tree, node, cladeSize, mcc } = subject;

  return (
    <Section title={nodeName(node)} condensed>
      <Facts>
        <Fact term="Tree">{tree}</Fact>
        <Fact term="Clade size">{leafCount(cladeSize)}</Fact>
        <Fact term="Added by resolution">{node.added ? "yes" : "no"}</Fact>
        <Fact term="Branch length">{formatBranchLength(node.branchLength)}</Fact>
        <Fact term="MCC">
          {mcc === undefined ? "none" : <MccValue mcc={mcc.index} size={mcc.size} slot={mcc.slot} />}
        </Fact>
      </Facts>
    </Section>
  );
}

function ArgNodeDetails({ node, segments }: { node: ArgNodeView; segments: SegmentLabels }) {
  return (
    <Section title={node.label === "" ? "Unnamed node" : node.label} condensed>
      <Facts>
        <Fact term="Segments">{segmentList(node.segments, segments)}</Fact>
        {node.segments.map((segment) => (
          <Fact key={segment} term={`Branch length in ${segmentName(segment, segments)}`}>
            {formatBranchLength(node.tau[segment] ?? null)}
          </Fact>
        ))}
        <Fact term="Hybrid node">{node.hybrid ? "yes" : "no"}</Fact>
      </Facts>
    </Section>
  );
}

function Section({
  title,
  swatch,
  condensed = false,
  children,
}: {
  title: string;
  swatch?: number;
  condensed?: boolean;
  children: ReactNode;
}) {
  return (
    <section aria-label={title} className="flex flex-col gap-3">
      <h2 className="text-ink flex items-center gap-2 text-base font-semibold wrap-anywhere">
        {swatch === undefined ? null : <MccSwatch slot={swatch} />}
        <span className={condensed ? "font-condensed font-medium" : undefined}>{title}</span>
      </h2>
      {children}
    </section>
  );
}

function Facts({ children }: { children: ReactNode }) {
  return <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-sm">{children}</dl>;
}

function Fact({ term, children }: { term: string; children: ReactNode }) {
  return (
    <>
      <dt className="text-ink-muted">{term}</dt>
      <dd className="text-ink min-w-0 tabular-nums">{children}</dd>
    </>
  );
}

function MccValue({ mcc, size, slot }: { mcc: number; size: number; slot: number }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <MccSwatch slot={slot} />
      {mccSummary(mcc, size)}
    </span>
  );
}

function NameList({ names }: { names: readonly string[] }) {
  return (
    <ul className="font-condensed flex max-h-32 flex-col overflow-auto">
      {names.map((name) => (
        <li key={name} className="truncate">
          {name}
        </li>
      ))}
    </ul>
  );
}

function nodeName(node: DrawNode): string {
  return node.name === "" ? "Unnamed node" : node.name;
}
