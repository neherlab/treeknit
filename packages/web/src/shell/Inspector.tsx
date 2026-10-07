import type { ArgNodeView, DrawNode, MccInfo } from "@neherlab/treeknit-wasm";
import { createContext, type ReactNode, use, useCallback, useMemo } from "react";
import type { Key, PressEvent } from "react-aria-components";
import { match } from "ts-pattern";
import BackIcon from "~icons/lucide/arrow-left";
import ZoomIcon from "~icons/lucide/scan-search";
import ClearIcon from "~icons/lucide/x";

import { currentData, useArgView, useConstellation, usePairView } from "../analysis/queries";
import { requestFocus } from "../drawing/focus";
import { formatBranchLength, leafCount, mccSummary, mccTitle } from "../drawing/format";
import { leafInPair, mccInTanglegram } from "../drawing/navigation";
import { selectionOf, type Selection } from "../drawing/selection";
import { segmentLabels, segmentList, segmentName, type SegmentLabels } from "../drawing/tooltip";
import { useDrawingSearch } from "../drawing/useDrawingSearch";
import { counted } from "../format/count";
import { NONE, yesNo } from "../format/words";
import { inspectorParent, type InspectorSubject, inspectorSubject, type LeafPair } from "../inspector/subject";
import { hasAmbiguousAttachment, mccsBySize } from "../tables/mccTable";
import { Button } from "../ui/Button";
import { EmptyState } from "../ui/EmptyState";
import { IconButton } from "../ui/IconButton";
import { Link } from "../ui/Link";
import { MccSwatch } from "../ui/MccSwatch";
import { useEscapeKey } from "../ui/useEscapeKey";
import { VirtualList } from "../ui/VirtualList";
import { useWorkspace } from "../workspace/context";
import type { WrittenSearch } from "../workspace/search";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";

const NOTHING_SELECTED = "Select a leaf, a branch, or an MCC to see details.";

const LIST_STYLE = "min-h-48 flex-1";

const SubjectNavigation = createContext<SubjectNavigationValue>({
  parent: null,
  onSelect: () => undefined,
  onClear: () => undefined,
});

export function Inspector() {
  const result = useWorkspace((state) => state.result);

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 px-4 py-4">
      {result === null ? (
        <EmptyState title={NOTHING_SELECTED} className="text-ink-muted" />
      ) : (
        <ResultInspector result={result} />
      )}
    </div>
  );
}

function ResultInspector({ result }: { result: RunResult }) {
  const { search, select, clear, clearOnEscape } = useDrawingSearch();
  const escapeProps = useEscapeKey(clearOnEscape);
  const { sessionId } = result;
  const selection = useMemo(() => selectionOf(search), [search]);
  const pair = currentData(usePairView(sessionId, search.pair, search.version, search.scale));
  const arg = currentData(useArgView(selection.node?.side === "arg" ? sessionId : null, search.scale));
  const constellation = useConstellation(selection.leaf === undefined ? null : sessionId).data;

  const subject = useMemo(
    () => inspectorSubject(selection, { pair, arg, constellation }),
    [selection, pair, arg, constellation],
  );

  const segments = useMemo(() => segmentLabels(result.request.trees), [result.request.trees]);

  const parent = inspectorParent(subject);
  const navigation = useMemo(() => ({ parent, onSelect: select, onClear: clear }), [parent, select, clear]);

  const details = match(subject)
    .with({ kind: "none" }, ({ mccs }) => <NothingSelected mccs={mccs} onSelect={select} />)
    .with({ kind: "mcc" }, ({ mcc }) => <MccDetails mcc={mcc} onSelect={select} />)
    .with({ kind: "leaf" }, (leaf) => <LeafDetails subject={leaf} />)
    .with({ kind: "node" }, (node) => <NodeDetails subject={node} />)
    .with({ kind: "argNode" }, ({ node }) => <ArgNodeDetails node={node} segments={segments} />)
    .exhaustive();

  return (
    <div {...escapeProps} className="flex min-h-0 flex-1 flex-col gap-4">
      <SubjectNavigation value={navigation}>{details}</SubjectNavigation>
    </div>
  );
}

interface SubjectNavigationValue {
  parent: Selection | null;
  onSelect: (selection: Selection) => void;
  onClear: () => void;
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

  if (items.length === 0) {
    return <EmptyState title={NOTHING_SELECTED} className="text-ink-muted" />;
  }

  return (
    <section aria-labelledby="inspector-mccs" className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex flex-col gap-0.5">
        <h2 id="inspector-mccs" className="text-ink text-base font-semibold">
          {counted(items.length, "MCC", "MCCs")}
        </h2>
        <p className="text-ink-muted text-sm">Select an MCC, a leaf, or a branch for details.</p>
      </div>
      <VirtualList label="MCCs of this pair" items={items} onAction={choose} className={LIST_STYLE}>
        {({ mcc }) => (
          <>
            <MccSwatch slot={mcc.slot} />
            <span className="w-20 shrink-0 whitespace-nowrap">{mccTitle(mcc.index)}</span>
            <span className="text-ink-muted w-20 shrink-0 text-right tabular-nums">{leafCount(mcc.size)}</span>
            <span className="text-ink-muted min-w-0 truncate">{mcc.leaves[0] ?? ""}</span>
          </>
        )}
      </VirtualList>
    </section>
  );
}

function MccDetails({ mcc, onSelect }: { mcc: MccInfo; onSelect: (selection: Selection) => void }) {
  const { search, update } = useWorkspaceSearch();
  const inAuspice = search.view === "auspice";
  const leaves = useMemo(() => mcc.leaves.map((name) => ({ id: name, text: name })), [mcc]);

  const zoom = useCallback(() => {
    if (!inAuspice) {
      update((written) => mccInTanglegram(written, mcc.index));
    }

    requestFocus({ kind: "mcc", mcc: mcc.index });
  }, [inAuspice, mcc.index, update]);

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
          {mcc.imputedLeaves.length === 0 ? NONE : <NameList names={mcc.imputedLeaves} />}
        </Fact>
        <Fact term="Attachment">{hasAmbiguousAttachment(mcc) ? "ambiguous for some members" : "unambiguous"}</Fact>
      </Facts>
      <Button size="sm" icon={ZoomIcon} onPress={zoom} className="self-start">
        Zoom to MCC
      </Button>
      <VirtualList
        label={`Leaves of ${mccTitle(mcc.index)}`}
        items={leaves}
        onAction={chooseLeaf}
        className={LIST_STYLE}
      >
        {({ text }) => <span className="min-w-0 truncate">{text}</span>}
      </VirtualList>
    </Section>
  );
}

function LeafDetails({ subject }: { subject: Extract<InspectorSubject, { kind: "leaf" }> }) {
  const { name, copies, mcc, ambiguous, pairs } = subject;

  return (
    <Section title={name}>
      <Facts>
        {copies.map(({ side, tree, node }) => (
          <Fact key={side} term={`Branch length in ${tree}`}>
            {formatBranchLength(node.branchLength)}
          </Fact>
        ))}
        {copies.map(({ side, tree, node }) => (
          <Fact key={`imputed-${side}`} term={`Imputed in ${tree}`}>
            {yesNo(node.imputed)}
          </Fact>
        ))}
        <Fact term="MCC">
          {mcc === undefined ? NONE : <MccValue mcc={mcc.index} size={mcc.size} slot={mcc.slot} />}
        </Fact>
        <Fact term="Attachment">{ambiguous ? "ambiguous" : "unambiguous"}</Fact>
      </Facts>
      {pairs.length === 0 ? null : <LeafPairs name={name} pairs={pairs} />}
    </Section>
  );
}

function LeafPairs({ name, pairs }: { name: string; pairs: readonly LeafPair[] }) {
  return (
    <section aria-label="MCC in each pair" className="flex flex-col gap-1.5">
      <h3 className="text-ink text-sm font-semibold">MCC in each pair</h3>
      <ul className="flex flex-col gap-1 text-sm">
        {pairs.map(({ pair, labels: [a, b], cell }) => (
          <li key={pair} className="flex items-center justify-between gap-2">
            {cell === null ? (
              <span className="text-ink-muted">{`${a} and ${b}`}</span>
            ) : (
              <LeafPairLink name={name} pair={pair} label={`${a} and ${b}`} />
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

function LeafPairLink({ name, pair, label }: { name: string; pair: number; label: string }) {
  const { pairLabels } = useWorkspaceSearch();

  const target = useCallback(
    (search: WrittenSearch): WrittenSearch => leafInPair(search, pairLabels, pair, name),
    [pairLabels, pair, name],
  );

  const focusLeaf = useCallback(
    ({ ctrlKey, metaKey, shiftKey }: PressEvent) => {
      if (!ctrlKey && !metaKey && !shiftKey) {
        requestFocus({ kind: "leaf", name }, pair);
      }
    },
    [name, pair],
  );

  return (
    <Link from="/" to="/" search={target} onPress={focusLeaf}>
      {label}
    </Link>
  );
}

function NodeDetails({ subject }: { subject: Extract<InspectorSubject, { kind: "node" }> }) {
  const { tree, node, mcc } = subject;

  return (
    <Section title={nodeName(node)}>
      <Facts>
        <Fact term="Tree">{tree}</Fact>
        <Fact term="Clade size">{leafCount(node.cladeSize)}</Fact>
        <Fact term="Added by resolution or imputation">{yesNo(node.added)}</Fact>
        <Fact term="Branch length">{formatBranchLength(node.branchLength)}</Fact>
        <Fact term="MCC">
          {mcc === undefined ? NONE : <MccValue mcc={mcc.index} size={mcc.size} slot={mcc.slot} />}
        </Fact>
      </Facts>
    </Section>
  );
}

function ArgNodeDetails({ node, segments }: { node: ArgNodeView; segments: SegmentLabels }) {
  return (
    <Section title={node.label === "" ? "Unnamed node" : node.label}>
      <Facts>
        <Fact term="Segments">{segmentList(node.segments, segments)}</Fact>
        {node.segments.map((segment) => (
          <Fact key={segment} term={`Branch length in ${segmentName(segment, segments)}`}>
            {formatBranchLength(node.tau[segment] ?? null)}
          </Fact>
        ))}
        <Fact term="Hybrid node">{yesNo(node.hybrid)}</Fact>
      </Facts>
    </Section>
  );
}

function Section({ title, swatch, children }: { title: string; swatch?: number; children: ReactNode }) {
  const { parent, onSelect, onClear } = use(SubjectNavigation);

  const stepUp = useCallback(() => {
    if (parent !== null) {
      onSelect(parent);
    }
  }, [parent, onSelect]);

  return (
    <section aria-label={title} className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-start gap-1">
        {parent?.mcc === undefined ? null : (
          <IconButton label={`Show ${mccTitle(parent.mcc)}`} icon={BackIcon} size="sm" onPress={stepUp} />
        )}
        <h2 className="text-ink flex min-w-0 flex-1 items-center gap-2 pt-0.5 text-base font-semibold wrap-anywhere">
          {swatch === undefined ? null : <MccSwatch slot={swatch} />}
          <span>{title}</span>
        </h2>
        <IconButton label="Clear selection" icon={ClearIcon} size="sm" onPress={onClear} />
      </div>
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
    <ul className="flex max-h-32 flex-col overflow-auto">
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
