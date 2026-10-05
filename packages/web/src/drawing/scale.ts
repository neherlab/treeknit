import type { Scale } from "@neherlab/treeknit-wasm";

import type { DrawingNotice } from "./DrawingPanel";

export const SCALE_LABELS: Record<Scale, string> = { div: "Divergence", depth: "Cladogram" };

export type ScaledDrawing = "pair" | "arg";

const CLADOGRAM_NOTICES: Record<ScaledDrawing, DrawingNotice> = {
  pair: { tone: "info", text: `${SCALE_LABELS.depth}: a tree of this pair has no branch lengths.` },
  arg: { tone: "info", text: `${SCALE_LABELS.depth}: neither tree has branch lengths.` },
};

export function shownScaleNotice(
  requested: Scale,
  shown: Scale | undefined,
  drawing: ScaledDrawing,
): DrawingNotice | undefined {
  return requested === "div" && shown === "depth" ? CLADOGRAM_NOTICES[drawing] : undefined;
}
