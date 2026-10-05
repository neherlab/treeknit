import { selectPair, type WorkspaceSearch } from "../workspace/search";
import { withSelection } from "./selection";

export function leafInPair(search: WorkspaceSearch, pair: number, leaf: string): WorkspaceSearch {
  return { ...withSelection(selectPair(search, pair), { leaf }), view: "tanglegram" };
}

export function mccInTanglegram(search: WorkspaceSearch, mcc: number): WorkspaceSearch {
  return { ...withSelection(search, { mcc }), view: "tanglegram" };
}
