import type { AnalysisRequest, LinkEntry, LinkSource, TreeAddress } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";
import { match } from "ts-pattern";

import type { AnalysisClient } from "../analysis/client";
import { type SearchRecord, VIEW_KEYS } from "../workspace/search";
import { queryRecord } from "../workspace/searchQuery";
import { selectRequest, selectStale, type WorkspaceData, type WorkspaceStore } from "../workspace/store";
import type { TreeSource } from "../workspace/treeSource";

export function describedLink(state: WorkspaceData): LinkSource | null {
  const addresses = state.trees.map(({ source }) => treeAddress(source));

  if (state.result === null) {
    return { request: selectRequest(state), addresses, run: false };
  }

  return selectStale(state) ? null : { request: state.result.request, addresses, run: true };
}

export function shownLink(state: WorkspaceData): LinkSource {
  const described = describedLink(state);

  if (described !== null) {
    return described;
  }

  const request: AnalysisRequest = state.result?.request ?? selectRequest(state);

  return { request, addresses: request.trees.map(() => null), run: state.result !== null };
}

export function treeAddress(source: TreeSource): TreeAddress | null {
  return match(source)
    .with({ kind: "example" }, ({ id, name }): TreeAddress | null =>
      id === undefined ? null : { kind: "example", id, file: name },
    )
    .with({ kind: "url" }, ({ url }): TreeAddress => ({ kind: "url", url }))
    .with({ kind: "data" }, (): TreeAddress => ({ kind: "data" }))
    .with({ kind: "file" }, { kind: "paste" }, { kind: "session" }, { kind: "message" }, () => null)
    .exhaustive();
}

export function linkSearch(previous: Readonly<SearchRecord>, pairs: readonly LinkEntry[] | null): SearchRecord {
  const view = Object.fromEntries(
    Object.entries(previous).filter(([key]) => VIEW_KEYS.some((viewKey) => viewKey === key)),
  );

  return { ...linkRecord(pairs ?? []), ...view };
}

export function linkRecord(pairs: readonly LinkEntry[]): SearchRecord {
  return queryRecord(pairs.map(({ key, value }) => ({ key, value: value === "" ? true : value })));
}

export class AddressBarSync {
  readonly #store: WorkspaceStore;
  readonly #client: Pick<AnalysisClient, "launchPairs">;
  readonly #page: AddressBarPage;
  #written: LinkSource | null = null;
  #sequence = 0;
  #stop: (() => void) | null = null;

  constructor(store: WorkspaceStore, client: Pick<AnalysisClient, "launchPairs">, page: AddressBarPage) {
    this.#store = store;
    this.#client = client;
    this.#page = page;
  }

  start(): void {
    if (this.#stop !== null) {
      return;
    }

    this.#stop = this.#store.subscribe((state, previous) => {
      if (state.trees !== previous.trees || state.settings !== previous.settings || state.result !== previous.result) {
        void this.refresh(false);
      }
    });

    void this.refresh(true);
  }

  stop(): void {
    this.#stop?.();
    this.#stop = null;
  }

  async refresh(force: boolean): Promise<void> {
    const link = describedLink(this.#store.getState());

    if (link === null || !this.#page.onWorkspace() || (!force && isDeepEqual(link, this.#written))) {
      return;
    }

    this.#sequence += 1;

    const sequence = this.#sequence;
    const pairs = await this.#client.launchPairs(link);

    if (sequence === this.#sequence) {
      this.#written = link;
      await this.#page.write(pairs ?? null);
    }
  }
}

export interface AddressBarPage {
  onWorkspace: () => boolean;
  write: (pairs: readonly LinkEntry[] | null) => Promise<void>;
}
