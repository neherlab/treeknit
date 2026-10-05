import type { AuspiceNode, ObservedMutations } from "auspice/src/state";

export declare function getParentBeyondPolytomy(
  node: AuspiceNode,
  distanceMeasure: string,
  observedMutations: ObservedMutations | undefined,
): AuspiceNode;
