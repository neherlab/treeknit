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

interface StoreRelay {
  store: WorkspaceStore | null;
}

export async function startWorkspace(client: AnalysisClient, queryClient: QueryClient): Promise<WorkspaceRuntime> {
  const relay: StoreRelay = { store: null };

  const persistence = new WorkspacePersistence({
    storage: storageOrNull() ?? unavailableStorage(),
    channel: broadcastChannel(),
    requestFile: async (request) => client.requestFile(request),
    onSwitchChange: (enabled) => {
      relay.store?.getState().setPersistence(enabled);
    },
  });

  const [defaults, restored] = await Promise.all([client.defaultSettings(), restoreWorkspace(client, persistence)]);
  const store = createWorkspaceStore(client, { defaults, restored });

  relay.store = store;
  store.getState().setPersistence(persistence.enabled);

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

  return { store, persistence };
}

export function workspaceSnapshot(state: WorkspaceData): WorkspaceSnapshot {
  return { request: selectRequest(state), sources: state.trees.map(({ source }) => source) };
}

async function restoreWorkspace(
  client: AnalysisClient,
  persistence: WorkspacePersistence,
): Promise<RestoredWorkspace | null> {
  const stored = await persistence.restore().catch(() => null);

  if (stored === null) {
    return null;
  }

  const request = await client.readRequest(stored.sessionFile).catch(() => null);

  return request === null ? null : { request, sources: stored.sources };
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
