import type { AnalysisRequest, Progress, Settings, SettingsSchema, Summary, TreeText } from "@neherlab/treeknit-wasm";
import { doNothing, isDeepEqual } from "remeda";
import { match } from "ts-pattern";
import { immer } from "zustand/middleware/immer";
import { createStore, type StoreApi } from "zustand/vanilla";

import { type FailureKind, RESULTS_LOST_MESSAGE, type RunOutcome } from "../analysis/client";
import { type Clock, monotonicClock } from "../run/clock";
import { SESSION_SOURCE, sourceFileName, type TreeSource } from "./treeSource";

export interface WorkspaceTree {
  id: string;
  label: string;
  newick: string;
  textId: number;
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
  | { status: "failed"; kind: FailureKind; message: string; request: AnalysisRequest };

export interface RunResult {
  sessionId: number;
  summary: Summary;
  request: AnalysisRequest;
  durationMs: number;
}

export type UndoEntry =
  | { kind: "tree"; tree: WorkspaceTree; index: number; seqLength: number | null }
  | {
      kind: "workspace";
      reason: WorkspaceReplacement;
      trees: WorkspaceTree[];
      settings: Settings;
      result: RunResult | null;
    }
  | { kind: "settings"; settings: Settings };

export type WorkspaceReplacement = "clear" | "session" | "link";

export interface WorkspaceData {
  trees: WorkspaceTree[];
  settings: Settings;
  run: RunState;
  result: RunResult | null;
  defaults: Settings;
  undo: UndoEntry | null;
  restored: boolean;
  nextTreeNumber: number;
  nextTextId: number;
  resetRevision: number;
}

export interface WorkspaceActions {
  addTrees(trees: readonly NewTree[]): Promise<void>;
  renameTree(id: string, label: string): void;
  replaceTree(id: string, newick: string, source: TreeSource): void;
  reorderTrees(order: readonly string[]): void;
  removeTree(id: string): void;
  restoreUndo(): void;
  dismissUndo(entry: UndoEntry): void;
  resetSettings(): void;
  setSettings(settings: Settings): void;
  setSeqLengthsEnabled(enabled: boolean): Promise<void>;
  clear(): void;
  loadRequest(request: AnalysisRequest): void;
  openLink(request: AnalysisRequest, sources: readonly TreeSource[]): void;
  runStarted(runId: number, request: AnalysisRequest): void;
  runProgressed(runId: number, progress: Progress): void;
  runFinished(runId: number, outcome: RunOutcome): void;
  resultLost(sessionId: number): void;
}

export type WorkspaceState = WorkspaceData & WorkspaceActions;

export type WorkspaceStore = StoreApi<WorkspaceState>;

export interface WorkspaceServices {
  treeLabels(fileNames: string[], existingLabels: string[]): Promise<string[]>;
  settingsSchema(k: number, settings: Settings): Promise<SettingsSchema>;
  cancel(): void;
}

interface WorkspaceBasis {
  trees: WorkspaceTree[];
  settings: Settings;
}

interface LabelledTrees {
  trees: WorkspaceTree[];
  labelled: { tree: NewTree; label: string }[];
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

const textIdCache = new WeakMap<WorkspaceTree[], readonly number[]>();

export function createWorkspaceStore(
  services: WorkspaceServices,
  start: WorkspaceStart,
  clock: Clock = monotonicClock,
): WorkspaceStore {
  const { defaults } = start;

  const restoredTrees =
    start.restored === null ? [] : sessionTrees(start.restored.request, start.restored.sources, { tree: 1, text: 1 });

  const pendingAdds = { queue: Promise.resolve() };
  const revisions = { replacement: 0, seqLengthToggle: 0 };

  const labelNewTrees = async (newTrees: readonly NewTree[], trees: WorkspaceTree[]): Promise<LabelledTrees> => {
    const automatic = newTrees.filter((tree) => tree.label === undefined);
    const explicitLabels = newTrees.flatMap((tree) => (tree.label === undefined ? [] : [tree.label]));
    const existingLabels = [...trees.map((tree) => tree.label), ...explicitLabels];

    const labels = await services.treeLabels(
      automatic.map((tree) => sourceFileName(tree.source)),
      existingLabels,
    );

    return { trees, labelled: assignLabels(newTrees, labels) };
  };

  return createStore<WorkspaceState>()(
    immer((set, get) => {
      const isCurrent = (basis: WorkspaceBasis): boolean => {
        const state = get();

        return state.trees === basis.trees && state.settings === basis.settings;
      };

      const addNow = async (
        newTrees: readonly NewTree[],
        replacement: number,
        previous?: LabelledTrees,
      ): Promise<void> => {
        const before = get();
        const current = previous?.trees === before.trees ? previous : await labelNewTrees(newTrees, before.trees);
        const { labelled } = current;
        const schema = await services.settingsSchema(before.trees.length + newTrees.length, before.settings);

        if (revisions.replacement !== replacement) {
          return;
        }

        if (!isCurrent(before)) {
          await addNow(newTrees, replacement, current);

          return;
        }

        set((state) => {
          for (const { tree, label } of labelled) {
            state.trees.push({
              id: `tree-${String(state.nextTreeNumber)}`,
              label,
              newick: tree.newick,
              textId: state.nextTextId,
              source: tree.source,
            });
            state.nextTreeNumber += 1;
            state.nextTextId += 1;
            state.settings.seqLengths?.push(schema.settings.seqLengths.default);
          }

          markEdited(state);
        });
      };

      const enableSeqLengths = async (toggle: number, replacement: number): Promise<void> => {
        const before = get();
        const replaced = () => revisions.seqLengthToggle !== toggle || revisions.replacement !== replacement;
        let schema: Awaited<ReturnType<typeof services.settingsSchema>>;

        try {
          schema = await services.settingsSchema(before.trees.length, before.settings);
        } catch (cause) {
          if (replaced()) {
            return;
          }

          throw cause;
        }

        if (replaced()) {
          return;
        }

        if (!isCurrent(before)) {
          await enableSeqLengths(toggle, replacement);

          return;
        }

        set((state) => {
          state.settings.seqLengths = state.trees.map(() => schema.settings.seqLengths.default);
          markEdited(state);
        });
      };

      const replaceWorkspaceWith = (replace: (state: WorkspaceData) => void): void => {
        const running = selectRunning(get());

        revisions.replacement += 1;
        set(replace);

        if (running) {
          services.cancel();
        }
      };

      return {
        trees: restoredTrees,
        settings: start.restored?.request.settings ?? defaults,
        run: { status: "idle" },
        result: null,
        defaults,
        undo: null,
        restored: start.restored !== null,
        nextTreeNumber: restoredTrees.length + 1,
        nextTextId: restoredTrees.length + 1,
        resetRevision: 0,

        async addTrees(newTrees) {
          const replacement = revisions.replacement;
          const task = pendingAdds.queue.then(async () => addNow(newTrees, replacement));

          pendingAdds.queue = task.catch(doNothing());

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
              tree.textId = state.nextTextId;
              tree.source = source;
              state.nextTextId += 1;
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
          const entry = get().undo;

          if (entry === null) {
            return;
          }

          match(entry)
            .with({ kind: "tree" }, ({ tree, index, seqLength }) => {
              set((state) => {
                state.trees.splice(index, 0, tree);

                if (seqLength !== null) {
                  state.settings.seqLengths?.splice(index, 0, seqLength);
                }

                state.undo = null;
                state.resetRevision += 1;
              });
            })
            .with({ kind: "workspace" }, ({ trees, settings, result }) => {
              replaceWorkspaceWith((state) => {
                state.trees = trees;
                state.settings = settings;
                state.result = result;
                state.run = { status: "idle" };
                state.undo = null;
                state.resetRevision += 1;
              });
            })
            .with({ kind: "settings" }, ({ settings }) => {
              set((state) => {
                state.settings = settings;
                state.undo = null;
                state.resetRevision += 1;
              });
            })
            .exhaustive();
        },

        dismissUndo(entry) {
          set((state) => {
            if (get().undo === entry) {
              state.undo = null;
            }
          });
        },

        resetSettings() {
          set((state) => {
            const { settings } = get();

            state.undo = { kind: "settings", settings };
            state.settings = defaults;
            state.restored = false;
            state.resetRevision += 1;
          });
        },

        setSettings(settings) {
          set((state) => {
            state.settings = settings;
            markEdited(state);
          });
        },

        async setSeqLengthsEnabled(enabled) {
          revisions.seqLengthToggle += 1;

          if (enabled) {
            await enableSeqLengths(revisions.seqLengthToggle, revisions.replacement);

            return;
          }

          set((state) => {
            state.settings.seqLengths = null;
            markEdited(state);
          });
        },

        clear() {
          replaceWorkspaceWith((state) => {
            replaceWorkspace(state, "clear", [], defaults);
          });
        },

        loadRequest(request) {
          replaceWorkspaceWith((state) => {
            const trees = sessionTrees(request, [], { tree: state.nextTreeNumber, text: state.nextTextId });

            state.nextTreeNumber += trees.length;
            state.nextTextId += trees.length;
            replaceWorkspace(state, "session", trees, request.settings ?? defaults);
          });
        },

        openLink(request, sources) {
          if (isDeepEqual(selectRequest(get()), request)) {
            set((state) => {
              state.trees.forEach((tree, index) => {
                tree.source = sources[index] ?? tree.source;
              });
            });

            return;
          }

          replaceWorkspaceWith((state) => {
            const trees = sessionTrees(request, sources, { tree: state.nextTreeNumber, text: state.nextTextId });

            state.nextTreeNumber += trees.length;
            state.nextTextId += trees.length;
            replaceWorkspace(state, "link", trees, request.settings ?? defaults);
          });
        },

        runStarted(runId, request) {
          const startedAt = clock();

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
          const finishedAt = clock();

          set((state) => {
            const run = state.run;

            if (run.status !== "running" || run.runId !== runId) {
              return;
            }

            match(outcome)
              .with({ status: "succeeded" }, ({ sessionId, summary }) => {
                state.result = { sessionId, summary, request: run.request, durationMs: finishedAt - run.startedAt };
                state.run = { status: "idle" };

                if (state.undo?.kind === "workspace") {
                  state.undo.result = null;
                }
              })
              .with({ status: "failed" }, ({ kind, message }) => {
                state.run = { status: "failed", kind, message, request: run.request };
              })
              .exhaustive();
          });
        },

        resultLost(sessionId) {
          set((state) => {
            if (state.undo?.kind === "workspace" && state.undo.result?.sessionId === sessionId) {
              state.undo.result = null;
            }

            const lost = state.result;

            if (lost?.sessionId !== sessionId) {
              return;
            }

            state.result = null;

            if (state.run.status !== "running") {
              state.run = { status: "failed", kind: "internal", message: RESULTS_LOST_MESSAGE, request: lost.request };
            }
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

export function selectTextIds(state: WorkspaceData): readonly number[] {
  const cached = textIdCache.get(state.trees);

  if (cached !== undefined) {
    return cached;
  }

  const textIds = state.trees.map(({ textId }) => textId);

  textIdCache.set(state.trees, textIds);

  return textIds;
}

export function selectStale(state: WorkspaceData): boolean {
  return state.result !== null && !isDeepEqual(selectRequest(state), state.result.request);
}

export function selectRunning(state: WorkspaceData): boolean {
  return state.run.status === "running";
}

function assignLabels(
  trees: readonly NewTree[],
  automaticLabels: readonly string[],
): { tree: NewTree; label: string }[] {
  const requested = trees.filter((tree) => tree.label === undefined).length;

  if (automaticLabels.length !== requested) {
    throw new Error(
      `The label service returned ${String(automaticLabels.length)} labels for ${String(requested)} trees.`,
    );
  }

  const remaining = automaticLabels.values();

  return trees.map((tree) => ({ tree, label: tree.label ?? takeLabel(remaining) }));
}

function takeLabel(labels: Iterator<string>): string {
  const next = labels.next();

  if (next.done === true) {
    throw new Error("The label service returned too few labels.");
  }

  return next.value;
}

function sessionTrees(
  request: AnalysisRequest,
  sources: readonly TreeSource[],
  first: { tree: number; text: number },
): WorkspaceTree[] {
  return request.trees.map(({ label, newick }, index) => ({
    id: `tree-${String(first.tree + index)}`,
    label,
    newick,
    textId: first.text + index,
    source: sources[index] ?? SESSION_SOURCE,
  }));
}

function replaceWorkspace(
  state: WorkspaceData,
  reason: WorkspaceReplacement,
  trees: WorkspaceTree[],
  settings: Settings,
): void {
  state.undo = { kind: "workspace", reason, trees: state.trees, settings: state.settings, result: state.result };
  state.trees = trees;
  state.settings = settings;
  state.result = null;
  state.run = { status: "idle" };
  state.restored = false;
  state.resetRevision += 1;
}

function markEdited(state: WorkspaceData): void {
  state.undo = null;
  state.restored = false;
}
