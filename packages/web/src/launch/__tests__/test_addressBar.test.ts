import type { LinkEntry, LinkSource, Settings, Summary } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { createWorkspaceStore, type WorkspaceStore } from "../../workspace/store";
import type { TreeSource } from "../../workspace/treeSource";
import { linkSearch } from "../addressBar";
import { AddressBarSync, describedLink, shownLink, treeAddress } from "../addressBar";
import { copyCheck, inlineSessionLink } from "../copyLink";
import { linkEntries } from "../entries";

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

const HA = "((A,B),(C,(D,X)));";

const NA = "((A,(B,X)),(C,D));";

const SUMMARY: Summary = {
  pairs: [],
  arg: { status: "built", reassortments: 1 },
  noReassortment: false,
  diagnostics: [],
};

const EXAMPLE_SOURCES: TreeSource[] = [
  { kind: "example", name: "ha.nwk", id: "5-leaves" },
  { kind: "example", name: "na.nwk", id: "5-leaves" },
];

const EXAMPLE_LINK: LinkSource = {
  request: {
    trees: [
      { label: "ha", newick: HA },
      { label: "na", newick: NA },
    ],
    settings: DEFAULTS,
  },
  addresses: [
    { kind: "example", id: "5-leaves", file: "ha.nwk" },
    { kind: "example", id: "5-leaves", file: "na.nwk" },
  ],
  run: false,
};

describe("treeAddress", () => {
  test.each<[string, TreeSource, ReturnType<typeof treeAddress>]>([
    [
      "an example tree",
      { kind: "example", name: "ha.nwk", id: "h3n2-2017" },
      { kind: "example", id: "h3n2-2017", file: "ha.nwk" },
    ],
    ["an example tree of a record without ids", { kind: "example", name: "ha.nwk" }, null],
    [
      "a tree at an address",
      { kind: "url", url: "https://x.org/ha.nwk", name: "ha.nwk", host: "x.org" },
      { kind: "url", url: "https://x.org/ha.nwk" },
    ],
    ["a tree of a data: link", { kind: "data" }, { kind: "data" }],
    ["a file", { kind: "file", name: "ha.nwk" }, null],
    ["a pasted tree", { kind: "paste" }, null],
    ["a tree of a session file", { kind: "session" }, null],
    ["a tree of a message", { kind: "message", origin: "https://nextstrain.org" }, null],
  ])("gives %s its address", (_case, source, address) => {
    expect(treeAddress(source)).toStrictEqual(address);
  });
});

describe("describedLink", () => {
  test("describes the trees and settings of the workspace without a run before a result", () => {
    expect(describedLink(exampleStore().getState())).toStrictEqual(EXAMPLE_LINK);
  });

  test("describes the request of the result with a run", () => {
    const store = exampleStore();

    finishRun(store);

    expect(describedLink(store.getState())).toStrictEqual({ ...EXAMPLE_LINK, run: true });
  });

  test("keeps the address bar after an edit to the form that follows a run, until the next run", () => {
    const store = exampleStore();

    finishRun(store);
    store.getState().setSettings({ ...DEFAULTS, gamma: 5 });

    expect({ described: describedLink(store.getState()), shown: shownLink(store.getState()) }).toStrictEqual({
      described: null,
      shown: { ...EXAMPLE_LINK, addresses: [null, null], run: true },
    });
  });

  test("gives a renamed tree its new label and keeps its address", () => {
    const store = exampleStore();

    store.getState().renameTree(store.getState().trees[0]?.id ?? "", "HA");

    expect(describedLink(store.getState())?.request.trees.map(({ label }) => label)).toStrictEqual(["HA", "na"]);
  });

  test("gives a tree from a file no address", () => {
    const store = exampleStore();

    replaceWithFile(store);

    expect(describedLink(store.getState())?.addresses).toStrictEqual([null, EXAMPLE_LINK.addresses[1]]);
  });
});

describe("linkSearch", () => {
  test("keeps the view keys and replaces every other key with the pairs of the link", () => {
    const previous = {
      view: "mccs",
      mcc: 2,
      tree: ["https://x.org/a.nwk", "https://x.org/b.nwk"],
      utm_source: "paper",
    };

    const pairs: LinkEntry[] = [
      { key: "example", value: "5-leaves" },
      { key: "gamma", value: "3" },
      { key: "run", value: "" },
    ];

    expect(linkSearch(previous, pairs)).toStrictEqual({
      example: "5-leaves",
      gamma: "3",
      run: true,
      view: "mccs",
      mcc: 2,
    });
  });

  test("removes the input keys of trees without an address", () => {
    expect(linkSearch({ view: "files", example: "5-leaves", run: true }, null)).toStrictEqual({ view: "files" });
  });
});

describe("the address bar sync", () => {
  test("writes the link of the workspace and follows its changes", async () => {
    const store = exampleStore();
    const page = new FakePage();
    const sync = new AddressBarSync(store, new FakeLinkClient(), page);

    sync.start();
    await page.settle();
    replaceWithFile(store);
    await page.settle();

    expect(page.written).toStrictEqual([[{ key: "example", value: "5-leaves" }], null]);
  });

  test("writes nothing for an edit to the form after a run", async () => {
    const store = exampleStore();
    const page = new FakePage();
    const sync = new AddressBarSync(store, new FakeLinkClient(), page);

    finishRun(store);
    sync.start();
    await page.settle();
    store.getState().setSettings({ ...DEFAULTS, gamma: 5 });
    await page.settle();

    expect(page.written).toStrictEqual([
      [
        { key: "example", value: "5-leaves" },
        { key: "run", value: "" },
      ],
    ]);
  });

  test("writes nothing outside the workspace page", async () => {
    const page = new FakePage(false);
    const sync = new AddressBarSync(exampleStore(), new FakeLinkClient(), page);

    sync.start();
    await page.settle();

    expect(page.written).toStrictEqual([]);
  });

  test("writes only the latest link when an earlier one answers later", async () => {
    const store = exampleStore();
    const page = new FakePage();
    const client = new FakeLinkClient();
    const sync = new AddressBarSync(store, client, page);

    client.hold = true;
    sync.start();
    client.hold = false;
    store.getState().setSettings({ ...DEFAULTS, gamma: 5 });
    await page.settle();
    client.releaseHeld();
    await page.settle();

    expect(page.written).toStrictEqual([
      [
        { key: "example", value: "5-leaves" },
        { key: "gamma", value: "5" },
      ],
    ]);
  });
});

describe("copy link", () => {
  test("puts an inline session into the fragment next to run and the view keys", () => {
    const link = inlineSessionLink(
      "https://neherlab.github.io/treeknit-rs/",
      { view: "tanglegram", pair: ["ha", "na"], mcc: 3, example: "5-leaves" },
      "data:application/gzip;base64,H4sI+/A=",
    );

    expect({ link, read: linkEntries(new URL(link)) }).toStrictEqual({
      link: "https://neherlab.github.io/treeknit-rs/?run&view=tanglegram&pair=ha:na&mcc=4#session=data:application/gzip;base64,H4sI%2B/A=",
      read: [
        { key: "run", value: true },
        { key: "view", value: "tanglegram" },
        { key: "pair", value: "ha:na" },
        { key: "mcc", value: "4" },
        { key: "session", value: "data:application/gzip;base64,H4sI+/A=" },
      ],
    });
  });

  test.each([
    ["a short link", 1999, { kind: "copy", length: 1999, long: false }],
    ["a link longer than chat apps keep", 2001, { kind: "copy", length: 2001, long: true }],
    ["the longest link an address bar shows", 32_000, { kind: "copy", length: 32_000, long: true }],
    ["a link too long to copy", 32_001, { kind: "tooLong", length: 32_001 }],
  ])("checks %s", (_case, length, check) => {
    expect(copyCheck("x".repeat(length), { maxLinkChars: 32_000, longLinkChars: 2000 })).toStrictEqual(check);
  });
});

function exampleStore(): WorkspaceStore {
  const store = createWorkspaceStore(
    {
      treeLabels: (names) => Promise.resolve(names.map((name) => name.replace(/\.nwk$/u, ""))),
      settingsSchema: () => Promise.reject(new Error("not expected")),
      cancel: () => undefined,
    },
    { defaults: DEFAULTS, restored: null },
  );

  store.getState().openLink(EXAMPLE_LINK.request, EXAMPLE_SOURCES);

  return store;
}

function replaceWithFile(store: WorkspaceStore): void {
  store.getState().replaceTree(store.getState().trees[0]?.id ?? "", HA, { kind: "file", name: "ha.nwk" });
}

function finishRun(store: WorkspaceStore): void {
  const request = describedLink(store.getState())?.request ?? EXAMPLE_LINK.request;

  store.getState().runStarted(1, request);
  store.getState().runFinished(1, { status: "succeeded", sessionId: 1, summary: SUMMARY });
}

class FakePage {
  readonly written: (readonly LinkEntry[] | null)[] = [];
  readonly #onWorkspace: boolean;

  constructor(onWorkspace = true) {
    this.#onWorkspace = onWorkspace;
  }

  onWorkspace(): boolean {
    return this.#onWorkspace;
  }

  write(pairs: readonly LinkEntry[] | null): Promise<void> {
    this.written.push(pairs);

    return Promise.resolve();
  }

  async settle(): Promise<void> {
    for (let round = 0; round < 5; round += 1) {
      await Promise.resolve();
    }
  }
}

class FakeLinkClient {
  hold = false;
  readonly #held: (() => void)[] = [];

  launchPairs({ addresses, request, run }: LinkSource): Promise<LinkEntry[] | undefined> {
    const pairs = addresses.every((address) => address?.kind === "example")
      ? [
          { key: "example", value: "5-leaves" },
          ...(request.settings?.gamma === 5 ? [{ key: "gamma", value: "5" }] : []),
          ...(run ? [{ key: "run", value: "" }] : []),
        ]
      : undefined;

    if (!this.hold) {
      return Promise.resolve(pairs);
    }

    const { promise, resolve } = Promise.withResolvers<LinkEntry[] | undefined>();

    this.#held.push(() => {
      resolve(pairs);
    });

    return promise;
  }

  releaseHeld(): void {
    for (const release of this.#held) {
      release();
    }
  }
}
