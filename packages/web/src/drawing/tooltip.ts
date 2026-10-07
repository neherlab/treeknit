import type { ArgView, PairView } from "@neherlab/treeknit-wasm";
import { LEGEND_LABELS } from "@neherlab/treeknit-wasm/variants";
import { match } from "ts-pattern";

import { formatBranchLength, leafCount, mccSummary } from "./format";
import { itemAt } from "./lookup";
import type { ArgTarget, PairTarget } from "./selection";

export type SegmentLabels = readonly [string, string];

export function segmentLabels(trees: readonly { label: string }[]): SegmentLabels {
  const [first, second] = trees;

  return [first?.label ?? "A", second?.label ?? "B"];
}

export function pairTooltip(view: PairView, target: PairTarget): string[] {
  return match(target)
    .returnType<string[]>()
    .with({ kind: "link" }, ({ link: index }) => {
      const link = view.links[index];
      const leaf = link === undefined ? undefined : view.left.nodes[link.left];

      return leaf === undefined || link === undefined ? [] : [leaf.name, mccLine(view, link.mcc)];
    })
    .with({ kind: "ribbon" }, ({ block: index }) => {
      const block = view.blocks[index];

      return block === undefined ? [] : [mccLine(view, block.mcc), blockLine(block.left)];
    })
    .with({ kind: "node" }, ({ side, node: index }) => {
      const node = view[side].nodes[index];

      return node === undefined
        ? []
        : [
            node.name === "" ? "Unnamed node" : node.name,
            node.mcc === null ? LEGEND_LABELS.noMcc : mccLine(view, node.mcc),
            `Branch length: ${formatBranchLength(node.branchLength)}`,
            ...(node.meanLength === null
              ? []
              : [`Drawn at the mean length of the trees: ${formatBranchLength(node.meanLength)}`]),
            ...(node.mccBreak ? [LEGEND_LABELS.reassortmentBranch] : []),
            ...(node.added ? [LEGEND_LABELS.addedNode] : []),
            ...(node.imputed ? [LEGEND_LABELS.imputedLeaf] : []),
          ];
    })
    .exhaustive();
}

export function argTooltip(view: ArgView, target: ArgTarget, segments: SegmentLabels): string[] {
  if (target.kind === "edge") {
    const edge = view.edges[target.edge];

    return edge === undefined
      ? []
      : [segmentList(edge.segments, segments), ...(edge.reticulation ? ["Reticulation edge"] : [])];
  }

  const node = view.nodes[target.node];

  if (node === undefined) {
    return [];
  }

  return [
    node.label === "" ? "Unnamed node" : node.label,
    segmentList(node.segments, segments),
    ...node.segments.map(
      (segment) => `Branch length in ${segmentName(segment, segments)}: ${tauOf(node.tau, segment)}`,
    ),
    ...(node.hybrid ? [LEGEND_LABELS.reassortment] : []),
  ];
}

export function segmentName(segment: number, segments: SegmentLabels): string {
  return segments[segment] ?? `segment ${String(segment + 1)}`;
}

export function segmentList(present: readonly number[], segments: SegmentLabels): string {
  return present.length > 1
    ? LEGEND_LABELS.bothSegments
    : present.map((segment) => `${LEGEND_LABELS.segmentA} ${segmentName(segment, segments)}`).join("");
}

function tauOf(tau: readonly (number | null)[], segment: number): string {
  return formatBranchLength(itemAt(tau, segment, "segment"));
}

function mccLine(view: PairView, mcc: number): string {
  return mccSummary(mcc, itemAt(view.mccs, mcc, "MCC").size);
}

function blockLine([first, last]: [number, number]): string {
  return `${leafCount(Math.abs(last - first) + 1)} in this block`;
}
