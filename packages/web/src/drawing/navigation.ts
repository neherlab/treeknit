import { type PairRef, selectPair, type WrittenSearch } from "../workspace/search";
import { withSelection } from "./selection";

export function leafInPair(
  search: WrittenSearch,
  pairLabels: readonly PairRef[],
  pair: number,
  leaf: string,
): WrittenSearch {
  return { ...withSelection(selectPair(search, pairLabels, pair), { leaf }), view: "tanglegram" };
}

export function mccInTanglegram(search: WrittenSearch, mcc: number): WrittenSearch {
  return { ...withSelection(search, { mcc }), view: "tanglegram" };
}
