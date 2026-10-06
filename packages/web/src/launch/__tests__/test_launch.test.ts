import type {
  AnalysisRequest,
  ExampleInfo,
  Launch,
  LaunchInput,
  LaunchParse,
  LinkLimits,
  Settings,
  SettingsPatch,
} from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { createWorkspaceStore, selectRequest, type WorkspaceStore } from "../../workspace/store";
import type { FetchFile } from "../download";
import { ignoredKeyNote, linkEntries } from "../entries";
import { type LaunchClient, type LaunchEnvironment, LinkLaunch, loadingMessage } from "../LinkLaunch";

const DEFAULTS: Settings = {
  gamma: 2,
  seqLengths: null,
  nMcmcIt: 50,
  resolve: "matched",
  preResolve: false,
  rounds: 1,
  finalRound: true,
  likelihood: true,
  naive: false,
  seed: 1,
};

const NO_PATCH: SettingsPatch = {
  gamma: null,
  seqLengths: null,
  nMcmcIt: null,
  resolve: null,
  preResolve: null,
  rounds: null,
  finalRound: null,
  likelihood: null,
  naive: null,
  seed: null,
};

const LIMITS: LinkLimits = {
  fetchTimeoutSeconds: 60,
  maxDownloadBytes: 1024,
  maxLinkChars: 32_000,
  longLinkChars: 2000,
};

const HA = "((A,B),(C,(D,X)));";

const NA = "((A,(B,X)),(C,D));";

const SMALL: ExampleInfo = {
  id: "5-leaves",
  name: "5 leaves",
  group: "small",
  groupName: "Small",
  trees: [
    { file: "ha.nwk", label: "ha", path: null, newick: HA },
    { file: "na.nwk", label: "na", path: null, newick: NA },
  ],
};

const SESSION: AnalysisRequest = {
  trees: [
    { label: "seg4", newick: HA },
    { label: "seg6", newick: NA },
  ],
  settings: { ...DEFAULTS, gamma: 4, seed: 5 },
};

const SESSION_TEXT = "the session file";

const URL_TREES: Launch = {
  input: {
    kind: "trees",
    trees: [
      { label: "ha", location: { kind: "url", url: "https://x.org/ha.nwk", fetch: "https://x.org/ha.nwk" } },
      { label: "na", location: { kind: "url", url: "https://x.org/na.nwk", fetch: "https://x.org/na.nwk" } },
    ],
  },
  settings: NO_PATCH,
  run: false,
};

describe("a link launch", () => {
  test("opens an example with the settings of the link and runs it, keeping the view the link names", async () => {
    const launch: Launch = {
      input: { kind: "example", id: "5-leaves" },
      settings: { ...NO_PATCH, gamma: 3 },
      run: true,
    };

    const runs: boolean[] = [];

    const { store, linkLaunch } = setUp(
      parsed(launch),
      { run: recordRuns(runs) },
      "?example=5-leaves&gamma=3&run&view=mccs",
    );

    await linkLaunch.start(store);

    expect({
      request: selectRequest(store.getState()),
      sources: store.getState().trees.map(({ source }) => source),
      undo: store.getState().undo?.kind === "workspace" ? store.getState().undo : null,
      status: linkLaunch.getSnapshot().status,
      runs,
    }).toMatchObject({
      request: {
        trees: [
          { label: "ha", newick: HA },
          { label: "na", newick: NA },
        ],
        settings: { ...DEFAULTS, gamma: 3 },
      },
      sources: [
        { kind: "example", name: "ha.nwk", id: "5-leaves" },
        { kind: "example", name: "na.nwk", id: "5-leaves" },
      ],
      undo: { reason: "link" },
      status: { kind: "loaded" },
      runs: [true],
    });
  });

  test("runs without keeping the view when the link names none", async () => {
    const launch: Launch = { input: { kind: "example", id: "5-leaves" }, settings: NO_PATCH, run: true };
    const runs: boolean[] = [];
    const { store, linkLaunch } = setUp(parsed(launch), { run: recordRuns(runs) }, "?example=5-leaves&run");

    await linkLaunch.start(store);

    expect(runs).toStrictEqual([false]);
  });

  test("leaves the workspace untouched while a tree fails, and loads every tree on retry", async () => {
    const answers = [new Response("", { status: 404, statusText: "Not Found" }), new Response(NA)];

    const fetchFile: FetchFile = (url) =>
      Promise.resolve(url.endsWith("ha.nwk") ? new Response(HA) : (answers.shift() ?? new Response(NA)));

    const { store, linkLaunch } = setUp(parsed(URL_TREES), { fetchFile });

    await linkLaunch.start(store);

    const failed = { status: linkLaunch.getSnapshot().status, trees: store.getState().trees.length };

    linkLaunch.retry();
    await settled(linkLaunch);

    expect({
      failed,
      retried: linkLaunch.getSnapshot().status,
      request: selectRequest(store.getState()).trees,
      sources: store.getState().trees.map(({ source }) => source),
    }).toStrictEqual({
      failed: { status: { kind: "failed", problems: ["Could not read x.org/na.nwk: 404 Not Found"] }, trees: 0 },
      retried: { kind: "loaded" },
      request: [
        { label: "ha", newick: HA },
        { label: "na", newick: NA },
      ],
      sources: [
        { kind: "url", url: "https://x.org/ha.nwk" },
        { kind: "url", url: "https://x.org/na.nwk" },
      ],
    });
  });

  test.each<[string, FetchFile, LinkLimits, string]>([
    [
      "a refused read, as for a host without CORS",
      () => Promise.reject(new TypeError("Failed to fetch")),
      LIMITS,
      "Could not read x.org/ha.nwk. The server may not allow other sites to read it (CORS), the address may be wrong, or the network is down. Download the file and drop it here instead.",
    ],
    [
      "a status without a reason phrase, as HTTP/2 answers",
      () => Promise.resolve(new Response("", { status: 403 })),
      LIMITS,
      "Could not read x.org/ha.nwk: HTTP status 403",
    ],
    [
      "a file above the size limit",
      () => Promise.resolve(new Response("x".repeat(2048))),
      LIMITS,
      "Could not read x.org/ha.nwk: the file is larger than 0.0009765625 MiB.",
    ],
    [
      "a server that does not answer",
      (_url, init) =>
        new Promise((_resolve, reject) => {
          init.signal?.addEventListener("abort", () => {
            reject(new Error("aborted"));
          });
        }),
      { ...LIMITS, fetchTimeoutSeconds: 0.01 },
      "Could not read x.org/ha.nwk: no answer within 0.01 seconds.",
    ],
  ])("reports %s by its address", async (_case, fetchHa, limits, message) => {
    const fetchFile: FetchFile = (url, init) =>
      url.endsWith("ha.nwk") ? fetchHa(url, init) : Promise.resolve(new Response(NA));

    const { store, linkLaunch } = setUp(parsed(URL_TREES), { fetchFile, limits });

    await linkLaunch.start(store);

    expect(linkLaunch.getSnapshot().status).toStrictEqual({ kind: "failed", problems: [message] });
  });

  test("applies the settings of the link to the settings of the session file", async () => {
    const launch: Launch = {
      input: { kind: "session", location: { kind: "data", text: SESSION_TEXT } },
      settings: { ...NO_PATCH, seed: 7 },
      run: false,
    };

    const { store, linkLaunch } = setUp(parsed(launch), {});

    await linkLaunch.start(store);

    expect(selectRequest(store.getState())).toStrictEqual({ ...SESSION, settings: { ...DEFAULTS, gamma: 4, seed: 7 } });
  });

  test("keeps the workspace and makes no undo entry when the link describes the saved workspace", async () => {
    const launch: Launch = { input: { kind: "example", id: "5-leaves" }, settings: NO_PATCH, run: false };

    const saved: AnalysisRequest = {
      trees: [
        { label: "ha", newick: HA },
        { label: "na", newick: NA },
      ],
      settings: DEFAULTS,
    };

    const { store, linkLaunch } = setUp(parsed(launch), {}, "?example=5-leaves", saved);

    await linkLaunch.start(store);

    expect({ undo: store.getState().undo, source: store.getState().trees[0]?.source }).toStrictEqual({
      undo: null,
      source: { kind: "example", name: "ha.nwk", id: "5-leaves" },
    });
  });

  test("lists the errors of the link and the ignored keys that have a hint", async () => {
    const parse: LaunchParse = {
      launch: null,
      errors: [
        {
          field: { kind: "linkKey", key: "gamma" },
          message: 'gamma must be a number, got "abc"',
          line: null,
          column: null,
        },
      ],
      ignored: [
        { key: "gama", suggestion: "gamma", afterLocationQuery: false },
        { key: "utm_source", suggestion: null, afterLocationQuery: false },
        { key: "c", suggestion: null, afterLocationQuery: false },
      ],
    };

    const { store, linkLaunch } = setUp(parse, {}, "?gamma=abc&gama=3&utm_source=paper&c=mcc");

    await linkLaunch.start(store);

    expect(linkLaunch.getSnapshot()).toStrictEqual({
      status: { kind: "none" },
      notes: [
        'gamma must be a number, got "abc"',
        'TreeKnit ignored the key "gama"; did you mean "gamma"?',
        '"c" is an Auspice setting: put Auspice settings inside the auspice key, for example auspice=c=mcc.',
      ],
    });
  });

  test("does not parse a link without keys", async () => {
    const { store, linkLaunch, client } = setUp(parsed(URL_TREES), {}, "");

    await linkLaunch.start(store);

    expect({ calls: client.parseCalls, state: linkLaunch.getSnapshot() }).toStrictEqual({
      calls: 0,
      state: { status: { kind: "none" }, notes: [] },
    });
  });
});

describe("linkEntries", () => {
  test.each([
    ["the query alone", { search: "?example=a&run", hash: "" }, ["example", "run"]],
    ["the query and the keys of the fragment", { search: "?run", hash: "#session=data:,x" }, ["run", "session"]],
    ["the query and not an anchor", { search: "?run", hash: "#help-cite" }, ["run"]],
  ])("read %s", (_case, location, keys) => {
    expect(linkEntries(location).map(({ key }) => key)).toStrictEqual(keys);
  });
});

describe("ignoredKeyNote", () => {
  test.each([
    [
      "a key after an address with a query",
      { key: "sig", suggestion: null, afterLocationQuery: true },
      'The key "sig" seems to belong to the address before it: write & inside an address as %26.',
    ],
    [
      "an Auspice filter",
      { key: "f_region", suggestion: null, afterLocationQuery: false },
      '"f_region" is an Auspice setting: put Auspice settings inside the auspice key, for example auspice=c=mcc.',
    ],
    [
      "a misspelled key",
      { key: "aupsice", suggestion: "auspice", afterLocationQuery: false },
      'TreeKnit ignored the key "aupsice"; did you mean "auspice"?',
    ],
    ["a key of another site", { key: "fbclid", suggestion: null, afterLocationQuery: false }, null],
  ])("explains %s", (_case, key, note) => {
    expect(ignoredKeyNote(key)).toBe(note);
  });
});

describe("loadingMessage", () => {
  test.each<[string, LaunchInput, string]>([
    ["trees from one site", URL_TREES.input, "Loading 2 trees from x.org"],
    ["an example", { kind: "example", id: "h3n2-2017" }, "Loading the example h3n2-2017"],
    ["inline trees", { kind: "trees", trees: [] }, "Loading 0 trees from the link"],
    [
      "a message",
      { kind: "message", source: "opener" },
      "Waiting for the session file from the page that opened TreeKnit",
    ],
  ])("describes %s", (_case, input, message) => {
    expect(loadingMessage(input)).toBe(message);
  });
});

function setUp(
  parse: LaunchParse,
  overrides: { fetchFile?: FetchFile; run?: (keepView: boolean) => Promise<void>; limits?: LinkLimits },
  query = "?tree=a&tree=b",
  restored: AnalysisRequest | null = null,
): LaunchFixture {
  const store = createWorkspaceStore(
    { treeLabels: unexpected, settingsSchema: unexpected, cancel: () => undefined },
    { defaults: DEFAULTS, restored: restored === null ? null : { request: restored, sources: [] } },
  );

  const client = new FakeLaunchClient(parse, overrides.limits ?? LIMITS);

  const environment: LaunchEnvironment = {
    client,
    fetchFile: overrides.fetchFile ?? (() => Promise.resolve(new Response(HA))),
    loadExample: (example) =>
      Promise.resolve(
        example.trees.map(({ file, label, newick }) => ({ fileName: file, label, newick: newick ?? "" })),
      ),
    receiveSession: unexpected,
    run: overrides.run ?? (() => Promise.resolve()),
  };

  return { store, linkLaunch: new LinkLaunch(environment, linkEntries({ search: query, hash: "" })), client };
}

interface LaunchFixture {
  store: WorkspaceStore;
  linkLaunch: LinkLaunch;
  client: FakeLaunchClient;
}

function parsed(launch: Launch): LaunchParse {
  return { launch, errors: [], ignored: [] };
}

function recordRuns(runs: boolean[]): (keepView: boolean) => Promise<void> {
  return (keepView) => {
    runs.push(keepView);

    return Promise.resolve();
  };
}

async function settled(linkLaunch: LinkLaunch): Promise<void> {
  if (linkLaunch.getSnapshot().status.kind !== "loading") {
    return;
  }

  const { promise, resolve } = Promise.withResolvers<undefined>();

  const stop = linkLaunch.subscribe(() => {
    if (linkLaunch.getSnapshot().status.kind !== "loading") {
      resolve(undefined);
    }
  });

  await promise;
  stop();
}

function unexpected(): never {
  throw new Error("not expected in this test");
}

class FakeLaunchClient implements LaunchClient {
  parseCalls = 0;
  readonly #parse: LaunchParse;
  readonly #limits: LinkLimits;

  constructor(parse: LaunchParse, limits: LinkLimits) {
    this.#parse = parse;
    this.#limits = limits;
  }

  parseLaunch(): Promise<LaunchParse> {
    this.parseCalls += 1;

    return Promise.resolve(this.#parse);
  }

  decodeTreeBytes(bytes: Uint8Array): Promise<string> {
    return Promise.resolve(new TextDecoder().decode(bytes));
  }

  examples(): Promise<ExampleInfo[]> {
    return Promise.resolve([SMALL]);
  }

  readSession(text: string): Promise<AnalysisRequest> {
    return text === SESSION_TEXT ? Promise.resolve(SESSION) : Promise.reject(new Error("not a TreeKnit session file"));
  }

  applySettings(base: Settings, patch: SettingsPatch): Promise<Settings> {
    const changed = Object.fromEntries(Object.entries(patch).filter(([, value]) => value !== null));

    return Promise.resolve({ ...base, ...changed });
  }

  linkLimits(): Promise<LinkLimits> {
    return Promise.resolve(this.#limits);
  }
}
