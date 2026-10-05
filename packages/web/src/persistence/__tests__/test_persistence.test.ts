import type { AnalysisRequest, OutputFile } from "@neherlab/treeknit-wasm";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { SAVE_DELAY_MS, type WorkspaceSnapshot, WorkspacePersistence } from "../persistence";
import type { PersistenceChannel, StoredRecord } from "../record";
import { MemoryChannelHub, MemoryStorage } from "./memoryStorage";

const ONE_TREE: WorkspaceSnapshot = {
  request: { trees: [{ label: "ha", newick: "(A,B);" }], settings: { gamma: 2 } },
  sources: [{ kind: "file", name: "ha.nwk" }],
};

const TWO_TREES: WorkspaceSnapshot = {
  request: {
    trees: [
      { label: "ha", newick: "(A,B);" },
      { label: "na", newick: "(A,C);" },
    ],
    settings: { gamma: 2 },
  },
  sources: [{ kind: "file", name: "ha.nwk" }, { kind: "paste" }],
};

const EMPTY: WorkspaceSnapshot = { request: { trees: [], settings: { gamma: 2 } }, sources: [] };

describe("workspace persistence", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("saves nothing while the switch is off", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS * 4);

    expect({ record: storage.record, writes: storage.writes, requested: tab.files.requested }).toStrictEqual({
      record: undefined,
      writes: 0,
      requested: 0,
    });
  });

  test("turning the switch on stores the workspace with the next generation", async () => {
    const storage = new MemoryStorage({ kind: "off", generation: 4 });
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);

    expect({ record: storage.record, switches: tab.switches }).toStrictEqual({
      record: workspaceRecord(5, ONE_TREE),
      switches: [true],
    });
  });

  test("saves the latest workspace once, after the save delay", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(EMPTY);
    tab.persistence.changed(ONE_TREE);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS - 1);
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS - 1);
    const early = storage.record;

    await vi.advanceTimersByTimeAsync(1);

    expect({ early, saved: storage.record, writes: storage.writes }).toStrictEqual({
      early: workspaceRecord(1, EMPTY),
      saved: workspaceRecord(1, TWO_TREES),
      writes: 2,
    });
  });

  test("turning the switch off replaces the stored workspace with an off marker at once", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(TWO_TREES);
    await tab.persistence.disable();

    expect({ record: storage.record, switches: tab.switches }).toStrictEqual({
      record: { kind: "off", generation: 2 },
      switches: [true, false],
    });
  });

  test("a stored workspace restores and turns the switch on", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, TWO_TREES));
    const tab = new Tab(storage);

    const restored = await tab.persistence.restore();

    expect({ restored, switches: tab.switches, enabled: tab.persistence.enabled }).toStrictEqual({
      restored: { sessionFile: sessionFileText(TWO_TREES.request), sources: TWO_TREES.sources },
      switches: [true],
      enabled: true,
    });
  });

  test("an off marker or no record restores nothing and leaves the switch off", async () => {
    const off = new Tab(new MemoryStorage({ kind: "off", generation: 2 }));
    const none = new Tab(new MemoryStorage());

    expect({
      off: await off.persistence.restore(),
      none: await none.persistence.restore(),
      switches: [...off.switches, ...none.switches],
    }).toStrictEqual({ off: null, none: null, switches: [] });
  });

  test("a pending save timer does not write after the switch is turned off", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.persistence.changed(TWO_TREES);
    await tab.persistence.disable();
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS * 2);

    expect(storage.record).toStrictEqual({ kind: "off", generation: 2 });
  });

  test("a save still waiting for the session file is dropped when the switch is turned off", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.files.hold();
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await tab.persistence.disable();
    tab.files.release();
    await vi.advanceTimersByTimeAsync(0);

    expect(storage.record).toStrictEqual({ kind: "off", generation: 2 });
  });

  test("clearing the workspace drops a pending save and a save waiting for the session file", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.files.hold();
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    tab.persistence.changed(ONE_TREE);
    tab.persistence.changed(EMPTY);
    tab.files.release();
    await vi.advanceTimersByTimeAsync(0);
    const afterRelease = storage.record;

    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ afterRelease, record: storage.record, enabled: tab.persistence.enabled }).toStrictEqual({
      afterRelease: workspaceRecord(1, ONE_TREE),
      record: workspaceRecord(1, EMPTY),
      enabled: true,
    });
  });

  test("keeps the previous stored value when the session file fails", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.files.failNext();
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ record: storage.record, enabled: tab.persistence.enabled }).toStrictEqual({
      record: workspaceRecord(1, ONE_TREE),
      enabled: true,
    });
  });

  test("a save that started before another tab turned persistence off neither writes nor keeps its switch on", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.persistence.restore();
    await second.persistence.restore();
    second.files.hold();
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await first.persistence.disable();
    second.files.release();
    await vi.advanceTimersByTimeAsync(0);

    expect({ record: storage.record, second: second.switches, enabled: second.persistence.enabled }).toStrictEqual({
      record: { kind: "off", generation: 2 },
      second: [true, false],
      enabled: false,
    });
  });

  test("a save that started before another tab turned persistence off and on again does not write", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.persistence.restore();
    await second.persistence.restore();
    second.files.hold();
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await first.persistence.disable();
    await first.persistence.enable(EMPTY);
    second.files.release();
    await vi.advanceTimersByTimeAsync(0);

    expect({ record: storage.record, enabled: second.persistence.enabled }).toStrictEqual({
      record: workspaceRecord(3, EMPTY),
      enabled: false,
    });
  });

  test("a tab that never saved does not write after another tab took over persistence", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.persistence.restore();
    await second.persistence.restore();
    await first.persistence.disable();
    await first.persistence.enable(EMPTY);
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ record: storage.record, second: second.switches }).toStrictEqual({
      record: workspaceRecord(3, EMPTY),
      second: [true, false],
    });
  });

  test("two tabs with one generation both save, and the last writer wins", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, EMPTY));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.persistence.restore();
    await second.persistence.restore();
    first.persistence.changed(ONE_TREE);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const afterFirst = storage.record;

    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({
      afterFirst,
      afterSecond: storage.record,
      enabled: [first.persistence.enabled, second.persistence.enabled],
    }).toStrictEqual({
      afterFirst: workspaceRecord(1, ONE_TREE),
      afterSecond: workspaceRecord(1, TWO_TREES),
      enabled: [true, true],
    });
  });

  test("with the channel, turning persistence off in one tab turns the other switch off at once", async () => {
    const hub = new MemoryChannelHub();
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage, hub.channel());
    const second = new Tab(storage, hub.channel());

    await first.persistence.restore();
    await second.persistence.restore();
    await first.persistence.disable();

    expect({ second: second.switches, writes: storage.writes }).toStrictEqual({ second: [true, false], writes: 1 });
  });

  test("with the channel, turning persistence on in one tab turns the other switch off", async () => {
    const hub = new MemoryChannelHub();
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage, hub.channel());
    const second = new Tab(storage, hub.channel());

    await second.persistence.restore();
    await first.persistence.enable(EMPTY);

    expect({ first: first.switches, second: second.switches }).toStrictEqual({ first: [true], second: [true, false] });
  });
});

class Tab {
  readonly switches: boolean[] = [];
  readonly files = new SessionFiles();
  readonly persistence: WorkspacePersistence;

  constructor(storage: MemoryStorage, channel: PersistenceChannel | null = null) {
    this.persistence = new WorkspacePersistence({
      storage,
      channel,
      requestFile: async (request) => this.files.requestFile(request),
      onSwitchChange: (enabled) => {
        this.switches.push(enabled);
      },
    });
  }
}

class SessionFiles {
  requested = 0;
  #held: PromiseWithResolvers<undefined> | undefined;
  #failNext = false;

  hold(): void {
    this.#held = Promise.withResolvers<undefined>();
  }

  release(): void {
    this.#held?.resolve(undefined);
    this.#held = undefined;
  }

  failNext(): void {
    this.#failNext = true;
  }

  async requestFile(request: AnalysisRequest): Promise<OutputFile> {
    this.requested += 1;
    await this.#held?.promise;

    if (this.#failNext) {
      this.#failNext = false;

      throw new Error("internal error");
    }

    return { path: "treeknit_request.json", mediaType: "application/json", text: sessionFileText(request) };
  }
}

function workspaceRecord(generation: number, snapshot: WorkspaceSnapshot): StoredRecord {
  return {
    kind: "workspace",
    generation,
    sessionFile: sessionFileText(snapshot.request),
    sources: snapshot.sources,
  };
}

function sessionFileText(request: AnalysisRequest): string {
  return JSON.stringify(request);
}
