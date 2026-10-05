import type { QueryClient } from "@tanstack/react-query";

import type { AnalysisClient } from "../analysis/client";
import { analysisKeys } from "../analysis/queries";
import { broadcastChannel, indexedDbStorage } from "../persistence/browserStorage";
import { WorkspacePersistence, type WorkspaceSnapshot } from "../persistence/persistence";
import type { RecordStorage } from "../persistence/record";
import {
  createWorkspaceStore,
  type RestoredWorkspace,
  selectRequest,
  type WorkspaceData,
  type WorkspaceStore,
} from "./store";

export interface WorkspaceRuntime {
  store: WorkspaceStore;
  persistence: WorkspacePersistence;
}

export async function startWorkspace(client: AnalysisClient, queryClient: QueryClient): Promise<WorkspaceRuntime> {
  const persistence = new WorkspacePersistence({
    storage: storageOrNull() ?? unavailableStorage(),
    channel: broadcastChannel(),
    requestFile: async (request) => client.requestFile(request),
  });

  const [defaults, restored] = await Promise.all([
    client.defaultSettings(),
    persistence.restore(async (stored): Promise<RestoredWorkspace> => ({
      request: await client.readRequest(stored.sessionFile),
      sources: stored.sources,
    })),
  ]);

  const store = createWorkspaceStore(client, { defaults, restored });

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

  globalThis.addEventListener("pagehide", () => {
    void persistence.saveNow();
  });

  return { store, persistence };
}

export function workspaceSnapshot(state: WorkspaceData): WorkspaceSnapshot {
  return { request: selectRequest(state), sources: state.trees.map(({ source }) => source) };
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
