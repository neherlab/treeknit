import type { AuspiceMetadataState, AuspiceNode, AuspicePublication } from "auspice/src/state";
import type { TFunction } from "i18next";
import type { Dispatch } from "redux";

export declare function SVG(
  dispatch: Dispatch,
  t: TFunction,
  metadata: AuspiceMetadataState,
  nodes: readonly AuspiceNode[] | null,
  visibility: readonly number[] | null,
  filePrefix: string,
  panelsInDOM: readonly string[],
  panelLayout: string,
  publications: readonly AuspicePublication[],
  caption?: readonly string[],
): void;
