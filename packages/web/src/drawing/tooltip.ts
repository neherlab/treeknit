import type { ArgView, PairView } from "@neherlab/treeknit-wasm";

import { formatBranchLength, leafCount, mccSummary } from "./format";
import { itemAt } from "./lookup";
import type { ArgTarget, PairTarget } from "./selection";

export type SegmentLabels = readonly [string, string];

export function segmentLabels(trees: readonly { label: string }[]): SegmentLabels {
  const [first, second] = trees;

  return [first?.label ?? "A", second?.label ?? "B"];
}

export function pairTooltip(view: PairView, target: PairTarget): string[] {
  if (target.kind === "link") {
    const link = view.links[target.link];
    const leaf = link === undefined ? undefined : view.left.nodes[link.left];

    return leaf === undefined || link === undefined ? [] : [leaf.name, mccLine(view, link.mcc)];
  }

  if (target.kind === "ribbon") {
    const block = view.blocks[target.block];

    return block === undefined ? [] : [mccLine(view, block.mcc), blockLine(block.left)];
  }

  const node = view[target.side].nodes[target.node];

  if (node === undefined) {
    return [];
  }

  return [
    node.name === "" ? "Unnamed node" : node.name,
    node.mcc === null ? "No MCC" : mccLine(view, node.mcc),
    `Branch length: ${formatBranchLength(node.branchLength)}`,
    ...(node.meanLength === null
      ? []
      : [`Drawn at the mean length of the trees: ${formatBranchLength(node.meanLength)}`]),
    ...(node.mccBreak ? ["Reassortment branch"] : []),
    ...(node.added ? ["Added by resolution or imputation"] : []),
    ...(node.imputed ? ["Imputed leaf"] : []),
  ];
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
    ...(node.hybrid ? ["Hybrid node"] : []),
  ];
}

export function segmentName(segment: number, segments: SegmentLabels): string {
  return segments[segment] ?? `segment ${String(segment + 1)}`;
}

export function segmentList(present: readonly number[], segments: SegmentLabels): string {
  const names = present.map((segment) => segmentName(segment, segments));

  return names.length > 1 ? `Segments ${names.join(" and ")}` : `Segment ${names.join("")}`;
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
