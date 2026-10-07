import type {
  AnalysisRequest,
  ExampleInfo,
  Launch,
  LaunchInput,
  LaunchParse,
  LinkLocation,
  MessageSource,
  Settings,
} from "@neherlab/treeknit-wasm";
import { getErrorMessage } from "react-error-boundary";
import { doNothing } from "remeda";
import { match } from "ts-pattern";
import { createStore } from "zustand/vanilla";

import type { AnalysisClient } from "../analysis/client";
import type { ExampleTree } from "../analysis/example";
import type { StatelessArgs, StatelessResult } from "../analysis/protocol";
import { VIEW_KEYS } from "../workspace/search";
import type { QueryEntry } from "../workspace/searchQuery";
import type { ReadableStore, WorkspaceStore } from "../workspace/store";
import { SESSION_SOURCE, type TreeSource, urlSource } from "../workspace/treeSource";
import { download, type DownloadLimits, downloadMessage, type FetchFile } from "./download";
import { ignoredKeyNote, launchEntries, namesView } from "./entries";

const NO_LAUNCH: LaunchState = { status: { kind: "none" }, notes: [] };

export const IDLE_LAUNCH: LaunchControl = {
  state: createStore<LaunchState>()(() => NO_LAUNCH),
  retry: doNothing(),
  dismissStatus: doNothing(),
  dismissNotes: doNothing(),
};

export class LinkLaunch implements LaunchControl {
  readonly #environment: LaunchEnvironment;
  readonly #entries: readonly QueryEntry[];
  readonly #state = createStore<LaunchState>()(() => NO_LAUNCH);
  readonly #settled = Promise.withResolvers<undefined>();
  #launch: Launch | null = null;
  #store: WorkspaceStore | null = null;

  constructor(environment: LaunchEnvironment, entries: readonly QueryEntry[]) {
    this.#environment = environment;
    this.#entries = entries;
  }

  async start(store: WorkspaceStore): Promise<void> {
    this.#store = store;

    if (this.#entries.length === 0) {
      this.#settled.resolve(undefined);

      return;
    }

    try {
      const parsed = await this.#environment.client.parseLaunch(launchEntries(this.#entries), [...VIEW_KEYS]);

      this.#launch = parsed.launch;
      this.#update({ notes: launchNotes(parsed) });

      if (parsed.launch === null) {
        this.#settled.resolve(undefined);
      }
    } catch (cause) {
      this.#update({
        status: {
          kind: "failed",
          problems: [`The link could not be read: ${getErrorMessage(cause) ?? String(cause)}`],
        },
      });

      return;
    }

    await this.#load();
  }

  get settled(): Promise<undefined> {
    return this.#settled.promise;
  }

  get state(): ReadableStore<LaunchState> {
    return this.#state;
  }

  retry(): void {
    void this.#load();
  }

  dismissStatus(): void {
    if (this.#state.getState().status.kind === "failed") {
      this.#update({ status: NO_LAUNCH.status });
      this.#settled.resolve(undefined);
    }
  }

  dismissNotes(): void {
    this.#update({ notes: [] });
  }

  async #load(): Promise<void> {
    const launch = this.#launch;
    const store = this.#store;

    if (launch === null || store === null || this.#state.getState().status.kind === "loading") {
      return;
    }

    this.#update({ status: { kind: "loading", message: loadingMessage(launch.input) } });

    let loaded: LoadedLaunch;

    try {
      loaded = await loadLaunch(launch, store.getState().defaults, this.#environment);
    } catch (cause) {
      const problems = cause instanceof LaunchLoadError ? cause.problems : [getErrorMessage(cause) ?? String(cause)];

      this.#update({ status: { kind: "failed", problems } });

      return;
    }

    store.getState().openLink(loaded.request, loaded.sources);
    this.#update({ status: { kind: "loaded" } });
    this.#settled.resolve(undefined);

    if (launch.run || loaded.run) {
      await this.#environment.run(namesView(this.#entries));
    }
  }

  #update(change: Partial<LaunchState>): void {
    this.#state.setState(change);
  }
}

export interface LaunchControl {
  readonly state: ReadableStore<LaunchState>;
  retry(): void;
  dismissStatus(): void;
  dismissNotes(): void;
}

export interface LaunchState {
  status: LaunchStatus;
  notes: readonly string[];
}

export type LaunchStatus =
  | { kind: "none" }
  | { kind: "loading"; message: string }
  | { kind: "failed"; problems: readonly string[] }
  | { kind: "loaded" };

export function launchClient(client: AnalysisClient): LaunchClient {
  return {
    async parseLaunch(...args) {
      return client.stateless(async (api) => api.parseLaunch(...args));
    },
    async decodeTreeBytes(...args) {
      return client.stateless(async (api) => api.decodeTreeBytes(...args));
    },
    async examples() {
      return client.stateless(async (api) => api.examples());
    },
    async readSession(...args) {
      return client.stateless(async (api) => api.readSession(...args));
    },
    async applySettings(...args) {
      return client.stateless(async (api) => api.applySettings(...args));
    },
    async linkLimits() {
      return client.stateless(async (api) => api.linkLimits());
    },
  };
}

export interface LaunchEnvironment {
  client: LaunchClient;
  fetchFile: FetchFile;
  loadExample: (example: ExampleInfo) => Promise<ExampleTree[]>;
  receiveSession: (source: MessageSource) => Promise<ReceivedSession>;
  run: (keepView: boolean) => Promise<void>;
}

export interface LaunchClient {
  parseLaunch(...args: StatelessArgs<"parseLaunch">): StatelessResult<"parseLaunch">;
  decodeTreeBytes(...args: StatelessArgs<"decodeTreeBytes">): StatelessResult<"decodeTreeBytes">;
  examples(): StatelessResult<"examples">;
  readSession(...args: StatelessArgs<"readSession">): StatelessResult<"readSession">;
  applySettings(...args: StatelessArgs<"applySettings">): StatelessResult<"applySettings">;
  linkLimits(): StatelessResult<"linkLimits">;
}

export interface ReceivedSession {
  text: string;
  origin: string;
  run: boolean;
}

export interface LoadedLaunch {
  request: AnalysisRequest;
  sources: readonly TreeSource[];
  run: boolean;
}

export class LaunchLoadError extends Error {
  readonly problems: readonly string[];

  constructor(problems: readonly string[]) {
    super(problems.join("\n"));
    this.name = "LaunchLoadError";
    this.problems = problems;
  }
}

export function launchNotes(parsed: LaunchParse): string[] {
  return [
    ...parsed.errors.map(({ message }) => message),
    ...parsed.ignored.flatMap((key) => {
      const note = ignoredKeyNote(key);

      return note === null ? [] : [note];
    }),
  ];
}

export function loadingMessage(input: LaunchInput): string {
  return match(input)
    .with({ kind: "example" }, ({ id }) => `Loading the example ${id}`)
    .with({ kind: "trees" }, ({ trees }) => {
      const hosts = new Set(trees.flatMap(({ location }) => (location.kind === "url" ? [location.host] : [])));
      const count = `${String(trees.length)} trees`;

      if (hosts.size === 0) {
        return `Loading ${count} from the link`;
      }

      const [host] = hosts;

      return hosts.size === 1 && host !== undefined
        ? `Loading ${count} from ${host}`
        : `Loading ${count} from ${String(hosts.size)} sites`;
    })
    .with({ kind: "session" }, ({ location }) =>
      location.kind === "url"
        ? `Loading the session file from ${location.host}`
        : "Loading the session file from the link",
    )
    .with({ kind: "message" }, ({ source }) =>
      source === "opener"
        ? "Waiting for the session file from the page that opened TreeKnit"
        : "Waiting for the session file from the page that embeds TreeKnit",
    )
    .exhaustive();
}

export async function loadLaunch(
  launch: Launch,
  defaults: Settings,
  environment: LaunchEnvironment,
): Promise<LoadedLaunch> {
  const { client } = environment;
  const settings = async (base: Settings) => client.applySettings(base, launch.settings);

  return match(launch.input)
    .with({ kind: "example" }, async ({ id }) => {
      const example = (await client.examples()).find((candidate) => candidate.id === id);

      if (example === undefined) {
        throw new LaunchLoadError([`There is no example ${id}.`]);
      }

      const trees = await environment.loadExample(example);

      return {
        request: { trees: trees.map(({ label, newick }) => ({ label, newick })), settings: await settings(defaults) },
        sources: trees.map(({ fileName }): TreeSource => ({ kind: "example", name: fileName, id })),
        run: false,
      };
    })
    .with({ kind: "trees" }, async ({ trees }) => {
      const limits = await client.linkLimits();

      const texts = await Promise.allSettled(
        trees.map(async ({ location }) => readLocation(location, limits, environment)),
      );

      const problems = texts.flatMap((text) =>
        text.status === "rejected" ? [getErrorMessage(text.reason) ?? String(text.reason)] : [],
      );

      if (problems.length > 0) {
        throw new LaunchLoadError(problems);
      }

      return {
        request: {
          trees: trees.map(({ label }, index) => ({ label, newick: fulfilled(texts[index]) })),
          settings: await settings(defaults),
        },
        sources: trees.map(({ location }): TreeSource =>
          location.kind === "url"
            ? urlSource(location.url, { host: location.host, fileName: location.fileName })
            : { kind: "data" },
        ),
        run: false,
      };
    })
    .with({ kind: "session" }, async ({ location }) => {
      const limits = await client.linkLimits();

      const text = await readLocation(location, limits, environment).catch((cause: unknown) => {
        throw new LaunchLoadError([getErrorMessage(cause) ?? String(cause)]);
      });

      return { ...(await openSession(text, SESSION_SOURCE, settings, defaults, client)), run: false };
    })
    .with({ kind: "message" }, async ({ source }) => {
      const received = await environment.receiveSession(source).catch((cause: unknown) => {
        throw new LaunchLoadError([getErrorMessage(cause) ?? String(cause)]);
      });

      const opened = await openSession(
        received.text,
        { kind: "message", origin: received.origin },
        settings,
        defaults,
        client,
      );

      return { ...opened, run: received.run };
    })
    .exhaustive();
}

async function readLocation(
  location: LinkLocation,
  limits: DownloadLimits,
  environment: LaunchEnvironment,
): Promise<string> {
  if (location.kind === "data") {
    return location.text;
  }

  try {
    const bytes = await download(location.fetch, limits, environment.fetchFile);

    return await environment.client.decodeTreeBytes(bytes);
  } catch (cause) {
    throw new Error(downloadMessage(location, cause, limits), { cause });
  }
}

async function openSession(
  text: string,
  source: TreeSource,
  settings: (base: Settings) => Promise<Settings>,
  defaults: Settings,
  client: LaunchClient,
): Promise<Omit<LoadedLaunch, "run">> {
  let request: AnalysisRequest;

  try {
    request = await client.readSession(text);
  } catch (cause) {
    throw new LaunchLoadError([`The session file could not be opened: ${getErrorMessage(cause) ?? String(cause)}`]);
  }

  return {
    request: { trees: request.trees, settings: await settings(request.settings ?? defaults) },
    sources: request.trees.map(() => source),
  };
}

function fulfilled(result: PromiseSettledResult<string> | undefined): string {
  if (result?.status !== "fulfilled") {
    throw new LaunchLoadError(["A tree of the link did not load."]);
  }

  return result.value;
}
