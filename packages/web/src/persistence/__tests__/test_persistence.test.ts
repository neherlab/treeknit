import type { AnalysisRequest, OutputFile } from "@neherlab/treeknit-wasm";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { SAVE_DELAY_MS, type StoredWorkspace, type WorkspaceSnapshot, WorkspacePersistence } from "../persistence";
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
    const storage = new MemoryStorage({ kind: "off", version: 1, generation: 4 });
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
      record: { kind: "off", version: 1, generation: 2 },
      switches: [true, false],
    });
  });

  test("a stored workspace restores and turns the switch on", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, TWO_TREES));
    const tab = new Tab(storage);

    const restored = await tab.restore();

    expect({ restored, switches: tab.switches, enabled: tab.persistence.state.enabled }).toStrictEqual({
      restored: { sessionFile: sessionFileText(TWO_TREES.request), sources: TWO_TREES.sources },
      switches: [true],
      enabled: true,
    });
  });

  test("a restore whose stored workspace is turned off in another tab during parsing restores nothing", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, TWO_TREES));
    const hub = new MemoryChannelHub();
    const restoring = new Tab(storage, hub.channel());
    const other = new Tab(storage, hub.channel());
    const parsing = Promise.withResolvers<undefined>();

    await other.restore();

    const restored = restoring.persistence.restore(async (stored) => {
      await parsing.promise;

      return stored;
    });

    await vi.advanceTimersByTimeAsync(0);
    await other.persistence.disable();
    parsing.resolve(undefined);

    expect({
      restored: await restored,
      switches: restoring.switches,
      enabled: restoring.persistence.state.enabled,
      record: storage.record,
    }).toStrictEqual({
      restored: null,
      switches: [],
      enabled: false,
      record: { kind: "off", version: 1, generation: 4 },
    });
  });

  test("a restore whose stored workspace is saved again during parsing restores the newer workspace", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, ONE_TREE));
    const parsed: string[] = [];
    const parsing = Promise.withResolvers<undefined>();
    const tab = new Tab(storage);

    const restored = tab.persistence.restore(async (stored) => {
      parsed.push(stored.sessionFile);
      await parsing.promise;

      return stored;
    });

    await vi.advanceTimersByTimeAsync(0);
    storage.record = workspaceRecord(3, TWO_TREES);
    parsing.resolve(undefined);

    expect({ restored: await restored, parsed: parsed.length, switches: tab.switches }).toStrictEqual({
      restored: { sessionFile: sessionFileText(TWO_TREES.request), sources: TWO_TREES.sources },
      parsed: 2,
      switches: [true],
    });
  });

  test("an off marker or no record restores nothing and leaves the switch off", async () => {
    const off = new Tab(new MemoryStorage({ kind: "off", version: 1, generation: 2 }));
    const none = new Tab(new MemoryStorage());

    expect({
      off: await off.restore(),
      none: await none.restore(),
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

    expect(storage.record).toStrictEqual({ kind: "off", version: 1, generation: 2 });
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

    expect(storage.record).toStrictEqual({ kind: "off", version: 1, generation: 2 });
  });

  test("a newer change drops a pending save and a save waiting for the session file", async () => {
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

    expect({ afterRelease, record: storage.record, enabled: tab.persistence.state.enabled }).toStrictEqual({
      afterRelease: workspaceRecord(1, ONE_TREE),
      record: workspaceRecord(1, EMPTY),
      enabled: true,
    });
  });

  test("a failed session file keeps the previous stored value and reports the failed save until a save succeeds", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.files.failNext();
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const failed = { record: storage.record, state: tab.persistence.state };

    tab.persistence.changed(EMPTY);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ failed, record: storage.record, state: tab.persistence.state }).toStrictEqual({
      failed: {
        record: workspaceRecord(1, ONE_TREE),
        state: { enabled: true, problem: { kind: "save", message: "internal error" } },
      },
      record: workspaceRecord(1, EMPTY),
      state: { enabled: true, problem: null },
    });
  });

  test("a failed storage write reports the failed save, and saving now writes the unsaved workspace", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    storage.failNextUpdate(new Error("The quota has been exceeded."));
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const failed = { record: storage.record, state: tab.persistence.state };

    await tab.persistence.saveNow();

    expect({ failed, record: storage.record, state: tab.persistence.state }).toStrictEqual({
      failed: {
        record: workspaceRecord(1, ONE_TREE),
        state: { enabled: true, problem: { kind: "save", message: "The quota has been exceeded." } },
      },
      record: workspaceRecord(1, TWO_TREES),
      state: { enabled: true, problem: null },
    });
  });

  test("saving now writes a pending change before the save delay", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    tab.persistence.changed(TWO_TREES);
    await tab.persistence.saveNow();
    const saved = storage.record;

    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS * 2);

    expect({ saved, writes: storage.writes }).toStrictEqual({ saved: workspaceRecord(1, TWO_TREES), writes: 2 });
  });

  test("a save that a newer change replaces during its storage write leaves the record as it is", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(EMPTY);
    tab.persistence.changed(ONE_TREE);
    storage.holdUpdate();
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await storage.updateStarted();
    tab.persistence.changed(TWO_TREES);
    storage.releaseUpdate();
    await vi.advanceTimersByTimeAsync(0);
    const replaced = { record: storage.record, writes: storage.writes };

    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ replaced, saved: storage.record, writes: storage.writes }).toStrictEqual({
      replaced: { record: workspaceRecord(1, EMPTY), writes: 1 },
      saved: workspaceRecord(1, TWO_TREES),
      writes: 2,
    });
  });

  test("a save over a record this version cannot read keeps that record and turns the switch off", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);
    const newer = { kind: "workspace", version: 2, generation: 1, payload: "from a newer version" };

    await tab.persistence.enable(EMPTY);
    storage.record = newer;
    tab.persistence.changed(ONE_TREE);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ record: storage.record, writes: storage.writes, switches: tab.switches }).toStrictEqual({
      record: newer,
      writes: 1,
      switches: [true, false],
    });
  });

  test("a change made while the session file of turning on is written is saved after the save delay", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    tab.files.hold();
    const enabling = tab.persistence.enable(ONE_TREE);

    tab.persistence.changed(TWO_TREES);
    tab.files.release();
    await enabling;
    const enabled = { record: storage.record, switches: [...tab.switches] };

    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ enabled, saved: storage.record, writes: storage.writes }).toStrictEqual({
      enabled: { record: workspaceRecord(1, ONE_TREE), switches: [true] },
      saved: workspaceRecord(1, TWO_TREES),
      writes: 2,
    });
  });

  test("a change made during the storage write of turning on is saved after the save delay", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    storage.holdUpdate();
    const enabling = tab.persistence.enable(ONE_TREE);

    await storage.updateStarted();
    tab.persistence.changed(TWO_TREES);
    storage.releaseUpdate();
    await enabling;
    const enabled = storage.record;

    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ enabled, saved: storage.record }).toStrictEqual({
      enabled: workspaceRecord(1, ONE_TREE),
      saved: workspaceRecord(1, TWO_TREES),
    });
  });

  test("turning the switch off while it is turning on leaves it off with an off marker", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, ONE_TREE));
    const tab = new Tab(storage);

    tab.files.hold();
    const enabling = tab.persistence.enable(TWO_TREES);

    await tab.persistence.disable();
    tab.files.release();
    await enabling;

    expect({ record: storage.record, switches: tab.switches, enabled: tab.persistence.state.enabled }).toStrictEqual({
      record: { kind: "off", version: 1, generation: 4 },
      switches: [],
      enabled: false,
    });
  });

  test("turning the switch off during the storage write of turning on leaves it off with an off marker", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    storage.holdUpdate();
    const enabling = tab.persistence.enable(ONE_TREE);

    await storage.updateStarted();
    const disabling = tab.persistence.disable();

    storage.releaseUpdate();
    await Promise.all([enabling, disabling]);

    expect({ record: storage.record, switches: tab.switches }).toStrictEqual({
      record: { kind: "off", version: 1, generation: 1 },
      switches: [],
    });
  });

  test("a failed storage write when turning on reports it and leaves the switch off", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    storage.failNextUpdate(new Error("The quota has been exceeded."));
    await tab.persistence.enable(ONE_TREE);

    expect({ record: storage.record, state: tab.persistence.state }).toStrictEqual({
      record: undefined,
      state: { enabled: false, problem: { kind: "enable", message: "The quota has been exceeded." } },
    });
  });

  test("a failed off marker reports it and leaves the switch on, still saving, until turning off again succeeds", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    storage.failNextUpdate(new Error("The database was closed."));
    await tab.persistence.disable();
    const failed = { record: storage.record, state: tab.persistence.state };

    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const afterChange = storage.record;

    await tab.persistence.disable();

    expect({
      failed,
      afterChange,
      record: storage.record,
      state: tab.persistence.state,
      switches: tab.switches,
    }).toStrictEqual({
      failed: {
        record: workspaceRecord(1, ONE_TREE),
        state: { enabled: true, problem: { kind: "disable", message: "The database was closed." } },
      },
      afterChange: workspaceRecord(1, TWO_TREES),
      record: { kind: "off", version: 1, generation: 2 },
      state: { enabled: false, problem: null },
      switches: [true, false],
    });
  });

  test("the switch stays on until the off marker is written", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    storage.holdUpdate();
    const disabling = tab.persistence.disable();

    await storage.updateStarted();
    const during = tab.persistence.state.enabled;

    storage.releaseUpdate();
    await disabling;

    expect({ during, after: tab.persistence.state.enabled }).toStrictEqual({ during: true, after: false });
  });

  test("a change while the off marker is written is not saved, and is saved after the marker fails", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    storage.holdUpdate();
    storage.failNextUpdate(new Error("The database was closed."));
    const disabling = tab.persistence.disable();

    await storage.updateStarted();
    tab.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const during = storage.record;

    storage.releaseUpdate();
    await disabling;
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ during, after: storage.record }).toStrictEqual({
      during: workspaceRecord(1, ONE_TREE),
      after: workspaceRecord(1, TWO_TREES),
    });
  });

  test("a change while the off marker is written is dropped once the marker is written", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    await tab.persistence.enable(ONE_TREE);
    storage.holdUpdate();
    const disabling = tab.persistence.disable();

    await storage.updateStarted();
    tab.persistence.changed(TWO_TREES);
    storage.releaseUpdate();
    await disabling;
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS * 2);

    expect(storage.record).toStrictEqual({ kind: "off", version: 1, generation: 2 });
  });

  test("a stored workspace that cannot be read back stays stored, with the switch off and the failure reported", async () => {
    const storage = new MemoryStorage(workspaceRecord(3, TWO_TREES));
    const tab = new Tab(storage);

    const restored = await tab.persistence.restore(() => Promise.reject(new Error("not a TreeKnit session file")));

    tab.persistence.changed(ONE_TREE);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({ restored, record: storage.record, state: tab.persistence.state }).toStrictEqual({
      restored: null,
      record: workspaceRecord(3, TWO_TREES),
      state: { enabled: false, problem: { kind: "restore", message: "not a TreeKnit session file" } },
    });
  });

  test("a storage that cannot be opened reports unavailable storage", async () => {
    const storage = new MemoryStorage();
    const tab = new Tab(storage);

    storage.failNextRead(new Error("The database could not be opened."));

    expect({ restored: await tab.restore(), state: tab.persistence.state }).toStrictEqual({
      restored: null,
      state: { enabled: false, problem: { kind: "unavailable", message: "The database could not be opened." } },
    });
  });

  test("a stored record that does not parse reports the failed restore and stays stored", async () => {
    const unreadable = { kind: "workspace", version: 1, generation: 3 };
    const storage = new MemoryStorage(unreadable);
    const tab = new Tab(storage);

    const restored = await tab.restore();
    const { problem } = tab.persistence.state;

    expect({ restored, record: storage.record, kind: problem?.kind }).toStrictEqual({
      restored: null,
      record: unreadable,
      kind: "restore",
    });
  });

  test("a save that started before another tab turned persistence off neither writes nor keeps its switch on", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.restore();
    await second.restore();
    second.files.hold();
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await first.persistence.disable();
    second.files.release();
    await vi.advanceTimersByTimeAsync(0);

    expect({
      record: storage.record,
      second: second.switches,
      enabled: second.persistence.state.enabled,
    }).toStrictEqual({
      record: { kind: "off", version: 1, generation: 2 },
      second: [true, false],
      enabled: false,
    });
  });

  test("a failed save stops being reported when another tab turns persistence on", async () => {
    const hub = new MemoryChannelHub();
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage, hub.channel());
    const second = new Tab(storage, hub.channel());

    await second.restore();
    second.files.failNext();
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const failed = second.persistence.state;

    await first.persistence.enable(EMPTY);

    expect({ failed, state: second.persistence.state }).toStrictEqual({
      failed: { enabled: true, problem: { kind: "save", message: "internal error" } },
      state: { enabled: false, problem: null },
    });
  });

  test("a save that started before another tab turned persistence off and on again does not write", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.restore();
    await second.restore();
    second.files.hold();
    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    await first.persistence.disable();
    await first.persistence.enable(EMPTY);
    second.files.release();
    await vi.advanceTimersByTimeAsync(0);

    expect({ record: storage.record, enabled: second.persistence.state.enabled }).toStrictEqual({
      record: workspaceRecord(3, EMPTY),
      enabled: false,
    });
  });

  test("a tab that never saved does not write after another tab took over persistence", async () => {
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage);
    const second = new Tab(storage);

    await first.restore();
    await second.restore();
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

    await first.restore();
    await second.restore();
    first.persistence.changed(ONE_TREE);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    const afterFirst = storage.record;

    second.persistence.changed(TWO_TREES);
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);

    expect({
      afterFirst,
      afterSecond: storage.record,
      enabled: [first.persistence.state.enabled, second.persistence.state.enabled],
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

    await first.restore();
    await second.restore();
    await first.persistence.disable();

    expect({ second: second.switches, writes: storage.writes }).toStrictEqual({ second: [true, false], writes: 1 });
  });

  test("with the channel, turning persistence on in one tab turns the other switch off", async () => {
    const hub = new MemoryChannelHub();
    const storage = new MemoryStorage(workspaceRecord(1, ONE_TREE));
    const first = new Tab(storage, hub.channel());
    const second = new Tab(storage, hub.channel());

    await second.restore();
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
    });
    this.persistence.subscribe(() => {
      const { enabled } = this.persistence.state;

      if (this.switches.at(-1) !== enabled && (this.switches.length > 0 || enabled)) {
        this.switches.push(enabled);
      }
    });
  }

  async restore(): Promise<StoredWorkspace | null> {
    return this.persistence.restore((stored) => Promise.resolve(stored));
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
    version: 1,
    generation,
    sessionFile: sessionFileText(snapshot.request),
    sources: snapshot.sources,
  };
}

function sessionFileText(request: AnalysisRequest): string {
  return JSON.stringify(request);
}
