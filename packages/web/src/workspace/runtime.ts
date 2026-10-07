import type { QueryClient } from "@tanstack/react-query";

import type { AnalysisClient } from "../analysis/client";
import { analysisKeys } from "../analysis/queries";
import { createFocusStore, type FocusStore } from "../drawing/focus";
import { broadcastChannel, indexedDbStorage } from "../persistence/browserStorage";
import { WorkspacePersistence, type WorkspaceSnapshot } from "../persistence/persistence";
import type { RecordStorage } from "../persistence/record";
import {
  createWorkspaceStore,
  type RestoredWorkspace,
  selectRequest,
  type WorkspaceData,
  type WorkspaceServices,
  type WorkspaceStore,
} from "./store";
import { restoredSources } from "./treeSource";

export interface WorkspaceRuntime {
  store: WorkspaceStore;
  persistence: WorkspacePersistence;
  focus: FocusStore;
}

export async function startWorkspace(client: AnalysisClient, queryClient: QueryClient): Promise<WorkspaceRuntime> {
  const persistence = new WorkspacePersistence({
    storage: storageOrNull() ?? unavailableStorage(),
    channel: broadcastChannel(),
    sessionFile: async (request) => client.stateless(async (api) => api.sessionFile(request)),
  });

  const [defaults, restored] = await Promise.all([
    client.stateless(async (api) => api.defaultSettings()),
    persistence.restore(async (stored): Promise<RestoredWorkspace> => {
      const [request, sources] = await Promise.all([
        client.stateless(async (api) => api.readSession(stored.sessionFile)),
        restoredSources(stored.sources, async (url) => client.stateless(async (api) => api.urlPlace(url))),
      ]);

      return { request, sources };
    }),
  ]);

  const store = createWorkspaceStore(workspaceServices(client), { defaults, restored });

  store.subscribe((state, previous) => {
    if (state.trees !== previous.trees || state.settings !== previous.settings) {
      persistence.changed(workspaceSnapshot(state));
    }

    const previousSession = previous.result?.sessionId;

    if (previousSession !== undefined && previousSession !== state.result?.sessionId) {
      queryClient.removeQueries({ queryKey: analysisKeys.session(previousSession) });
    }
  });

  client.onSessionLost((sessionId) => {
    queryClient.removeQueries({ queryKey: analysisKeys.session(sessionId) });
    store.getState().resultLost(sessionId);
  });

  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") {
      void persistence.saveNow();
    }
  });

  return { store, persistence, focus: createFocusStore() };
}

export function workspaceSnapshot(state: WorkspaceData): WorkspaceSnapshot {
  return { request: selectRequest(state), sources: state.trees.map(({ source }) => source) };
}

function workspaceServices(client: AnalysisClient): WorkspaceServices {
  return {
    async treeLabels(...args) {
      return client.stateless(async (api) => api.treeLabels(...args));
    },
    async settingsSchema(...args) {
      return client.stateless(async (api) => api.settingsSchema(...args));
    },
    cancel() {
      client.cancel();
    },
  };
}

function storageOrNull(): RecordStorage | null {
  return "indexedDB" in globalThis ? indexedDbStorage() : null;
}

function unavailableStorage(): RecordStorage {
  return { read: rejectUnavailable, update: rejectUnavailable };
}

function rejectUnavailable(): Promise<never> {
  return Promise.reject(new Error("This browser does not provide IndexedDB storage."));
}
