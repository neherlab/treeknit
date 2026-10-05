import type { AnalysisRequest, Progress, Settings, SettingsSchema, Summary, TreeText } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";
import { match } from "ts-pattern";
import { immer } from "zustand/middleware/immer";
import { createStore, type StoreApi } from "zustand/vanilla";

import { type FailureKind, RESULTS_LOST_MESSAGE, type RunOutcome } from "../analysis/client";
import { SESSION_SOURCE, sourceFileName, type TreeSource } from "./treeSource";

export interface WorkspaceTree {
  id: string;
  label: string;
  newick: string;
  source: TreeSource;
}

export interface NewTree {
  newick: string;
  source: TreeSource;
  label?: string;
}

export type RunState =
  | { status: "idle" }
  | { status: "running"; runId: number; request: AnalysisRequest; progress: Progress | null; startedAt: number }
  | { status: "failed"; kind: FailureKind; message: string };

export interface RunResult {
  sessionId: number;
  summary: Summary;
  request: AnalysisRequest;
}

export type UndoEntry =
  | { kind: "tree"; tree: WorkspaceTree; index: number; seqLength: number | null }
  | { kind: "workspace"; trees: WorkspaceTree[]; settings: Settings; result: RunResult | null };

export interface WorkspaceData {
  trees: WorkspaceTree[];
  settings: Settings;
  run: RunState;
  result: RunResult | null;
  undo: UndoEntry | null;
  restored: boolean;
  persistence: boolean;
  nextTreeNumber: number;
}

export interface WorkspaceActions {
  addTrees(trees: readonly NewTree[]): Promise<void>;
  renameTree(id: string, label: string): void;
  replaceTree(id: string, newick: string, source: TreeSource): void;
  reorderTrees(order: readonly string[]): void;
  removeTree(id: string): void;
  restoreUndo(): void;
  setSettings(settings: Settings): void;
  setSeqLengthsEnabled(enabled: boolean): Promise<void>;
  clear(): void;
  loadRequest(request: AnalysisRequest): void;
  runStarted(runId: number, request: AnalysisRequest, startedAt: number): void;
  runProgressed(runId: number, progress: Progress): void;
  runFinished(runId: number, outcome: RunOutcome): void;
  resultLost(sessionId: number): void;
  setPersistence(enabled: boolean): void;
}

export type WorkspaceState = WorkspaceData & WorkspaceActions;

export type WorkspaceStore = StoreApi<WorkspaceState>;

export interface WorkspaceServices {
  treeLabels(fileNames: string[], existingLabels: string[]): Promise<string[]>;
  settingsSchema(k: number, settings: Settings): Promise<SettingsSchema>;
}

export interface RestoredWorkspace {
  request: AnalysisRequest;
  sources: readonly TreeSource[];
}

export interface WorkspaceStart {
  defaults: Settings;
  restored: RestoredWorkspace | null;
}

const requestCache = new WeakMap<WorkspaceTree[], WeakMap<Settings, AnalysisRequest>>();

export function createWorkspaceStore(services: WorkspaceServices, start: WorkspaceStart): WorkspaceStore {
  const { defaults } = start;
  const restoredTrees = start.restored === null ? [] : sessionTrees(start.restored.request, start.restored.sources, 1);
  const pendingAdds = { queue: Promise.resolve() };

  return createStore<WorkspaceState>()(
    immer((set, get) => {
      const addNow = async (newTrees: readonly NewTree[]): Promise<void> => {
        const before = get();
        const automatic = newTrees.filter((tree) => tree.label === undefined);
        const explicitLabels = newTrees.flatMap((tree) => (tree.label === undefined ? [] : [tree.label]));
        const existingLabels = [...before.trees.map((tree) => tree.label), ...explicitLabels];

        const labels = await services.treeLabels(
          automatic.map((tree) => sourceFileName(tree.source)),
          existingLabels,
        );

        const schema = await services.settingsSchema(before.trees.length + newTrees.length, before.settings);
        const automaticLabels = labels.values();

        set((state) => {
          for (const tree of newTrees) {
            state.trees.push({
              id: `tree-${String(state.nextTreeNumber)}`,
              label: tree.label ?? automaticLabels.next().value ?? sourceFileName(tree.source),
              newick: tree.newick,
              source: tree.source,
            });
            state.nextTreeNumber += 1;
            state.settings.seqLengths?.push(schema.settings.seqLengths.default);
          }

          markEdited(state);
        });
      };

      return {
        trees: restoredTrees,
        settings: start.restored === null ? defaults : mergeSettings(defaults, start.restored.request.settings),
        run: { status: "idle" },
        result: null,
        undo: null,
        restored: start.restored !== null,
        persistence: false,
        nextTreeNumber: restoredTrees.length + 1,

        async addTrees(newTrees) {
          const task = pendingAdds.queue.then(async () => addNow(newTrees));

          pendingAdds.queue = task.catch(ignore);

          return task;
        },

        renameTree(id, label) {
          set((state) => {
            const tree = state.trees.find((candidate) => candidate.id === id);

            if (tree !== undefined) {
              tree.label = label;
              markEdited(state);
            }
          });
        },

        replaceTree(id, newick, source) {
          set((state) => {
            const tree = state.trees.find((candidate) => candidate.id === id);

            if (tree !== undefined) {
              tree.newick = newick;
              tree.source = source;
              markEdited(state);
            }
          });
        },

        reorderTrees(order) {
          set((state) => {
            const positions = order.map((id) => state.trees.findIndex((tree) => tree.id === id));

            if (
              positions.length !== state.trees.length ||
              new Set(positions).size !== positions.length ||
              positions.includes(-1)
            ) {
              return;
            }

            const seqLengths = state.settings.seqLengths;

            state.trees = positions.flatMap((position) => state.trees.slice(position, position + 1));

            if (seqLengths !== null && seqLengths !== undefined) {
              state.settings.seqLengths = positions.flatMap((position) => seqLengths.slice(position, position + 1));
            }

            markEdited(state);
          });
        },

        removeTree(id) {
          set((state) => {
            const index = state.trees.findIndex((tree) => tree.id === id);

            if (index === -1) {
              return;
            }

            const [tree] = state.trees.splice(index, 1);

            if (tree === undefined) {
              return;
            }

            const [seqLength] = state.settings.seqLengths?.splice(index, 1) ?? [];

            markEdited(state);
            state.undo = { kind: "tree", tree, index, seqLength: seqLength ?? null };
          });
        },

        restoreUndo() {
          set((state) => {
            const entry = state.undo;

            if (entry === null) {
              return;
            }

            match(entry)
              .with({ kind: "tree" }, ({ tree, index, seqLength }) => {
                state.trees.splice(index, 0, tree);

                if (seqLength !== null) {
                  state.settings.seqLengths?.splice(index, 0, seqLength);
                }
              })
              .with({ kind: "workspace" }, ({ trees, settings, result }) => {
                state.trees = trees;
                state.settings = settings;
                state.result = result;
              })
              .exhaustive();
            state.undo = null;
          });
        },

        setSettings(settings) {
          set((state) => {
            state.settings = settings;
            markEdited(state);
          });
        },

        async setSeqLengthsEnabled(enabled) {
          if (!enabled) {
            set((state) => {
              state.settings.seqLengths = null;
              markEdited(state);
            });

            return;
          }

          const before = get();
          const schema = await services.settingsSchema(before.trees.length, before.settings);

          set((state) => {
            state.settings.seqLengths = state.trees.map(() => schema.settings.seqLengths.default);
            markEdited(state);
          });
        },

        clear() {
          set((state) => {
            replaceWorkspace(state, [], defaults);
          });
        },

        loadRequest(request) {
          set((state) => {
            const trees = sessionTrees(request, [], state.nextTreeNumber);

            state.nextTreeNumber += trees.length;
            replaceWorkspace(state, trees, mergeSettings(defaults, request.settings));
          });
        },

        runStarted(runId, request, startedAt) {
          set((state) => {
            state.run = { status: "running", runId, request, progress: null, startedAt };
            state.restored = false;
          });
        },

        runProgressed(runId, progress) {
          set((state) => {
            if (state.run.status === "running" && state.run.runId === runId) {
              state.run.progress = progress;
            }
          });
        },

        runFinished(runId, outcome) {
          set((state) => {
            const run = state.run;

            if (run.status !== "running" || run.runId !== runId) {
              return;
            }

            match(outcome)
              .with({ status: "succeeded" }, ({ sessionId, summary }) => {
                state.result = { sessionId, summary, request: run.request };
                state.run = { status: "idle" };
              })
              .with({ status: "failed" }, ({ kind, message }) => {
                state.run = { status: "failed", kind, message };
              })
              .exhaustive();
          });
        },

        resultLost(sessionId) {
          set((state) => {
            if (state.undo?.kind === "workspace" && state.undo.result?.sessionId === sessionId) {
              state.undo.result = null;
            }

            if (state.result?.sessionId !== sessionId) {
              return;
            }

            state.result = null;

            if (state.run.status !== "running") {
              state.run = { status: "failed", kind: "internal", message: RESULTS_LOST_MESSAGE };
            }
          });
        },

        setPersistence(enabled) {
          set((state) => {
            state.persistence = enabled;
          });
        },
      };
    }),
  );
}

export function selectRequest(state: WorkspaceData): AnalysisRequest {
  const bySettings = requestCache.get(state.trees) ?? new WeakMap<Settings, AnalysisRequest>();
  const cached = bySettings.get(state.settings);

  if (cached !== undefined) {
    return cached;
  }

  const request: AnalysisRequest = {
    trees: state.trees.map(({ label, newick }): TreeText => ({ label, newick })),
    settings: state.settings,
  };

  bySettings.set(state.settings, request);
  requestCache.set(state.trees, bySettings);

  return request;
}

export function selectStale(state: WorkspaceData): boolean {
  return state.result !== null && !isDeepEqual(selectRequest(state), state.result.request);
}

export function selectRunning(state: WorkspaceData): boolean {
  return state.run.status === "running";
}

function sessionTrees(request: AnalysisRequest, sources: readonly TreeSource[], firstNumber: number): WorkspaceTree[] {
  return request.trees.map(({ label, newick }, index) => ({
    id: `tree-${String(firstNumber + index)}`,
    label,
    newick,
    source: sources[index] ?? SESSION_SOURCE,
  }));
}

function mergeSettings(defaults: Settings, settings: Settings | undefined): Settings {
  return { ...defaults, ...settings };
}

function replaceWorkspace(state: WorkspaceData, trees: WorkspaceTree[], settings: Settings): void {
  state.undo = { kind: "workspace", trees: state.trees, settings: state.settings, result: state.result };
  state.trees = trees;
  state.settings = settings;
  state.result = null;
  state.restored = false;
}

function markEdited(state: WorkspaceData): void {
  state.undo = null;
  state.restored = false;
}

function ignore(): undefined {
  return undefined;
}
